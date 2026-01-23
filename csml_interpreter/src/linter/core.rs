use crate::data::{
    Literal,
    ast::{
        Block, DoType, Expr, FromFlow, Function, GotoType, GotoValueType, Identifier, IfStatement,
        InstructionScope, Interval, ObjectType, PathState,
    },
    position::Position,
    primitive::{PrimitiveClosure, PrimitiveType},
    tokens::{BUILT_IN, BUILT_IN_WITHOUT_WARNINGS, COMPONENT, Span},
    warnings::{WARNING_FN, WARNING_OBJECT, WARNING_USE, Warnings},
};
use crate::error_format::{
    ErrorInfo, convert_error_from_interval, gen_error_info, gen_infinite_loop_error_msg,
    gen_warning_info,
};
use crate::interpreter::variable_handler::interval::interval_from_expr;
use crate::linter::{
    ConstantInfo, FlowConstantUse, FlowToValidate, FunctionCallInfo, FunctionInfo, ImportInfo,
    InsertInfo, LinterInfo, ScopeType, State, StepBreakers, StepInfo,
};

use std::collections::{HashMap, HashSet};

pub(crate) const ERROR_GOTO_IN_FN: &str = "'goto' action is not allowed in function scope";
pub(crate) const ERROR_REMEMBER_IN_FN: &str = "'remember' action is not allowed in function scope";
pub(crate) const ERROR_SAY_IN_FN: &str = "'say' action is not allowed in function scope";
pub(crate) const ERROR_RETURN_IN_FN: &str = "'return' action is not allowed outside function scope";
pub(crate) const ERROR_BREAK_IN_LOOP: &str = "'break' action is not allowed outside loop";
pub(crate) const ERROR_CONTINUE_IN_LOOP: &str = "'continue' action is not allowed outside loop";
pub(crate) const ERROR_HOLD_IN_LOOP: &str = "'hold' action is not allowed in function scope";

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub(crate) fn lint_bot(
    flows: &[FlowToValidate],
    modules: &[FlowToValidate],
    errors: &mut Vec<ErrorInfo>,
    warnings: &mut Vec<Warnings>,
    native_components: Option<&serde_json::Map<String, serde_json::Value>>,
    default_flow: &str,
) {
    let scope_type = ScopeType::Step("start".to_owned());
    let mut bot_constants = HashMap::new();
    let mut goto_list = vec![];
    let mut step_list = HashSet::new();
    let mut function_list = HashSet::new();
    let mut import_list = HashSet::new();
    let mut insert_list = HashSet::new();
    let mut valid_closure_list = vec![];
    let mut functions_call_list = vec![];

    let mut linter_info = LinterInfo::new(
        "",
        scope_type,
        "",
        &mut goto_list,
        &mut step_list,
        &mut function_list,
        default_flow,
        &mut bot_constants,
        &mut import_list,
        &mut insert_list,
        &mut valid_closure_list,
        &mut functions_call_list,
        errors,
        warnings,
        native_components,
    );

    for flow in flows {
        linter_info.flow_name = &flow.flow_name;
        linter_info.raw_flow = flow.raw_flow;

        // init flow constant save box
        linter_info.bot_constants.insert(
            flow.flow_name.clone(),
            FlowConstantUse {
                constants: vec![],
                updated_vars: HashMap::new(),
            },
        );

        validate_flow_ast(flow, &mut linter_info, false);
    }

    for flow in modules {
        linter_info.flow_name = &flow.flow_name;
        linter_info.raw_flow = flow.raw_flow;

        validate_flow_ast(flow, &mut linter_info, true);
    }

    validate_gotos(&mut linter_info);
    validate_imports(&mut linter_info);
    validate_functions(&mut linter_info);
    validate_constants(&mut linter_info);
    validate_inserts(&mut linter_info);

    if let Some((infinite_loop, interval, flow)) = infinite_loop_check(
        &linter_info,
        vec![],
        &mut vec![],
        default_flow.to_owned(),
        "start".to_owned(),
    ) {
        linter_info.warnings.push(gen_warning_info(
            Position::new(interval, &flow),
            format!(
                "infinite loop detected between:\n {}",
                gen_infinite_loop_error_msg(&infinite_loop)
            ),
        ));
    }
}

// TODO
pub(crate) fn validate_gotos(_linter_info: &mut LinterInfo) {
    // for goto_info in &*linter_info.goto_list {
    //     if goto_info.step == "end" {
    //         continue;
    //     }
    //
    //     // toto
    //
    //     // if let None = linter_info.step_list.get(&goto_info) {
    //     //     linter_info.errors.push(gen_error_info(
    //     //         Position::new(goto_info.interval.to_owned(), &goto_info.in_flow),
    //     //         convert_error_from_interval(
    //     //             Span::new(goto_info.raw_flow),
    //     //             format!(
    //     //                 "step {} at flow {} does not exist",
    //     //                 goto_info.step, goto_info.flow
    //     //             ),
    //     //             goto_info.interval.to_owned(),
    //     //         ),
    //     //     ));
    //     // }
    // }
}

pub(crate) fn validate_imports(linter_info: &mut LinterInfo) {
    'outer: for import_info in &*linter_info.import_list {
        let extern_module = matches!(import_info.from_flow, FromFlow::Extern(_));

        if linter_info
            .function_list
            .get(&FunctionInfo::new(
                import_info.as_name.clone(),
                import_info.in_flow,
                import_info.interval,
                extern_module,
            ))
            .is_some()
        {
            gen_function_error(
                linter_info.errors,
                import_info.raw_flow,
                linter_info.flow_name,
                import_info.interval,
                &format!(
                    "import failed a function named '{}' already exist in current flow '{}'",
                    import_info.as_name, import_info.in_flow
                ),
            );
        }

        match import_info {
            ImportInfo {
                as_name,
                original_name,
                from_flow: FromFlow::Normal(flow),
                raw_flow,
                interval,
                in_flow,
            } => {
                let as_name = original_name.as_ref().unwrap_or(as_name);

                if linter_info
                    .function_list
                    .get(&FunctionInfo::new(as_name.clone(), flow, *interval, false))
                    .is_none()
                {
                    gen_function_error(
                        linter_info.errors,
                        raw_flow,
                        in_flow,
                        *interval,
                        &format!("import failed function '{as_name}' not found in flow '{flow}'"),
                    );
                }
            }
            ImportInfo {
                as_name,
                original_name,
                from_flow: FromFlow::Extern(flow),
                raw_flow,
                interval,
                in_flow,
            } => {
                let as_name = original_name.as_ref().unwrap_or(as_name);

                if linter_info
                    .function_list
                    .get(&FunctionInfo::new(as_name.clone(), flow, *interval, true))
                    .is_none()
                {
                    gen_function_error(
                        linter_info.errors,
                        raw_flow,
                        in_flow,
                        *interval,
                        &format!("import failed function '{as_name}' not found in flow '{flow}'"),
                    );
                }
            }
            ImportInfo {
                as_name,
                original_name,
                raw_flow,
                interval,
                ..
            } => {
                let as_name = original_name.as_ref().unwrap_or(as_name);

                for function in &*linter_info.function_list {
                    if &function.name == as_name {
                        continue 'outer;
                    }
                }

                gen_function_error(
                    linter_info.errors,
                    raw_flow,
                    linter_info.flow_name,
                    *interval,
                    &format!("function '{as_name}' not found in bot",),
                );
            }
        }
    }
}

pub(crate) fn validate_inserts(linter_info: &mut LinterInfo) {
    for insert_info in &*linter_info.insert_list {
        if linter_info
            .step_list
            .get(&StepInfo::new(
                insert_info.in_flow,
                &insert_info.as_name,
                insert_info.raw_flow,
                insert_info.in_flow.to_owned(),
                vec![],
                insert_info.interval,
            ))
            .is_some()
        {
            gen_function_error(
                linter_info.errors,
                insert_info.raw_flow,
                linter_info.flow_name,
                insert_info.interval,
                &format!(
                    "insert failed, a step named '{}' already exist in current flow '{}'",
                    insert_info.as_name, insert_info.in_flow
                ),
            );
        }

        let as_name = insert_info
            .original_name
            .as_deref()
            .unwrap_or(&insert_info.as_name);

        if linter_info
            .step_list
            .get(&StepInfo::new(
                &insert_info.from_flow,
                as_name,
                insert_info.raw_flow,
                insert_info.in_flow.to_owned(),
                vec![],
                insert_info.interval,
            ))
            .is_none()
        {
            gen_function_error(
                linter_info.errors,
                insert_info.raw_flow,
                insert_info.in_flow,
                insert_info.interval,
                &format!(
                    "insert failed, step '{}' not found in flow '{}'",
                    as_name, insert_info.from_flow
                ),
            );
        }
    }
}

pub(crate) fn validate_functions(linter_info: &mut LinterInfo) {
    for info in &*linter_info.functions_call_list {
        let is_native_component = linter_info
            .native_components
            .is_some_and(|native_components| native_components.contains_key(&info.name));

        if !is_native_component
            && !BUILT_IN.contains(&info.name.as_str())
            && !BUILT_IN_WITHOUT_WARNINGS.contains(&info.name.as_str())
            && COMPONENT != info.name
            && !validate_closure(info, linter_info)
            && !function_exist(info, linter_info)
        {
            linter_info.errors.push(gen_error_info(
                Position::new(info.interval, info.in_flow),
                convert_error_from_interval(
                    &Span::new(info.raw_flow),
                    &format!("function [{}] does not exist", info.name),
                    info.interval,
                ),
            ));
        }
    }
}

pub(crate) fn validate_constants(linter_info: &mut LinterInfo) {
    for (flow, constant_info) in &*linter_info.bot_constants {
        for constant in &constant_info.constants {
            if let Some(interval) = constant_info.updated_vars.get(&constant.name) {
                linter_info.errors.push(gen_error_info(
                    Position::new(*interval, flow),
                    convert_error_from_interval(
                        &Span::new(constant.raw_flow),
                        &format!(
                            "constant '{}' is immutable and can not be changed",
                            constant.name
                        ),
                        *interval,
                    ),
                ));
            }
        }
    }
}

pub(crate) fn validate_flow_ast(
    flow: &FlowToValidate,
    linter_info: &mut LinterInfo,
    extern_module: bool,
) {
    let mut is_step_start_present = false;
    let mut steps_nbr = 0;

    // save all flow  constant info in linter_info
    for constant in flow.ast.constants.keys() {
        if let Some(flow_constants) = linter_info.bot_constants.get_mut(linter_info.flow_name) {
            flow_constants.constants.push(ConstantInfo {
                name: constant.clone(),
                raw_flow: linter_info.raw_flow,
            });
        }
    }

    for (instruction_scope, scope) in &flow.ast.flow_instructions {
        match instruction_scope {
            InstructionScope::StepScope(step_name) => {
                steps_nbr += 1;
                if step_name == "start" {
                    is_step_start_present = true;
                }
                linter_info.scope_type = ScopeType::Step(step_name.clone());

                if let Expr::Scope { scope, range, .. } = scope {
                    let mut step_breakers = vec![];

                    validate_scope(
                        scope,
                        &mut State::new(0),
                        linter_info,
                        &mut Some(&mut step_breakers),
                    );

                    linter_info.step_list.insert(StepInfo::new(
                        &flow.flow_name,
                        step_name,
                        linter_info.raw_flow,
                        flow.flow_name.clone(),
                        step_breakers,
                        *range,
                    ));
                }
            }
            InstructionScope::FunctionScope { name, .. } => {
                let save_step_name = linter_info.scope_type.clone();
                linter_info.scope_type = ScopeType::Function(name.clone());

                if let Expr::Scope { scope, .. } = scope {
                    validate_scope(scope, &mut State::new(1), linter_info, &mut None);
                }

                linter_info.scope_type = save_step_name;

                linter_info.function_list.insert(FunctionInfo::new(
                    name.clone(),
                    linter_info.flow_name,
                    interval_from_expr(scope),
                    extern_module,
                ));
            }
            InstructionScope::ImportScope(import_scope) => {
                linter_info.import_list.insert(ImportInfo::new(
                    import_scope.name.clone(),
                    import_scope.original_name.clone(),
                    import_scope.from_flow.clone(),
                    linter_info.flow_name,
                    linter_info.raw_flow,
                    import_scope.interval,
                ));
            }

            InstructionScope::InsertStep(insert_step) => {
                linter_info.insert_list.insert(InsertInfo::new(
                    insert_step.name.clone(),
                    insert_step.original_name.clone(),
                    insert_step.from_flow.clone(),
                    linter_info.flow_name,
                    linter_info.raw_flow,
                    insert_step.interval,
                ));
            }

            InstructionScope::Constant(_) => {}

            InstructionScope::DuplicateInstruction(interval, info) => {
                linter_info.errors.push(gen_error_info(
                    Position::new(*interval, linter_info.flow_name),
                    convert_error_from_interval(
                        &Span::new(flow.raw_flow),
                        &format!("duplicate {info}"),
                        *interval,
                    ),
                ));
            }
        }
    }

    if !is_step_start_present && (steps_nbr > 0 && linter_info.default_flow != flow.flow_name) {
        linter_info.errors.push(gen_error_info(
            Position::new(Interval::default(), linter_info.flow_name),
            format!("missing step 'start' in flow [{}]", flow.flow_name),
        ));
    }
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn register_closure(
    name: &Identifier,
    is_permanent: bool,
    expr: &Expr,
    linter_info: &mut LinterInfo,
) {
    if let Expr::LitExpr { literal, .. } = expr {
        // register closure var name for function validation
        if literal.primitive.get_type() == PrimitiveType::PrimitiveClosure {
            linter_info.valid_closure_list.push(FunctionCallInfo::new(
                name.ident.clone(),
                linter_info.flow_name,
                linter_info.scope_type.clone(),
                is_permanent,
                linter_info.raw_flow,
                name.interval,
            ));
        }
    }
}

fn register_flow_breaker(
    step_breakers: &mut Option<&mut Vec<StepBreakers>>,
    breaker: StepBreakers,
) {
    if let Some(step_breakers) = step_breakers {
        step_breakers.push(breaker);
    }
}

fn is_in_list(list: &[(String, String)], flow: &str, step: &str) -> bool {
    list.iter()
        .any(|(next_flow, next_step)| flow == next_flow && step == next_step)
}

fn validate_expr_literals(to_be_literal: &Expr, state: &mut State, linter_info: &mut LinterInfo) {
    match to_be_literal {
        Expr::ObjectExpr(ObjectType::As(name, value)) => {
            register_closure(name, false, value, linter_info);

            validate_expr_literals(value, state, linter_info);
        }
        Expr::PathExpr { literal, path } => {
            validate_expr_literals(literal, state, linter_info);
            for (_, node) in path {
                match node {
                    PathState::ExprIndex(expr) => validate_expr_literals(expr, state, linter_info),
                    PathState::Func(Function { args, .. }) => {
                        validate_expr_literals(args, state, linter_info);
                    }
                    PathState::StringIndex(_) => {}
                }
            }
        }
        Expr::ObjectExpr(ObjectType::BuiltIn(Function {
            name,
            args,
            interval,
        })) => {
            if name == "Object" {
                linter_info.warnings.push(Warnings::new(
                    linter_info.flow_name,
                    *interval,
                    WARNING_OBJECT,
                ));
            } else if name == "Fn" {
                linter_info.warnings.push(Warnings::new(
                    linter_info.flow_name,
                    *interval,
                    WARNING_FN,
                ));
            }

            linter_info.functions_call_list.push(FunctionCallInfo::new(
                name.clone(),
                linter_info.flow_name,
                linter_info.scope_type.clone(),
                false,
                linter_info.raw_flow,
                *interval,
            ));

            validate_expr_literals(args, state, linter_info);
        }
        Expr::MapExpr { object, .. } => {
            for expr in object.values() {
                validate_expr_literals(expr, state, linter_info);
            }
        }
        Expr::VecExpr(vec, ..) | Expr::ComplexLiteral(vec, ..) => {
            for expr in vec {
                validate_expr_literals(expr, state, linter_info);
            }
        }
        Expr::InfixExpr(_, exp_1, exp_2) => {
            validate_expr_literals(exp_1, state, linter_info);
            validate_expr_literals(exp_2, state, linter_info);
        }
        Expr::LitExpr { literal, .. } => {
            if literal.primitive.get_type() == PrimitiveType::PrimitiveClosure
                && let Ok(closure) = Literal::get_value::<PrimitiveClosure, _>(
                    &literal.primitive,
                    linter_info.flow_name,
                    literal.interval,
                    String::new(),
                )
                && let Expr::Scope { scope, .. } = &*closure.func
            {
                state.in_function += 1;
                validate_scope(scope, state, linter_info, &mut None);
                state.in_function -= 1;
            }
        }
        Expr::ObjectExpr(ObjectType::Assign(_assign, target, new)) => {
            validate_expr_literals(target, state, linter_info);
            validate_expr_literals(new, state, linter_info);
        }
        // Expr::IdentExpr(..) => {}
        _ => {}
    }
}

fn validate_if_scope(
    if_statement: &IfStatement,
    state: &mut State,
    linter_info: &mut LinterInfo,
    step_breakers: &mut Option<&mut Vec<StepBreakers>>,
) {
    match if_statement {
        IfStatement::IfStmt {
            consequence,
            then_branch,
            ..
        } => {
            validate_scope(consequence, state, linter_info, step_breakers);

            if let Some(else_scope) = then_branch {
                validate_if_scope(else_scope, state, linter_info, step_breakers);
            }
        }
        IfStatement::ElseStmt(block, ..) => {
            validate_scope(block, state, linter_info, step_breakers);
        }
    }
}

fn validate_scope(
    scope: &Block,
    state: &mut State,
    linter_info: &mut LinterInfo,
    step_breakers: &mut Option<&mut Vec<StepBreakers>>,
) {
    for (action, _) in &scope.commands {
        match action {
            Expr::ObjectExpr(ObjectType::Return(value)) => {
                if state.in_function == 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(interval_from_expr(value), linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_RETURN_IN_FN,
                            interval_from_expr(value),
                        ),
                    ));
                }
            }
            Expr::ObjectExpr(ObjectType::Goto(goto, interval)) => {
                if state.in_function > 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(*interval, linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_GOTO_IN_FN,
                            *interval,
                        ),
                    ));
                }

                match goto {
                    GotoType::Step(GotoValueType::Name(step))
                    | GotoType::StepFlow {
                        step: Some(GotoValueType::Name(step)),
                        flow: None,
                        bot: None,
                    } => {
                        register_flow_breaker(
                            step_breakers,
                            StepBreakers::Goto {
                                flow: linter_info.flow_name.to_owned(),
                                step: step.ident.clone(),
                                interval: *interval,
                            },
                        );

                        linter_info.goto_list.push(StepInfo::new(
                            linter_info.flow_name,
                            &step.ident,
                            linter_info.raw_flow,
                            linter_info.flow_name.to_owned(),
                            vec![],
                            *interval,
                        ));
                    }
                    GotoType::Flow(GotoValueType::Name(flow))
                    | GotoType::StepFlow {
                        step: None,
                        flow: Some(GotoValueType::Name(flow)),
                        bot: None,
                    } => {
                        register_flow_breaker(
                            step_breakers,
                            StepBreakers::Goto {
                                flow: flow.ident.clone(),
                                step: "start".to_owned(),
                                interval: *interval,
                            },
                        );

                        linter_info.goto_list.push(StepInfo::new(
                            &flow.ident,
                            "start",
                            linter_info.raw_flow,
                            linter_info.flow_name.to_owned(),
                            vec![],
                            *interval,
                        ));
                    }
                    GotoType::StepFlow {
                        step: Some(GotoValueType::Name(step)),
                        flow: Some(GotoValueType::Name(flow)),
                        bot: None,
                    } => {
                        register_flow_breaker(
                            step_breakers,
                            StepBreakers::Goto {
                                flow: flow.ident.clone(),
                                step: step.ident.clone(),
                                interval: *interval,
                            },
                        );

                        linter_info.goto_list.push(StepInfo::new(
                            &flow.ident,
                            &step.ident,
                            linter_info.raw_flow,
                            linter_info.flow_name.to_owned(),
                            vec![],
                            *interval,
                        ));
                    }
                    _ => {}
                }
            }

            Expr::ObjectExpr(ObjectType::Break(interval)) => {
                if state.loop_scope == 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(*interval, linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_BREAK_IN_LOOP,
                            *interval,
                        ),
                    ));
                }
            }
            Expr::ObjectExpr(ObjectType::Continue(interval)) => {
                if state.loop_scope == 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(*interval, linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_CONTINUE_IN_LOOP,
                            *interval,
                        ),
                    ));
                }
            }

            Expr::ObjectExpr(ObjectType::Hold(interval)) => {
                register_flow_breaker(step_breakers, StepBreakers::Hold);

                if state.in_function > 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(*interval, linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_HOLD_IN_LOOP,
                            *interval,
                        ),
                    ));
                }
            }
            Expr::ObjectExpr(ObjectType::Say(value)) => {
                if state.in_function > 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(interval_from_expr(value), linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_SAY_IN_FN,
                            interval_from_expr(value),
                        ),
                    ));
                }

                validate_expr_literals(value, state, linter_info);
            }

            Expr::ObjectExpr(ObjectType::Use(value)) => {
                linter_info.warnings.push(Warnings::new(
                    linter_info.flow_name,
                    interval_from_expr(value),
                    WARNING_USE,
                ));
                validate_expr_literals(value, state, linter_info);
            }

            Expr::ObjectExpr(ObjectType::Do(DoType::Update(_assign, target, new))) => {
                if let Expr::IdentExpr(name) = &**target {
                    if let Some(flow_constants) =
                        linter_info.bot_constants.get_mut(linter_info.flow_name)
                    {
                        flow_constants
                            .updated_vars
                            .insert(name.ident.clone(), name.interval);
                    }

                    register_closure(name, false, new, linter_info);
                }

                validate_expr_literals(target, state, linter_info);
                validate_expr_literals(new, state, linter_info);
            }
            Expr::ObjectExpr(ObjectType::Do(DoType::Exec(expr))) => {
                validate_expr_literals(expr, state, linter_info);
            }

            Expr::ObjectExpr(ObjectType::Remember(name, value)) => {
                register_closure(name, true, value, linter_info);

                if state.in_function > 0 {
                    linter_info.errors.push(gen_error_info(
                        Position::new(name.interval, linter_info.flow_name),
                        convert_error_from_interval(
                            &Span::new(linter_info.raw_flow),
                            ERROR_REMEMBER_IN_FN,
                            name.interval,
                        ),
                    ));
                }
                validate_expr_literals(value, state, linter_info);
            }

            Expr::IfExpr(if_statement) => {
                validate_if_scope(if_statement, state, linter_info, step_breakers);
            }
            Expr::ForEachExpr(_ident, _index, _expr, block, _range) => {
                state.enter_loop();
                validate_scope(block, state, linter_info, step_breakers);
                state.exit_loop();
            }
            Expr::WhileExpr(_expr, block, _range) => {
                state.enter_loop();
                validate_scope(block, state, linter_info, step_breakers);
                state.exit_loop();
            }
            _ => {}
        }
    }
}

fn gen_function_error(
    errors: &mut Vec<ErrorInfo>,
    raw_flow: &str,
    flow_name: &str,
    interval: Interval,
    message: &str,
) {
    errors.push(gen_error_info(
        Position::new(interval, flow_name),
        convert_error_from_interval(&Span::new(raw_flow), message, interval),
    ));
}

fn function_exist(info: &FunctionCallInfo, linter_info: &LinterInfo) -> bool {
    if linter_info
        .function_list
        .iter()
        .any(|func| func.name == info.name && func.in_flow == info.in_flow)
    {
        return true;
    }

    linter_info
        .import_list
        .iter()
        .any(|import| import.as_name == info.name && import.in_flow == info.in_flow)
}

fn validate_closure(info: &FunctionCallInfo, linter_info: &LinterInfo) -> bool {
    linter_info.valid_closure_list.iter().any(|func| {
        func.name == info.name && (func.scope_type == info.scope_type || func.is_permanent)
    })
}

fn add_to_step_list(
    hold_detected: bool,
    mut step_list: Vec<(String, String)>, // flow, step
    search_step_info: &StepInfo,
    flow: String,
    step: String,
) -> Vec<(String, String)> {
    if !hold_detected {
        if step_list.is_empty() {
            step_list.push((search_step_info.flow.clone(), search_step_info.step.clone()));
        }
        step_list.push((flow, step));
    }

    step_list
}

#[allow(clippy::type_complexity)]
fn infinite_loop_check(
    linter_info: &LinterInfo,
    mut step_list: Vec<(String, String)>,   // flow, step
    close_list: &mut Vec<(String, String)>, // flow, step
    previews_flow: String,
    previews_step: String,
) -> Option<(Vec<(String, String)>, Interval, String)> {
    let search_step_info = StepInfo {
        flow: previews_flow.clone(),
        step: previews_step,
        raw_flow: "",
        in_flow: String::new(),
        step_breakers: vec![],
        interval: Interval::default(),
    };

    if let Some(step_info) = linter_info.step_list.get(&search_step_info) {
        let mut hold_detected = false;

        for breaker in &step_info.step_breakers {
            match breaker {
                StepBreakers::Hold => {
                    hold_detected = true;
                    step_list.clear();
                }
                StepBreakers::Goto {
                    flow,
                    step,
                    interval,
                } => {
                    let is_infinite_loop = is_in_list(&step_list, flow, step);
                    if is_infinite_loop {
                        step_list.push((flow.clone(), step.clone()));
                        return Some((step_list.clone(), *interval, previews_flow));
                    }

                    if is_in_list(close_list, flow, step) {
                        continue;
                    }
                    close_list.push((flow.clone(), step.clone()));

                    let new_step_list = add_to_step_list(
                        hold_detected,
                        step_list.clone(),
                        &search_step_info,
                        flow.clone(),
                        step.clone(),
                    );

                    if let Some(infinite_loop_vec) = infinite_loop_check(
                        linter_info,
                        new_step_list,
                        close_list,
                        flow.clone(),
                        step.clone(),
                    ) {
                        return Some(infinite_loop_vec);
                    }
                }
            }
        }
    }

    None
}
