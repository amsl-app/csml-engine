use crate::data::{Literal, ast::Interval, position::Position, primitive::PrimitiveString};
use crate::error_format::{ERROR_JWT_ALGO, ErrorInfo, gen_error_info};
use crate::interpreter::json_to_literal;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use super::PrimitiveObject;

fn jwt_algorithm_to_str(algo: jsonwebtoken::Algorithm) -> &'static str {
    match algo {
        jsonwebtoken::Algorithm::HS256 => "HS256",
        jsonwebtoken::Algorithm::HS384 => "HS384",
        jsonwebtoken::Algorithm::HS512 => "HS512",

        jsonwebtoken::Algorithm::ES256 => "ES256",
        jsonwebtoken::Algorithm::ES384 => "ES384",

        jsonwebtoken::Algorithm::RS256 => "RS256",
        jsonwebtoken::Algorithm::RS384 => "RS384",
        jsonwebtoken::Algorithm::RS512 => "RS512",

        jsonwebtoken::Algorithm::PS256 => "PS256",
        jsonwebtoken::Algorithm::PS384 => "PS384",
        jsonwebtoken::Algorithm::PS512 => "PS512",

        jsonwebtoken::Algorithm::EdDSA => "EdDSA",
    }
}

fn header_to_literal(header: &jsonwebtoken::Header, interval: Interval) -> Literal {
    let mut map = HashMap::new();

    if let Some(typ) = &header.typ {
        map.insert(
            "typ".to_owned(),
            PrimitiveString::get_literal(typ, interval),
        );
    }
    map.insert(
        "alg".to_owned(),
        PrimitiveString::get_literal(jwt_algorithm_to_str(header.alg), interval),
    );
    if let Some(cty) = &header.cty {
        map.insert(
            "cty".to_owned(),
            PrimitiveString::get_literal(cty, interval),
        );
    }
    if let Some(jku) = &header.jku {
        map.insert(
            "jku".to_owned(),
            PrimitiveString::get_literal(jku, interval),
        );
    }
    if let Some(kid) = &header.kid {
        map.insert(
            "kid".to_owned(),
            PrimitiveString::get_literal(kid, interval),
        );
    }
    if let Some(x5u) = &header.x5u {
        map.insert(
            "x5u".to_owned(),
            PrimitiveString::get_literal(x5u, interval),
        );
    }
    if let Some(x5t) = &header.x5t {
        map.insert(
            "x5t".to_owned(),
            PrimitiveString::get_literal(x5t, interval),
        );
    }
    PrimitiveObject::get_literal(map, interval)
}

pub fn token_data_to_literal(
    data: &jsonwebtoken::TokenData<serde_json::Value>,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let mut map = HashMap::new();

    let headers = header_to_literal(&data.header, interval);
    map.insert("header".to_owned(), headers);

    let claims = json_to_literal(&data.claims, interval, flow_name)?;
    map.insert("payload".to_owned(), claims);

    Ok(PrimitiveObject::get_literal(map, interval))
}

pub fn get_algorithm(
    lit: &Literal,
    flow_name: &str,
    interval: Interval,
) -> Result<jsonwebtoken::Algorithm, ErrorInfo> {
    let algo =
        Literal::get_value::<String, _>(&lit.primitive, flow_name, interval, ERROR_JWT_ALGO)?;

    match jsonwebtoken::Algorithm::from_str(algo) {
        Ok(algorithm)
            if algorithm == jsonwebtoken::Algorithm::HS256
                || algorithm == jsonwebtoken::Algorithm::HS384
                || algorithm == jsonwebtoken::Algorithm::HS512 =>
        {
            Ok(algorithm)
        }
        _ => Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_JWT_ALGO.to_string(),
        )),
    }
}

pub fn get_headers(
    lit: &Literal,
    flow_name: &str,
    interval: Interval,
    headers: &mut jsonwebtoken::Header,
) -> Result<(), ErrorInfo> {
    let map = Literal::get_value::<HashMap<String, Literal>, _>(
        &lit.primitive,
        flow_name,
        interval,
        "JWT Headers wrong format",
    )?;
    for (key, value) in map {
        match key.as_ref() {
            "typ" => {
                headers.typ = Some(
                    Literal::get_value::<String, _>(
                        &value.primitive,
                        flow_name,
                        interval,
                        "JWT Headers 'typ' must be of type String",
                    )?
                    .clone(),
                );
            }
            "cty" => {
                headers.cty = Some(
                    Literal::get_value::<String, _>(
                        &lit.primitive,
                        flow_name,
                        interval,
                        "JWT Headers 'cty' must be of type String",
                    )?
                    .clone(),
                );
            }
            "jku" => {
                headers.jku = Some(
                    Literal::get_value::<String, _>(
                        &lit.primitive,
                        flow_name,
                        interval,
                        "JWT Headers 'jku' must be of type String",
                    )?
                    .clone(),
                );
            }
            "kid" => {
                headers.kid = Some(
                    Literal::get_value::<String, _>(
                        &lit.primitive,
                        flow_name,
                        interval,
                        "JWT Headers 'kid' must be of type String",
                    )?
                    .clone(),
                );
            }
            "x5u" => {
                headers.x5u = Some(
                    Literal::get_value::<String, _>(
                        &lit.primitive,
                        flow_name,
                        interval,
                        "JWT Headers 'x5u' must be of type String",
                    )?
                    .clone(),
                );
            }
            "x5t" => {
                headers.x5t = Some(
                    Literal::get_value::<String, _>(
                        &lit.primitive,
                        flow_name,
                        interval,
                        "JWT Headers 'x5t' must be of type String",
                    )?
                    .clone(),
                );
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn get_validation(
    lit: &Literal,
    flow_name: &str,
    interval: Interval,
    validation: &mut jsonwebtoken::Validation,
) -> Result<(), ErrorInfo> {
    let map = Literal::get_value::<HashMap<String, Literal>, _>(
        &lit.primitive,
        flow_name,
        interval,
        "JWT Headers wrong format",
    )?;
    for (key, value) in map {
        match key.as_ref() {
            "leeway" => {
                Literal::get_value::<u64, _>(
                    &value.primitive,
                    flow_name,
                    interval,
                    "JWT Validation 'leeway' must be of type Int",
                )?
                .clone_into(&mut validation.leeway);
            }
            "validate_exp" => {
                Literal::get_value::<bool, _>(
                    &value.primitive,
                    flow_name,
                    interval,
                    "JWT Validation 'validate_exp' must be of type Boolean",
                )?
                .clone_into(&mut validation.validate_exp);
            }
            "validate_nbf" => {
                Literal::get_value::<bool, _>(
                    &value.primitive,
                    flow_name,
                    interval,
                    "JWT Validation 'validate_nbf' must be of type Boolean",
                )?
                .clone_into(&mut validation.validate_nbf);
            }
            "aud" => {
                let vec = Literal::get_value::<Vec<String>, _>(
                    &value.primitive,
                    flow_name,
                    interval,
                    "JWT Validation 'aud' must be of type Boolean",
                )?;

                validation.aud = Some(vec.iter().cloned().collect());
            }
            "iss" => {
                let mut iss = HashSet::new();
                let iss_value = Literal::get_value::<String, _>(
                    &value.primitive,
                    flow_name,
                    interval,
                    "JWT Validation 'validate_nbf' must be of type Boolean",
                )?
                .clone();

                iss.insert(iss_value);

                validation.iss = Some(iss);
            }
            "sub" => {
                validation.sub = Some(
                    Literal::get_value::<String, _>(
                        &value.primitive,
                        flow_name,
                        interval,
                        "JWT Validation 'validate_nbf' must be of type Boolean",
                    )?
                    .clone(),
                );
            }
            _ => {}
        }
    }

    Ok(())
}
