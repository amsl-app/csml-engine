use crate::data::position::Position;
use crate::data::primitive::common;
use crate::data::primitive::common::{
    get_args, get_index_args, get_int_arg, get_int_args, get_literal_args, get_string_args,
};
use crate::data::primitive::utils::{arg_name, illegal_math_ops, impl_basic_cmp, require_n_args};
use crate::data::{
    ArgsType, Interval, Literal, MSG, MemoryType, Message, MessageData,
    core::{Data, init_child_context, init_child_scope},
    literal,
    literal::ContentType,
    primitive::{
        Primitive, PrimitiveBoolean, PrimitiveClosure, PrimitiveInt, PrimitiveNull,
        PrimitiveString, PrimitiveType, Right,
    },
    tokens::TYPES,
};
use crate::error_format::{
    ERROR_ARRAY_BOUNDS, ERROR_ARRAY_INDEX, ERROR_ARRAY_INIT, ERROR_ARRAY_INSERT_AT,
    ERROR_ARRAY_INSERT_AT_INT, ERROR_ARRAY_JOIN, ERROR_ARRAY_OVERFLOW, ERROR_ARRAY_POP,
    ERROR_ARRAY_REMOVE_AT, ERROR_ARRAY_UNKNOWN_METHOD, ERROR_CONSTANT_MUTABLE_FUNCTION, ErrorInfo,
    OVERFLOWING_OPERATION, gen_error_info,
};
use crate::interpreter::variable_handler::resolve_csml_object::{
    exec_closure, insert_args_in_scope_memory, insert_memories_in_scope_memory,
};
use num_traits::ToPrimitive;
use phf::phf_map;
use rand::RngExt;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::any::Any;
use std::cmp::Ordering;
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    array: &mut PrimitiveArray,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    interval: Interval,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo>;

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveArray::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveArray::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveArray::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveArray::type_of as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveArray::get_info as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, interval, _, _, _| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveArray::convert_to_string as PrimitiveMethod, Right::Read),

    "init" => (PrimitiveArray::init as PrimitiveMethod, Right::Read),
    "find" => (PrimitiveArray::find as PrimitiveMethod, Right::Read),
    "is_empty" => (PrimitiveArray::is_empty as PrimitiveMethod, Right::Read),
    "insert_at" => (PrimitiveArray::insert_at as PrimitiveMethod, Right::Write),
    "index_of" => (PrimitiveArray::index_of as PrimitiveMethod, Right::Read),
    "join" => (PrimitiveArray::join as PrimitiveMethod, Right::Read),
    "length" => (PrimitiveArray::length as PrimitiveMethod, Right::Read),
    "one_of" => (PrimitiveArray::one_of as PrimitiveMethod, Right::Read),
    "push" => (PrimitiveArray::push as PrimitiveMethod, Right::Write),
    "pop" => (PrimitiveArray::pop as PrimitiveMethod, Right::Write),
    "remove_at" => (PrimitiveArray::remove_at as PrimitiveMethod, Right::Write),
    "slice" => (PrimitiveArray::slice as PrimitiveMethod, Right::Read),
    "shuffle" => (PrimitiveArray::shuffle as PrimitiveMethod, Right::Write),
    "map" => (PrimitiveArray::map as PrimitiveMethod, Right::Read),
    "filter" => (PrimitiveArray::filter as PrimitiveMethod, Right::Read),
    "reduce" => (PrimitiveArray::reduce as PrimitiveMethod, Right::Read),
    "reverse" => (PrimitiveArray::reverse as PrimitiveMethod, Right::Read),
    "append" => (PrimitiveArray::append as PrimitiveMethod, Right::Read),
    "flatten" => (PrimitiveArray::flatten as PrimitiveMethod, Right::Read),
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveArray {
    pub value: Vec<Literal>,
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn cast_to_index(
    index: i64,
    length: usize,
    data: &Data,
    interval: Interval,
) -> Result<usize, ErrorInfo> {
    let Some(index) = index.to_usize() else {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_ARRAY_BOUNDS.to_owned(),
        ));
    };

    if index > length {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_ARRAY_INDEX.to_owned(),
        ));
    }

    Ok(index)
}

impl PrimitiveArray {
    #[allow(clippy::needless_pass_by_value)]
    fn is_number(
        _array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_number() => boolean")?;

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_int(
        _array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_int() => boolean")?;

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_float(
        _array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_float() => boolean")?;

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "type_of() => string")?;

        Ok(PrimitiveString::get_literal("array", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _array: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        literal::get_info(&args, additional_info, interval, data)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_string(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_string() => string")?;

        Ok(PrimitiveString::get_literal(&array.to_string(), interval))
    }
}

impl PrimitiveArray {
    #[allow(clippy::needless_pass_by_value)]
    fn init(
        _array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [capacity] = get_int_args(
            &args,
            data,
            interval,
            ERROR_ARRAY_INIT,
            "init(capacity: Int) => [Literal]",
        )?;
        let vec = vec![PrimitiveNull::get_literal(interval); capacity];

        Ok(Self::get_literal(vec, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn find(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "find(value: primitive) => array";

        let [value]: [_; 1] = get_literal_args(&args, data, interval, usage, usage)?;

        let vector = array
            .value
            .iter()
            .filter(|&literal| literal == value)
            .cloned()
            .collect();

        Ok(Self::get_literal(vector, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_empty(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_empty() => boolean")?;

        let result = array.value.is_empty();

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn insert_at(
        array: &mut Self,
        mut args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "insert_at(index: int, value: primitive) => null";

        require_n_args(2, &args, interval, data, usage)?;

        let index = get_int_arg(
            arg_name!(0),
            &args,
            interval,
            data,
            ERROR_ARRAY_INSERT_AT_INT,
        )?;

        let Some(value) = args.remove(arg_name!(1)) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        };

        let index = cast_to_index(index, array.value.len(), data, interval)?;

        array.value.insert(index, value);

        Ok(PrimitiveNull::get_literal(interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn index_of(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "index_of(value: primitive) => int";

        let [value]: [_; 1] = get_literal_args(&args, data, interval, usage, usage)?;

        for (index, literal) in array.value.iter().enumerate() {
            if literal == value {
                let index = index.to_i64().ok_or(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    OVERFLOWING_OPERATION.to_owned(),
                ))?;
                return Ok(PrimitiveInt::get_literal(index, interval));
            }
        }

        Ok(PrimitiveInt::get_literal(-1, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn join(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [separator] = get_string_args(
            &args,
            data,
            interval,
            ERROR_ARRAY_JOIN,
            "join(separator: string) => string",
        )?;

        let length = array.value.len();
        let mut result = String::new();

        for (index, string) in array.value.iter().enumerate() {
            result.push_str(&string.primitive.to_string());

            if index + 1 != length {
                result.push_str(separator);
            }
        }

        Ok(PrimitiveString::get_literal(&result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn length(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "length() => int")?;

        let result = array.value.len();

        Ok(PrimitiveInt::get_literal(
            result.to_i64().ok_or_else(|| {
                gen_error_info(
                    Position::new(interval, &data.context.flow),
                    OVERFLOWING_OPERATION.to_owned(),
                )
            })?,
            interval,
        ))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn one_of(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "one_of() => primitive")?;

        if let Some(res) = array
            .value
            .get(rand::rng().random_range(0..array.value.len()))
        {
            return Ok(res.clone());
        }

        Ok(PrimitiveNull::get_literal(interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn push(
        array: &mut Self,
        mut args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "push(value: primitive) => null";

        require_n_args(1, &args, interval, data, usage)?;

        if array.value.len() == usize::MAX {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("{} {}", ERROR_ARRAY_OVERFLOW, usize::MAX,),
            ));
        }

        let Some(value) = args.remove(arg_name!(0)) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        };

        array.value.push(value);

        Ok(PrimitiveNull::get_literal(interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn pop(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "pop() => primitive")?;

        match array.value.pop() {
            Some(literal) => Ok(literal),
            None => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_ARRAY_POP.to_owned(),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn remove_at(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [index] = get_int_args(
            &args,
            data,
            interval,
            ERROR_ARRAY_REMOVE_AT,
            "remove_at(index: int) => primitive",
        )?;

        let index = cast_to_index(index, array.value.len(), data, interval)?;

        Ok(array.value.remove(index))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn shuffle(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "shuffle() => array")?;

        let mut vector = array.value.clone();

        vector.shuffle(&mut rand::rng());

        Ok(Self::get_literal(vector, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn slice(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "slice(start: Integer, end: Optional<Integer>) => [Literal]";

        let (start_index, end_index) =
            get_index_args(array.value.len(), &args, interval, data, usage)?;

        let value = array.value[start_index..end_index].to_vec();

        Ok(Self::get_literal(value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn reverse(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "reverse() => [Literal]")?;

        let reversed_list = array.value.iter().rev().cloned().collect();

        Ok(Self::get_literal(reversed_list, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn append(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(
            1,
            &args,
            interval,
            data,
            "append(other_array: [Literal]) => [Literal]",
        )?;

        let mut other_array: Vec<Literal> = match args.get(arg_name!(0)) {
            Some(res) if res.content_type == "array" => {
                let value = Literal::get_value::<Vec<Literal>, _>(
                    &res.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_ARRAY_INSERT_AT,
                )?;

                (*value).clone().clone()
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_ARRAY_INSERT_AT.to_owned(),
                ));
            }
        };

        let mut new_array = array.value.clone();

        new_array.append(&mut other_array);

        Ok(Self::get_literal(new_array, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn flatten(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "flatten() => [Literal]")?;

        let mut new_array = vec![];

        for lit in &array.value {
            if lit.content_type == "array" {
                let value = Literal::get_value::<Vec<Literal>, _>(
                    &lit.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_ARRAY_INSERT_AT,
                )?;
                for elem in value {
                    new_array.push(elem.clone());
                }
            } else {
                new_array.push(lit.clone());
            }
        }

        Ok(Self::get_literal(new_array, interval))
    }
}

impl PrimitiveArray {
    #[allow(clippy::needless_pass_by_value)]
    fn map(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "map(fn) expect one argument of type [Closure]";

        let [closure]: [&PrimitiveClosure; 1] =
            get_args(&args, data, interval, format!("usage: {usage}"), usage)?;

        let mut vec = vec![];
        let mut context = init_child_context(data);
        let mut step_count = *data.step_count;
        let mut new_scope_data = init_child_scope(data, &mut context, &mut step_count);

        if let Some(memories) = &closure.enclosed_variables {
            insert_memories_in_scope_memory(&mut new_scope_data, memories, msg_data, sender);
        }

        for (index, value) in array.value.iter().enumerate() {
            let mut map = HashMap::new();
            map.insert(arg_name!(0).to_owned(), value.clone());
            if closure.args.len() >= 2 {
                map.insert(
                    arg_name!(1).to_owned(),
                    PrimitiveInt::get_literal(
                        index.to_i64().ok_or_else(|| {
                            gen_error_info(
                                Position::new(interval, &data.context.flow),
                                OVERFLOWING_OPERATION.to_owned(),
                            )
                        })?,
                        interval,
                    ),
                );
            }

            let args = ArgsType::Normal(map);
            insert_args_in_scope_memory(
                &mut new_scope_data,
                &closure.args,
                &args,
                msg_data,
                sender,
            );

            let result = exec_closure(
                &closure.func,
                &closure.args,
                &args,
                interval,
                &mut new_scope_data,
                msg_data,
                sender,
            )?;
            vec.push(result);
        }

        Ok(Self::get_literal(vec, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn filter(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "filter(fn) expect one argument of type [Closure]";

        let [closure]: [&PrimitiveClosure; 1] =
            get_args(&args, data, interval, format!("usage: {usage}"), usage)?;

        let mut vec = vec![];

        let mut context = init_child_context(data);
        let mut step_count = *data.step_count;
        let mut new_scope_data = init_child_scope(data, &mut context, &mut step_count);

        if let Some(memories) = &closure.enclosed_variables {
            insert_memories_in_scope_memory(&mut new_scope_data, memories, msg_data, sender);
        }

        for (index, value) in array.value.iter().enumerate() {
            let mut map = HashMap::new();
            map.insert(arg_name!(0).to_owned(), value.clone());
            if closure.args.len() >= 2 {
                map.insert(
                    arg_name!(1).to_owned(),
                    PrimitiveInt::get_literal(
                        index.to_i64().ok_or_else(|| {
                            gen_error_info(
                                Position::new(interval, &data.context.flow),
                                OVERFLOWING_OPERATION.to_owned(),
                            )
                        })?,
                        interval,
                    ),
                );
            }

            let args = ArgsType::Normal(map);

            insert_args_in_scope_memory(
                &mut new_scope_data,
                &closure.args,
                &args,
                msg_data,
                sender,
            );

            let result = exec_closure(
                &closure.func,
                &closure.args,
                &args,
                interval,
                &mut new_scope_data,
                msg_data,
                sender,
            )?;

            if result.primitive.as_bool() {
                vec.push(value.clone());
            }
        }

        Ok(Self::get_literal(vec, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn reduce(
        array: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "reduce(acc, fn) expect tow arguments an initial value and 'Closure' with two arguments: an 'accumulator', and an element";

        require_n_args(2, &args, interval, data, usage)?;

        match (args.get(arg_name!(0)), args.get(arg_name!(1))) {
            (Some(acc), Some(closure)) => {
                let mut accumulator = acc.clone();

                let closure: &PrimitiveClosure = Literal::get_value::<PrimitiveClosure, _>(
                    &closure.primitive,
                    &data.context.flow,
                    interval,
                    format!("usage: {usage}"),
                )?;

                let mut context = init_child_context(data);
                let mut step_count = *data.step_count;
                let mut new_scope_data = init_child_scope(data, &mut context, &mut step_count);

                for (index, value) in array.value.iter().enumerate() {
                    let mut map = HashMap::new();
                    map.insert(arg_name!(0).to_owned(), accumulator);
                    map.insert(arg_name!(1).to_owned(), value.clone());

                    if closure.args.len() >= 2 {
                        map.insert(
                            "arg2".to_owned(),
                            PrimitiveInt::get_literal(
                                index.to_i64().ok_or_else(|| {
                                    gen_error_info(
                                        Position::new(interval, &data.context.flow),
                                        OVERFLOWING_OPERATION.to_owned(),
                                    )
                                })?,
                                interval,
                            ),
                        );
                    }

                    let args = ArgsType::Normal(map);

                    insert_args_in_scope_memory(
                        &mut new_scope_data,
                        &closure.args,
                        &args,
                        msg_data,
                        sender,
                    );

                    accumulator = exec_closure(
                        &closure.func,
                        &closure.args,
                        &args,
                        interval,
                        &mut new_scope_data,
                        msg_data,
                        sender,
                    )?;
                }

                Ok(accumulator)
            }
            _ => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            )),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveArray {
    #[must_use]
    pub fn new(value: Vec<Literal>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn get_literal(vector: Vec<Literal>, interval: Interval) -> Literal {
        let primitive = Box::new(Self::new(vector));

        Literal {
            content_type: "array".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[typetag::serde]
impl Primitive for PrimitiveArray {
    impl_basic_cmp!();

    illegal_math_ops!();

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn into_value(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self.value)
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveArray
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        let mut vector: Vec<serde_json::Value> = Vec::new();

        for literal in &self.value {
            let value = literal.primitive.to_json();

            if TYPES.contains(&&(*literal.content_type)) {
                vector.push(value);
            } else {
                let mut map = serde_json::Map::new();
                map.insert("content_type".to_owned(), json!(literal.content_type));
                map.insert("content".to_owned(), value);

                vector.push(json!(map));
            }
        }

        serde_json::Value::Array(vector)
    }

    fn format_mem(&self, _content_type: &str, first: bool) -> serde_json::Value {
        let mut vector: Vec<serde_json::Value> = Vec::new();

        for literal in &self.value {
            let content_type = &literal.content_type;
            let value = literal.primitive.format_mem(content_type, first);
            vector.push(value);
        }

        serde_json::Value::Array(vector)
    }

    fn to_string(&self) -> String {
        self.to_json().to_string()
    }

    fn as_bool(&self) -> bool {
        true
    }

    fn get_value(&self) -> &dyn Any {
        &self.value
    }

    fn get_mut_value(&mut self) -> &mut dyn Any {
        &mut self.value
    }

    fn to_msg(&self, content_type: String) -> Message {
        let vec = self.value.iter().fold(vec![], |mut acc, v| {
            acc.push(v.primitive.to_json());
            acc
        });
        Message {
            content_type,
            content: json!(vec),
        }
    }

    fn do_exec(
        &mut self,
        name: &str,
        args: HashMap<String, Literal>,
        mem_type: &MemoryType,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        _content_type: &ContentType,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<(Literal, Right), ErrorInfo> {
        if let Some((f, right)) = FUNCTIONS.get(name) {
            if *mem_type == MemoryType::Constant && *right == Right::Write {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_CONSTANT_MUTABLE_FUNCTION.to_string(),
                ));
            }
            let res = f(
                self,
                args,
                additional_info,
                interval,
                data,
                msg_data,
                sender,
            )?;

            return Ok((res, *right));
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{name}] {ERROR_ARRAY_UNKNOWN_METHOD}"),
        ))
    }
}
