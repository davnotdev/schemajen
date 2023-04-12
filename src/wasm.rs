use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn supported_accumulators() -> JsValue {
    let mut s = String::from("[");
    ACCUMULATOR_SUPPORT_LIST
        .iter()
        .for_each(|v| s += &format!(r#""{}","#, v));
    s.pop();
    s.push(']');
    s.into()
}

#[wasm_bindgen]
pub fn generate_js(accumulator: &str, name: &str, str: &str) -> JsValue {
    let Some(mut accumulator) = accumulator_choose_with_str(accumulator) else {
        return format!("Error: That accumulator is not supported here.").into();
    };
    let res = generate(&mut accumulator, name, str);

    if let Err(e) = res {
        return format!("Error: {:?}", e).into();
    }
    res.unwrap().into()
}
