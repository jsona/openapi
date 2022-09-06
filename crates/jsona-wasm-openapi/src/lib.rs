use jsona::{dom::Node, error::ErrorObject, util::mapper::Mapper};
use jsona_openapi::Openapi;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ParseResult {
    value: Option<Openapi>,
    errors: Option<Vec<ErrorObject>>,
}

#[wasm_bindgen]
pub fn parse(input: &str) -> JsValue {
    let mapper = Mapper::new_utf16(input, false);
    let node = match Node::from_str(input) {
        Ok(v) => v,
        Err(err) => {
            return JsValue::from_serde(&ParseResult {
                value: None,
                errors: Some(err.to_error_objects(&mapper)),
            }).unwrap()
        }
    };
    let result = match Openapi::try_from(&node) {
        Ok(v) => ParseResult {
            value: Some(v),
            errors: None,
        },
        Err(errs) => ParseResult {
            value: None,
            errors: Some(
                errs.iter()
                    .map(|v| v.to_error_object(&node, &mapper))
                    .collect::<Vec<_>>(),
            ),
        },
    };
	JsValue::from_serde(&result).unwrap()
}
