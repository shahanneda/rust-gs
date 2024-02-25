mod utils;

use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn run() {
    bare_bones();
    // using_a_macro();
    using_web_sys();
    set_panic_hook();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

fn bare_bones() {
    log("testing ");
    log_u32(42);
    log_many("Logging", "many values!");
}

#[wasm_bindgen]
pub fn greet() {
    // log("testing");
    // log_u32(42);
    // log_many("Logging", "many values!");
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn using_web_sys() {
    use web_sys::console;
    // console::log_1(&"Hello using web-sys".into());
    console::log_1(&"testing".into());
    console_log!("hello {} {}", 1, 2);

    // let js: JsValue = 4.into();
    // console::log_2(&"Logging arbitrary values looks like".into(), &js);
}
