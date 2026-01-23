use crate::data::primitive::utils::{arg_name, require_n_args};
use crate::data::primitive::{PrimitiveBoolean, PrimitiveType};
use crate::data::{Data, Interval, Literal, Position};
use crate::error_format::{
    ERROR_SLICE_ARG_INT, ERROR_SLICE_ARG_LEN, ERROR_SLICE_ARG2, ErrorInfo, gen_error_info,
};
use num_traits::{NumCast, ToPrimitive};
use std::array;
use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::MaybeUninit;

const ARG_STRINGS: [&str; 8] = [
    arg_name!(0),
    arg_name!(1),
    arg_name!(2),
    arg_name!(3),
    arg_name!(4),
    arg_name!(5),
    arg_name!(6),
    arg_name!(7),
];

fn get_key(index: usize) -> Cow<'static, str> {
    if index < ARG_STRINGS.len() {
        Cow::Borrowed(ARG_STRINGS[index])
    } else {
        Cow::Owned(format!("arg{index}"))
    }
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn is_error(
    additional_info: Option<&HashMap<String, Literal>>,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    if let Some(map) = additional_info
        && map.contains_key("error")
    {
        return Ok(PrimitiveBoolean::get_literal(true, interval));
    }
    Ok(PrimitiveBoolean::get_literal(false, interval))
}

pub fn get_index(len: usize, offset: i64) -> Option<usize> {
    if offset < 0 {
        len.checked_sub(offset.checked_abs()?.to_usize()?)
    } else {
        offset.to_usize()
    }
}

pub(crate) fn get_int_arg<T: NumCast, E: Into<Cow<'static, str>>>(
    arg: &str,
    args: &HashMap<String, Literal>,
    interval: Interval,
    data: &Data,
    error: E,
) -> Result<T, ErrorInfo> {
    let error = error.into();
    if let Some(index) = args.get(arg)
        && index.primitive.get_type() == PrimitiveType::PrimitiveInt
    {
        let int = Literal::cast_value::<i64>(index.primitive.as_ref());
        if let Some(int) = int.copied().and_then(NumCast::from) {
            return Ok(int);
        }
    }
    Err(gen_error_info(
        Position::new(interval, &data.context.flow),
        String::from(error),
    ))
}

pub(crate) fn get_index_args<E: Into<Cow<'static, str>>>(
    len: usize,
    args: &HashMap<String, Literal>,
    interval: Interval,
    data: &mut Data,
    usage: E,
) -> Result<(usize, usize), ErrorInfo> {
    if !(1..=2).contains(&args.len()) {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            usage.into().into_owned(),
        ));
    }

    let gen_error = || {
        gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_SLICE_ARG_INT.to_owned(),
        )
    };

    let get_int = |arg| -> Result<Option<usize>, ErrorInfo> {
        let Some(literal) = args.get(arg) else {
            return Ok(None);
        };
        let int = Literal::get_value::<i64, _>(
            &literal.primitive,
            &data.context.flow,
            literal.interval,
            ERROR_SLICE_ARG_INT,
        )?;

        let index = get_index(len, *int).ok_or_else(gen_error)?;
        Ok(Some(index))
    };

    let start_index = get_int(arg_name!(0))?.ok_or_else(gen_error)?;
    let end_index = get_int(arg_name!(1))?.unwrap_or(len);

    if end_index < start_index {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_SLICE_ARG2.to_string(),
        ));
    }

    if !(start_index < len && end_index <= len) {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_SLICE_ARG_LEN.to_string(),
        ));
    }

    Ok((start_index, end_index))
}

// pub(crate) fn extract_string_args<'a, E: Into<Cow<'static, str>>, const N: usize>(
//     mut args: HashMap<String, Literal>,
//     data: &Data,
//     interval: Interval,
//     error: E,
//     usage: &str,
// ) -> Result<[String; N], ErrorInfo> {
//     require_n_args(N, &args, interval, data, usage)?;
//
//     // SAFETY: Copied from array::try_from_fn
//     let mut res = unsafe { MaybeUninit::<[MaybeUninit<String>; N]>::uninit().assume_init() };
//
//     let error = error.into();
//     for (index, item) in res.iter_mut().enumerate() {
//         let key = get_key(index);
//         if let Some(mut literal) = args.remove(key.as_ref()) {
//             if literal.primitive.get_type() == PrimitiveType::PrimitiveString {
//                 let value = literal.primitive.into_any().downcast::<PrimitiveString>().map_err(|_| {
//                     gen_error_info(Position::new(interval, &data.context.flow), error.to_string())
//                 })?;
//                 item.write(value.value);
//                 continue;
//             }
//         }
//         return Err(gen_error_info(
//             Position::new(interval, &data.context.flow),
//             error.to_string(),
//         ));
//     }
//     // SAFETY: If we get here, the array is initialized
//     Ok(array::from_fn(|i| unsafe {
//         let mut item = MaybeUninit::uninit();
//         ptr::swap(res.get_unchecked_mut(i), &mut item);
//         item.assume_init()
//     }))
// }

pub(crate) fn get_string_args<'a, E: Into<Cow<'static, str>>, const N: usize>(
    args: &'a HashMap<String, Literal>,
    data: &Data,
    interval: Interval,
    error: E,
    usage: &str,
) -> Result<[&'a String; N], ErrorInfo> {
    require_n_args(N, args, interval, data, usage)?;

    // SAFETY: Copied from array::try_from_fn
    let mut res = unsafe { MaybeUninit::<[MaybeUninit<&String>; N]>::uninit().assume_init() };

    let error = error.into();
    for (index, item) in res.iter_mut().enumerate() {
        let key = get_key(index);
        if let Some(literal) = args.get(key.as_ref())
            && literal.primitive.get_type() == PrimitiveType::PrimitiveString
        {
            let value = Literal::get_value::<String, _>(
                &literal.primitive,
                &data.context.flow,
                literal.interval,
                error.clone(),
            )?;
            item.write(value);
            continue;
        }
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            error.to_string(),
        ));
    }
    // SAFETY: If we get here, the array is initialized
    Ok(array::from_fn(|i| unsafe {
        res.get_unchecked(i).assume_init()
    }))
}

pub(crate) fn get_int_args<T: NumCast + Copy, E: Into<Cow<'static, str>>, const N: usize>(
    args: &HashMap<String, Literal>,
    data: &Data,
    interval: Interval,
    error: E,
    usage: &str,
) -> Result<[T; N], ErrorInfo> {
    require_n_args(N, args, interval, data, usage)?;

    // SAFETY: Copied from array::try_from_fn
    let mut res = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

    let error = error.into();
    for (index, item) in res.iter_mut().enumerate() {
        let key = get_key(index);
        get_int_arg(&key, args, interval, data, error.clone()).map(|int| item.write(int))?;
    }
    // SAFETY: If we get here, the array is initialized
    Ok(array::from_fn(|i| unsafe {
        res.get_unchecked(i).assume_init()
    }))
}

pub(crate) fn get_args<'a, T: 'static, E: Into<Cow<'static, str>>, const N: usize>(
    args: &'a HashMap<String, Literal>,
    data: &Data,
    interval: Interval,
    error: E,
    usage: &str,
) -> Result<[&'a T; N], ErrorInfo> {
    require_n_args(N, args, interval, data, usage)?;

    // SAFETY: Copied from array::try_from_fn
    let mut res = unsafe { MaybeUninit::<[MaybeUninit<&T>; N]>::uninit().assume_init() };

    let error = error.into();
    for (index, item) in res.iter_mut().enumerate() {
        let key = get_key(index);
        if let Some(literal) = args.get(key.as_ref()) {
            let value = Literal::get_value::<T, _>(
                &literal.primitive,
                &data.context.flow,
                literal.interval,
                error.clone(),
            )?;
            item.write(value);
            continue;
        }
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            error.to_string(),
        ));
    }
    // SAFETY: If we get here, the array is initialized
    Ok(array::from_fn(|i| unsafe {
        res.get_unchecked(i).assume_init()
    }))
}

pub(crate) fn get_literal_args<'a, E: Into<Cow<'static, str>>, const N: usize>(
    args: &'a HashMap<String, Literal>,
    data: &Data,
    interval: Interval,
    error: E,
    usage: &str,
) -> Result<[&'a Literal; N], ErrorInfo> {
    require_n_args(N, args, interval, data, usage)?;

    // SAFETY: Copied from array::try_from_fn
    let mut res = unsafe { MaybeUninit::<[MaybeUninit<&Literal>; N]>::uninit().assume_init() };

    let error = error.into();
    for (index, item) in res.iter_mut().enumerate() {
        let key = get_key(index);
        if let Some(literal) = args.get(key.as_ref()) {
            item.write(literal);
            continue;
        }
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            error.to_string(),
        ));
    }
    // SAFETY: If we get here, the array is initialized
    Ok(array::from_fn(|i| unsafe {
        res.get_unchecked(i).assume_init()
    }))
}
