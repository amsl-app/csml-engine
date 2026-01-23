use crate::data::{
    Interval, Literal,
    position::Position,
    primitive::{PrimitiveArray, PrimitiveObject, PrimitiveString},
};
use crate::error_format::{ErrorInfo, gen_error_info};

use std::collections::{HashMap, hash_map::Iter};

use crate::data::primitive::PrimitiveType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum ArgsType {
    Named(HashMap<String, Literal>),
    Normal(HashMap<String, Literal>),
}

impl ArgsType {
    #[must_use]
    pub fn args_to_debug(self, interval: Interval) -> Literal {
        match self {
            Self::Named(mut map) | Self::Normal(mut map) => {
                let mut args = Vec::with_capacity(map.len());
                let mut is_secure = false;
                for index in 0..map.len() {
                    let lit = map.remove(&format!("arg{index}")).unwrap();
                    is_secure |= lit.secure_variable;
                    let value =
                        PrimitiveString::new(lit.primitive.to_string()).to_literal(lit.interval);
                    args.push(value);
                }

                let obj = HashMap::from([(
                    "args".to_owned(),
                    PrimitiveArray::get_literal(args, interval),
                )]);

                let mut lit = PrimitiveObject::get_literal(obj, interval);
                lit.secure_variable = is_secure;
                lit.set_content_type("debug");

                lit
            }
        }
    }

    #[must_use]
    pub fn args_to_log(self) -> String {
        const ERROR: &str = "secure variables can not be logged";
        match self {
            Self::Named(mut map) | Self::Normal(mut map) => {
                let size = map.len();
                if size == 0 {
                    return String::new();
                }

                let lit = map.remove("arg0").unwrap();
                if lit.secure_variable {
                    return ERROR.to_string();
                }
                let mut result = lit.primitive.to_string();
                for index in 1..size {
                    let lit = map.remove(&format!("arg{index}")).unwrap();
                    if lit.secure_variable {
                        return ERROR.to_string();
                    }

                    let value = lit.primitive.to_string();
                    result.push_str(", ");
                    result.push_str(&value);
                }

                result
            }
        }
    }

    #[must_use]
    pub fn get(&self, key: &str, index: usize) -> Option<&Literal> {
        match self {
            Self::Named(var) => {
                match (var.get(key), index) {
                    (Some(val), _) => Some(val),
                    // tmp ?
                    (None, 0) => var.get(&format!("arg{index}")),
                    (None, _) => None,
                }
            }
            Self::Normal(var) => var.get(&format!("arg{index}")),
        }
    }

    #[must_use]
    pub fn remove(&mut self, key: &str, index: usize) -> Option<Literal> {
        match self {
            Self::Named(var) => {
                match (var.remove(key), index) {
                    (Some(val), _) => Some(val),
                    // tmp ?
                    (None, 0) => var.remove(&format!("arg{index}")),
                    (None, _) => None,
                }
            }
            Self::Normal(var) => var.remove(&format!("arg{index}")),
        }
    }

    #[must_use]
    pub fn remove_typed(
        &mut self,
        key: &str,
        index: usize,
        expected_type: PrimitiveType,
    ) -> Option<Literal> {
        let entry = self.remove(key, index)?;
        if entry.primitive.get_type() == expected_type {
            Some(entry)
        } else {
            None
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Named(var) | Self::Normal(var) => var.len(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Named(var) | Self::Normal(var) => var.is_empty(),
        }
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, String, Literal> {
        match self {
            Self::Named(var) | Self::Normal(var) => var.iter(),
        }
    }

    pub fn populate(
        &self,
        map: &mut HashMap<String, Literal>,
        vec: &[&str],
        flow_name: &str,
        interval: Interval,
    ) -> Result<(), ErrorInfo> {
        match self {
            Self::Named(var) => {
                for (key, value) in var {
                    if !vec.contains(&(key as &str)) && key != "arg0" {
                        map.insert(key.clone(), value.clone());
                    }
                }
            }
            Self::Normal(var) => {
                if vec.len() < var.len() {
                    //TODO:: error msg
                    return Err(gen_error_info(
                        Position::new(interval, flow_name),
                        "to many arguments".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn populate_json_to_literal(
        &self,
        map: &mut HashMap<String, Literal>,
        vec: &[serde_json::Value],
        flow_name: &str,
        interval: Interval,
    ) -> Result<(), ErrorInfo> {
        match self {
            Self::Named(var) => {
                for (key, value) in var {
                    let contains = vec.iter().find(|obj| {
                        if let Some(map) = obj.as_object() {
                            map.contains_key(key)
                        } else {
                            false
                        }
                    });

                    if let (None, true) = (contains, key != "arg0") {
                        map.insert(key.clone(), value.clone());
                    }
                }
            }
            Self::Normal(var) => {
                if vec.len() < var.len() {
                    return Err(gen_error_info(
                        Position::new(interval, flow_name),
                        "to many arguments".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a ArgsType {
    type Item = (&'a String, &'a Literal);
    type IntoIter = Iter<'a, String, Literal>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        let map = HashMap::from([
            (
                "arg0".to_owned(),
                PrimitiveString::get_literal("test", Interval::default()),
            ),
            (
                "arg1".to_owned(),
                PrimitiveString::get_literal("test2", Interval::default()),
            ),
        ]);
        let args = ArgsType::Named(map);

        let result = args.args_to_debug(Interval::default());
        let obj = HashMap::from([(
            "args".to_owned(),
            PrimitiveArray::get_literal(
                vec![
                    PrimitiveString::get_literal("test", Interval::default()),
                    PrimitiveString::get_literal("test2", Interval::default()),
                ],
                Interval::default(),
            ),
        )]);
        let expected = PrimitiveObject::get_literal(obj, Interval::default());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_log() {
        let mut map = HashMap::new();
        map.insert(
            "arg0".to_owned(),
            PrimitiveString::get_literal("test", Interval::default()),
        );
        map.insert(
            "arg1".to_owned(),
            PrimitiveString::get_literal("test2", Interval::default()),
        );
        let args = ArgsType::Named(map);

        let result = args.args_to_log();
        let expected = "test, test2".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_log_secure() {
        let mut map = HashMap::new();
        map.insert(
            "arg0".to_owned(),
            PrimitiveString::get_literal("insecure", Interval::default()),
        );
        let mut secure = PrimitiveString::get_literal("secure", Interval::default());
        secure.secure_variable = true;
        map.insert("arg1".to_owned(), secure);
        let args = ArgsType::Named(map);

        let result = args.args_to_log();
        let expected = "secure variables can not be logged".to_string();
        assert_eq!(result, expected);
    }
}
