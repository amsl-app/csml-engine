use crate::data::position::Position;
use crate::data::primitive::{Primitive, PrimitiveObject, PrimitiveString};
use crate::data::{Data, Interval};
use crate::error_format::{ErrorInfo, gen_error_info};
use std::borrow::Cow;

use bincode::de::Decoder;
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::ops::Add;
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Literal {
    pub content_type: String,
    pub primitive: Box<dyn Primitive>,
    // this adds complementary information about the origin of the variable
    pub additional_info: Option<HashMap<String, Literal>>,
    pub secure_variable: bool,
    pub interval: Interval,
}

impl bincode::Encode for Literal {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        bincode::Encode::encode(&self.content_type, encoder)?;
        let config = *encoder.config();
        bincode::serde::encode_into_writer(&self.primitive, encoder.writer(), config)?;
        bincode::Encode::encode(&self.additional_info, encoder)?;
        bincode::Encode::encode(&self.secure_variable, encoder)?;
        bincode::Encode::encode(&self.interval, encoder)?;
        Ok(())
    }
}

impl<Context> bincode::Decode<Context> for Literal {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let content_type = bincode::Decode::decode(decoder)?;
        let config = *decoder.config();
        let primitive = bincode::serde::decode_from_reader(decoder.reader(), config)?;
        let additional_info = bincode::Decode::decode(decoder)?;
        let secure_variable = bincode::Decode::decode(decoder)?;
        let interval = bincode::Decode::decode(decoder)?;
        Ok(Self {
            content_type,
            primitive,
            additional_info,
            secure_variable,
            interval,
        })
    }
}

impl<'de, Context> bincode::BorrowDecode<'de, Context> for Literal {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let content_type = bincode::BorrowDecode::borrow_decode(decoder)?;
        let config = *decoder.config();
        let primitive = bincode::serde::decode_from_reader(decoder.reader(), config)?;
        let additional_info = bincode::BorrowDecode::borrow_decode(decoder)?;
        let secure_variable = bincode::BorrowDecode::borrow_decode(decoder)?;
        let interval = bincode::BorrowDecode::borrow_decode(decoder)?;
        Ok(Self {
            content_type,
            primitive,
            additional_info,
            secure_variable,
            interval,
        })
    }
}

#[derive(Debug)]
pub enum ContentType {
    Event(String),
    Http,
    Smtp,
    Base64,
    Hex,
    Jwt,
    Crypto,
    Time,
    Primitive,
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::implicit_hasher)]
pub fn get_info<S: BuildHasher>(
    args: &HashMap<String, Literal, S>,
    additional_info: Option<&HashMap<String, Literal>>,
    interval: Interval,
    data: &mut Data,
) -> Result<Literal, ErrorInfo> {
    let usage = "get_info(Optional<String: search_key>) => Literal";

    match (additional_info, args.get("arg0")) {
        (Some(map), None) => Ok(PrimitiveObject::get_literal(map.clone(), interval)),

        (Some(map), Some(key)) => {
            let key = Literal::get_value::<String, _>(
                &key.primitive,
                &data.context.flow,
                interval,
                usage,
            )?;

            if let Some(value) = map.get(key) {
                Ok(value.clone())
            } else {
                let mut lit = PrimitiveObject::get_literal(map.clone(), interval);
                let error_msg = format!("get_info() failed, key '{key}' not found");

                // add an error message to additional info
                lit.add_error_to_info(&error_msg);

                Ok(lit)
            }
        }

        _ => Ok(PrimitiveString::get_literal("Null", interval)),
    }
}

#[must_use]
pub fn create_error_info(error_msg: &str, interval: Interval) -> HashMap<String, Literal> {
    let mut map = HashMap::new();

    map.insert(
        "error".to_owned(),
        PrimitiveString::get_literal(error_msg, interval),
    );

    map
}

////////////////////////////////////////////////////////////////////////////////
// Implementations
////////////////////////////////////////////////////////////////////////////////

impl Literal {
    #[allow(clippy::borrowed_box)]
    pub fn get_value<'lifetime, T: 'static, E: Into<Cow<'static, str>>>(
        primitive: &'lifetime Box<dyn Primitive>,
        flow_name: &str,
        interval: Interval,
        error_message: E,
    ) -> Result<&'lifetime T, ErrorInfo> {
        Self::cast_value(primitive.as_ref()).ok_or_else(|| {
            let error_message = error_message.into();
            gen_error_info(
                Position::new(interval, flow_name),
                String::from(error_message),
            )
        })
    }

    #[must_use]
    pub fn cast_value<T: 'static>(primitive: &dyn Primitive) -> Option<&T> {
        primitive.get_value().downcast_ref::<T>()
    }

    pub fn cast_into_value<T: 'static>(self) -> Result<(Box<T>, Interval), Interval> {
        match self.primitive.into_value().downcast() {
            Ok(primitive) => Ok((primitive, self.interval)),
            Err(_) => Err(self.interval),
        }
    }

    pub fn get_mut_value<'lifetime, T: 'static>(
        primitive: &'lifetime mut Box<dyn Primitive>,
        flow_name: &str,
        interval: Interval,
        error_message: String,
    ) -> Result<&'lifetime mut T, ErrorInfo> {
        match primitive.get_mut_value().downcast_mut::<T>() {
            Some(sep) => Ok(sep),
            None => Err(gen_error_info(
                Position::new(interval, flow_name),
                error_message,
            )),
        }
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        content_type.clone_into(&mut self.content_type);
    }

    pub fn add_info(&mut self, key: &str, value: Self) {
        let map = self.additional_info.get_or_insert_with(HashMap::new);
        map.insert(key.to_owned(), value);
    }

    pub fn add_info_block(&mut self, info: HashMap<String, Self>) {
        let map = self.additional_info.get_or_insert_with(HashMap::new);
        map.extend(info);
    }

    pub fn add_error_to_info(&mut self, error_msg: &str) {
        let map = self.additional_info.get_or_insert_with(HashMap::new);
        map.insert(
            "error".to_owned(),
            PrimitiveString::get_literal(error_msg, self.interval),
        );
    }

    pub fn add_literal_to_info(&mut self, key: String, lit: Self) {
        let map = self.additional_info.get_or_insert_with(HashMap::new);
        map.insert(key, lit);
    }
}

impl ContentType {
    #[must_use]
    pub fn get(literal: &Literal) -> Self {
        match literal.content_type.as_ref() {
            "http" => Self::Http,
            "smtp" => Self::Smtp,
            "base64" => Self::Base64,
            "hex" => Self::Hex,
            "jwt" => Self::Jwt,
            "crypto" => Self::Crypto,
            "time" => Self::Time,
            "event" => Self::Event(String::new()),
            _ => Self::Primitive,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.primitive.partial_cmp(&other.primitive)
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.primitive.is_eq(&(*other.primitive))
    }
}

impl Add for Literal {
    type Output = Result<Box<dyn Primitive + 'static>, String>;

    fn add(self, rhs: Self) -> Result<Box<dyn Primitive + 'static>, String> {
        self.primitive + rhs.primitive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BINCODE_CONFIG;

    #[test]
    fn test_literal_encode_decode() {
        let lit = Literal {
            content_type: "test".to_owned(),
            primitive: Box::new(PrimitiveString::new("testing".to_owned())),
            additional_info: Some(HashMap::from([
                (
                    "foo".to_owned(),
                    Literal {
                        content_type: "bar".to_owned(),
                        primitive: Box::new(PrimitiveString::new("foo bar".to_owned())),
                        additional_info: None,
                        secure_variable: true,
                        interval: Interval::default(),
                    },
                ),
                (
                    "hello".to_owned(),
                    Literal {
                        content_type: "world".to_owned(),
                        primitive: Box::new(PrimitiveString::new("hello world".to_owned())),
                        additional_info: None,
                        secure_variable: false,
                        interval: Interval::default(),
                    },
                ),
            ])),
            secure_variable: false,
            interval: Interval::default(),
        };

        let encoded = bincode::encode_to_vec(&lit, BINCODE_CONFIG).unwrap();
        let decoded: Literal = bincode::decode_from_slice(&encoded, BINCODE_CONFIG)
            .unwrap()
            .0;

        assert_eq!(lit, decoded);
    }
}
