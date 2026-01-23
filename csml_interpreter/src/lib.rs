pub mod data;
pub mod error_format;
pub(crate) mod fold_bot;
pub mod interpreter;
pub(crate) mod linter;
pub mod parser;

pub use interpreter::components::load_components;
pub use parser::step_checksum::get_step;

use interpreter::{interpret_scope, json_to_literal};
use parser::parse_flow;

use data::CsmlResult;
use data::ast::{Expr, Flow, InsertStep, InstructionScope, Interval};
use data::context::{ContextStepInfo, get_hashmap_from_mem};
use data::error_info::ErrorInfo;
use data::event::Event;
use data::literal::create_error_info;
use data::message_data::MessageData;
use data::msg::MSG;
use data::{Context, Data, Position, STEP_LIMIT};
use data::{CsmlFlow, csml_bot::CsmlBot};
use error_format::{ERROR_STEP_EXIST, ERROR_STEP_LIMIT, gen_error_info};
use fold_bot::fold_bot as fold;
use linter::{FlowToValidate, core::lint_bot};
use parser::ExitCondition;

use base64::Engine;
use std::collections::HashMap;
use std::env;
use std::sync::mpsc;

pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn execute_step(
    step: &str,
    flow: &Flow,
    data: &mut Data,
    sender: Option<&mpsc::Sender<MSG>>,
) -> MessageData {
    // stop execution if step_count >= STEP_LIMIT to avoid infinite loops
    if *data.step_count >= data.step_limit {
        let msg_data = Err(gen_error_info(
            Position::new(
                Interval::new_as_u32(0, 0, 0, None, None),
                &data.context.flow,
            ),
            format!("{ERROR_STEP_LIMIT}, stop at step {step}"),
        ));

        return MessageData::error_to_message(msg_data, sender);
    }

    let mut msg_data = match flow
        .flow_instructions
        .get(&InstructionScope::StepScope(step.to_owned()))
    {
        Some(Expr::Scope { scope, .. }) => {
            *data.step_count += 1;
            interpret_scope(scope, data, sender)
        }
        _ => Err(gen_error_info(
            Position::new(
                Interval::new_as_u32(0, 0, 0, None, None),
                &data.context.flow,
            ),
            format!("[{step}] {ERROR_STEP_EXIST}"),
        )),
    };

    if let Ok(msg_data) = &mut msg_data {
        match &mut msg_data.exit_condition {
            Some(condition) if *condition == ExitCondition::Goto => {
                msg_data.exit_condition = None;
            }
            Some(_) => (),
            // if no goto at the end of the scope end conversation
            None => {
                msg_data.exit_condition = Some(ExitCondition::End);
                data.context.step = ContextStepInfo::Normal("end".to_string());
                MSG::send(
                    sender,
                    MSG::Next {
                        flow: None,
                        step: Some(ContextStepInfo::Normal("end".to_owned())),
                        bot: None,
                    },
                );
            }
        }
    }

    MessageData::error_to_message(msg_data, sender)
}

fn get_step_limit(event: &Event) -> usize {
    if let Some(step_limit) = event.step_limit {
        return step_limit;
    }
    if let Ok(step_limit) = env::var("STEP_LIMIT") {
        return step_limit.parse::<usize>().unwrap_or(STEP_LIMIT);
    }
    STEP_LIMIT
}

fn get_flow_ast<'a, 'b>(
    flows: &'a HashMap<String, Flow>,
    flow: &'b str,
    bot_id: &'b str,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<&'a Flow, Box<MessageData>> {
    flows.get(flow).ok_or_else(|| {
        let error_message = format!("flow: [{flow}] does not exist in bot: [{bot_id}]");
        let error_info = create_error_info(&error_message, Interval::default());

        Box::new(MessageData::error_to_message(
            Err(ErrorInfo {
                position: Position {
                    flow: flow.to_owned(),
                    interval: Interval::default(),
                },
                message: error_message,
                additional_info: Some(Box::new(error_info)),
            }),
            sender,
        ))
    })
}

fn get_inserted_ast<'a>(
    flows: &'a HashMap<String, Flow>,
    ast: &'a Flow,
    step: &ContextStepInfo,
    bot_id: &str,
    sender: Option<&mpsc::Sender<MSG>>,
) -> (bool, Option<&'a Flow>) {
    match &step {
        ContextStepInfo::Normal(step) => {
            let missing_step = !ast
                .flow_instructions
                .contains_key(&InstructionScope::StepScope(step.clone()));

            (missing_step, None)
        }
        ContextStepInfo::UnknownFlow(step_name) => {
            let missing_step = !ast
                .flow_instructions
                .contains_key(&InstructionScope::StepScope(step_name.clone()));

            if missing_step {
                match ast
                    .flow_instructions
                    .get_key_value(&InstructionScope::InsertStep(InsertStep {
                        name: step_name.clone(),
                        original_name: None,
                        from_flow: String::new(),
                        interval: Interval::default(),
                    })) {
                    Some((InstructionScope::InsertStep(insert_step), _expr)) => {
                        let step = ContextStepInfo::InsertedStep {
                            step: step_name.clone(),
                            flow: insert_step.from_flow.clone(),
                        };

                        get_inserted_ast(flows, ast, &step, bot_id, sender)
                    }
                    _ => (missing_step, None),
                }
            } else {
                (missing_step, None)
            }
        }
        ContextStepInfo::InsertedStep { step, flow } => {
            match get_flow_ast(flows, flow, bot_id, sender) {
                Ok(inserted_ast) => {
                    let missing_step = !inserted_ast
                        .flow_instructions
                        .contains_key(&InstructionScope::StepScope(step.clone()));

                    (missing_step, Some(inserted_ast))
                }
                Err(_) => (true, None),
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

struct BotParseResult<'a> {
    pub flows: Vec<FlowToValidate<'a>>,
    pub modules: Vec<FlowToValidate<'a>>,
    pub errors: Vec<ErrorInfo>,
}

impl BotParseResult<'_> {
    pub fn new<'a>(
        flows: Vec<FlowToValidate<'a>>,
        modules: Vec<FlowToValidate<'a>>,
        errors: Vec<ErrorInfo>,
    ) -> BotParseResult<'a> {
        BotParseResult {
            flows,
            modules,
            errors,
        }
    }
}

#[must_use]
pub fn get_steps_from_flow(bot: &CsmlBot) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();

    for flow in &bot.flows {
        if let Ok(parsed_flow) = parse_flow(&flow.content, &flow.name) {
            let mut vec = vec![];

            for instruction_type in parsed_flow.flow_instructions.keys() {
                if let InstructionScope::StepScope(step_name, ..) = instruction_type {
                    vec.push(step_name.clone());
                }
            }
            result.insert(flow.name.clone(), vec);
        }
    }
    result
}

#[must_use]
pub fn validate_bot(bot: &CsmlBot) -> CsmlResult {
    let BotParseResult {
        flows,
        modules,
        mut errors,
    } = parse_bot(bot);

    let mut warnings = vec![];
    // only use the linter if there is no error in the paring, otherwise the linter will catch false errors
    if errors.is_empty() {
        lint_bot(
            &flows,
            &modules,
            &mut errors,
            &mut warnings,
            bot.native_components.as_ref(),
            &bot.default_flow,
        );
    }

    CsmlResult::new(
        FlowToValidate::get_flows(flows),
        FlowToValidate::get_flows(modules),
        warnings,
        errors,
    )
}

#[must_use]
pub fn fold_bot(bot: &CsmlBot) -> String {
    let BotParseResult {
        flows,
        modules,
        mut errors,
    } = parse_bot(bot);

    let mut warnings = vec![];
    // only use the fold if there is no error in the paring, otherwise the linter will catch false errors

    fold(
        &flows,
        &modules,
        &mut errors,
        &mut warnings,
        bot.native_components.as_ref(),
        &bot.default_flow,
    )
}

#[must_use]
fn parse_bot(bot: &CsmlBot) -> BotParseResult<'_> {
    let mut flows = vec![];
    let mut modules = vec![];
    let mut errors = Vec::new();
    let mut imports = Vec::new();

    for flow in &bot.flows {
        match parse_flow(&flow.content, &flow.name) {
            Ok(ast_flow) => {
                for (scope, ..) in &ast_flow.flow_instructions {
                    if let InstructionScope::ImportScope(import_scope) = scope {
                        imports.push(import_scope.clone());
                    }
                }

                // flows.insert(flow.name.to_owned(), ast_flow);
                flows.push(FlowToValidate {
                    flow_name: flow.name.clone(),
                    ast: ast_flow,
                    raw_flow: &flow.content,
                });
            }
            Err(error) => {
                errors.push(error);
            }
        }
    }
    if let Some(ref mods) = bot.modules {
        for module in mods {
            if let Some(flow) = &module.flow {
                match parse_flow(&flow.content, &flow.name) {
                    Ok(ast_flow) => {
                        modules.push(FlowToValidate {
                            flow_name: flow.name.clone(),
                            ast: ast_flow,
                            raw_flow: &flow.content,
                        });
                    }
                    Err(error) => {
                        errors.push(error);
                    }
                }
            }
        }
    }
    BotParseResult::new(flows, modules, errors)
}

#[must_use]
fn get_flows(bot: &CsmlBot) -> (HashMap<String, Flow>, HashMap<String, Flow>) {
    let Some(bot) = &bot.bot_ast else {
        let CsmlResult {
            flows,
            extern_flows,
            ..
        } = validate_bot(bot);

        return (flows, extern_flows);
    };

    let base64decoded = base64::engine::general_purpose::STANDARD
        .decode(bot)
        .unwrap();
    bincode::decode_from_slice(&base64decoded[..], BINCODE_CONFIG)
        .unwrap()
        .0
}

pub fn search_for_modules(bot: &mut CsmlBot) -> Result<(), String> {
    let default_auth = env::var("MODULES_AUTH").ok();
    let default_url = env::var("MODULES_URL").ok();

    if let Some(ref mut modules) = bot.modules {
        for module in &mut *modules {
            if module.flow.is_some() {
                // module already downloaded
                continue;
            }
            let client = reqwest::blocking::Client::new();
            let url_and_auth = module
                .url
                .as_ref()
                .map(|url| (url, module.auth.as_ref()))
                .or_else(|| default_url.as_ref().map(|url| (url, default_auth.as_ref())));
            let Some((url, auth)) = url_and_auth else {
                return Err(format!(
                    "missing url in order to get module [{}]",
                    module.name
                ));
            };

            let mut request = client.request(reqwest::Method::GET, url);
            if let Some(auth) = auth {
                let authorization = format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD.encode(auth.as_bytes())
                );
                request = request.header("Authorization", &authorization);
            }

            match request.send() {
                Ok(response) => {
                    let Ok(flow_content) = response.text() else {
                        return Err(format!("invalid module {}", module.name));
                    };

                    module.flow = Some(CsmlFlow {
                        id: module.name.clone(),
                        name: module.name.clone(),
                        content: flow_content,
                        commands: vec![],
                    });
                }
                Err(error) => return Err(error.to_string()),
            }
        }
    }

    Ok(())
}

#[must_use]
pub fn interpret(
    bot: &CsmlBot,
    mut context: Context,
    event: &Event,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Box<MessageData> {
    let mut msg_data = MessageData::default();

    let mut flow = context.flow.clone();
    let mut step = context.step.clone();

    let mut step_count = 0;
    let step_limit = get_step_limit(event);

    let mut step_vars = context
        .hold
        .as_ref()
        .map(|hold| get_hashmap_from_mem(&hold.step_vars, &flow))
        .unwrap_or_default();

    let native = bot.native_components.clone().unwrap_or_default();

    let custom = bot
        .custom_components
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_default();

    let (flows, extern_flows) = get_flows(bot);

    let env = match &bot.env {
        Some(env) => json_to_literal(env, Interval::default(), &flow).unwrap(),
        None => data::primitive::PrimitiveNull::get_literal(Interval::default()),
    };

    let mut previous_info = context.hold.as_ref().and_then(|hold| hold.previous.clone());

    while msg_data.exit_condition.is_none() {
        let ast = match get_flow_ast(&flows, &flow, &bot.id, sender) {
            Ok(ast) => ast,
            Err(message_data) => return message_data,
        };

        let (missing_step, inserted_ast) = get_inserted_ast(&flows, ast, &step, &bot.id, sender);

        // if the target flow dose not contains a 'start' flow change the target to the default_flow
        if step.is_start() && missing_step {
            flow.clone_from(&bot.default_flow);
            continue;
        }

        let mut data = Data::new(
            &flows,
            &extern_flows,
            ast,
            bot.default_flow.clone(),
            &mut context,
            event,
            &env,
            vec![],
            0,
            &mut step_count,
            step_limit,
            step_vars,
            previous_info.clone(),
            &custom,
            &native,
        );

        msg_data = match inserted_ast {
            Some(inserted_ast) => {
                msg_data + execute_step(step.get_step(), inserted_ast, &mut data, sender)
            }
            None => msg_data + execute_step(step.get_step(), ast, &mut data, sender),
        };

        previous_info.clone_from(&data.previous_info);
        flow.clone_from(&data.context.flow);
        step.clone_from(&data.context.step);

        // add reset loops index
        step_vars = HashMap::new();
    }

    Box::new(msg_data)
}
