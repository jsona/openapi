use jsona::{dom::Node, util::mapper::Mapper};
use jsona_openapi::Openapi;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, JsValue> {
    let mapper = Mapper::new_utf16(input, false);
    let node = Node::from_str(input)
        .map_err(|err| JsValue::from_serde(&err.to_error_objects(&mapper)).unwrap())?;
    let openapi = Openapi::try_from(&node).map_err(|errs| {
        let error_objects: Vec<_> = errs
            .iter()
            .map(|v| v.to_error_object(&node, &mapper))
            .collect::<Vec<_>>();
        JsValue::from_serde(&error_objects).unwrap()
    })?;
    Ok(JsValue::from_serde(&openapi).unwrap())
}
