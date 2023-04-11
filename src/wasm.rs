use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn supported_accumulators() -> JsValue {
    r#"[
        "mock"
    ]"#.into()
}

#[wasm_bindgen]
pub fn generate_js(accumulator: &str, name: &str, str: &str) -> JsValue {
    let res = generate(
        match accumulator {
            "mock" => MockAccumulator::begin(),
            _ => unimplemented!()
        },
        name,
        str,
    );

    if let Err(e) = res {
        return format!("Got error: {:?}", e).into();
    }
    res.unwrap().into()
}
