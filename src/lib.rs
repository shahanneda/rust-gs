#[allow(dead_code)]
mod utils;
mod scene;
pub mod loader;

use utils::{compile_shader, link_program, set_panic_hook};
extern crate js_sys;
extern crate ply_rs;
extern crate wasm_bindgen;
extern crate web_sys;
use js_sys::{Float32Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
    Third = 99,
}


#[wasm_bindgen]
pub struct GSBuffer {
    pub width: u8,
    pub height: u8,
    cells: Vec<u8>
}

#[wasm_bindgen]
impl GSBuffer {
    pub fn new() -> GSBuffer {
        let size = 10;
        let cells = vec![0, size];
        return GSBuffer {
            width: 10,
            height: 10,
            cells,
        };
    }

    pub fn k() {
        log!("hello");
    }

    pub fn display(&self) {
        let num = self.cells[0];
        log!("cell 0 is {num}");
    }
}

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = console)]
    // fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_u32(a: u32);

    // // Multiple arguments too!
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_many(a: &str, b: &str);

    #[wasm_bindgen(js_name = test)]
    fn test_js();

    #[wasm_bindgen(js_namespace = WebAssembly, js_name = Memory)]
    pub type Memory;

    #[wasm_bindgen(constructor)]
    fn new(initial_size: u32) -> Memory;
}

// fn bare_bones() {
//     log("testing ");
//     log_u32(42);
//     log_many("Logging", "many values!");
// }

#[wasm_bindgen]
pub fn greet() {
    // log("testing");
    // log_u32(42);
    // log_many("Logging", "many values!");
}

// macro_rules! console_log {
//     // Note that this is using the `log` function imported above during
//     // `bare_bones`
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

// fn using_web_sys() {
//     use web_sys::console;
//     // console::log_1(&"Hello using web-sys".into());
//     console::log_1(&"testing".into());
//     // console_log!("hello {} {}", 1, 2);

//     // let js: JsValue = 4.into();
//     // console::log_2(&"Logging arbitrary values looks like".into(), &js);
// }






async fn test() -> Result<(), JsValue> {
    // print!("testing");
    // log!("hello")
    // // set up a reader, in this case a file.
    // let resp = reqwest::blocking::get("https://httpbin.org/ip").expect("failed 1");

    // ?
    //     .json::<HashMap<String, String>>()?;
    // println!("{resp:#?}");
    // log!("{resp:#?}");

    // let path = "splats/Shahan_03_id01-30000.ply";
    // let mut f = std::fs::File::open(path).expect("failed to open ply file!");

    // // create a parser
    // let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();

    // // use the parser: read the entire file
    // let ply = p.read_ply(&mut f);

    // // make sure it did work
    // assert!(ply.is_ok());
    // let ply = ply.unwrap();

    // // proof that data has been read
    // log!("Ply header: {:#?}", ply.header);
    // log!("Ply data: {:?}", ply.payload);
    return Ok(());
}

fn render_webgl() -> Result<(), JsValue> {
    /*============ Creating a canvas =================*/
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    /*==========Defining and storing the geometry=======*/

    let vertices: [f32; 9] = [
        -0.5, 0.5, 0.0, //
        0.0, 0.5, 0.0, //
        -0.25, 0.25, 0.0, //
    ];
    let vertices_array = {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let location: u32 = vertices.as_ptr() as u32 / 4;
        Float32Array::new(&memory_buffer).subarray(location, location + vertices.len() as u32)
    };

    // Create an empty buffer object to store the vertex buffer
    let vertex_buffer = gl.create_buffer().ok_or("failed to create buffer")?;

    //Bind appropriate array buffer to it
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

    // Pass the vertex data to the buffer
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    // Unbind the buffer
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
    /*=========================Shaders========================*/

    // vertex shader source code
    let vertCode = r#"attribute vec3 coordinates;
void main(void) {
    gl_Position = vec4(coordinates, 1.0);
    gl_PointSize = 5.0;
}
"#;
    // Create a vertex shader object
    let vertShader = compile_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, vertCode)?;

    // fragment shader source code
    let fragCode = r#"void main(void) {
    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
}"#;
    // Create fragment shader object
    let fragShader = compile_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, fragCode)?;
    // Link both programs
    let shaderProgram = link_program(&gl, &vertShader, &fragShader)?;
    // Use the combined shader program object
    gl.use_program(Some(&shaderProgram));

    /*======== Associating shaders to buffer objects ========*/

    // Bind vertex buffer object
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

    // Get the attribute location
    let coord = gl.get_attrib_location(&shaderProgram, "coordinates") as u32;

    // Point an attribute to the currently bound VBO
    gl.vertex_attrib_pointer_with_i32(coord, 3, WebGlRenderingContext::FLOAT, false, 0, 0);

    // Enable the attribute
    gl.enable_vertex_attrib_array(coord);

    /*============= Drawing the primitive ===============*/

    // Clear the canvas
    gl.clear_color(0.5, 0.5, 0.5, 0.9);

    // Enable the depth test
    gl.enable(WebGlRenderingContext::DEPTH_TEST);

    // Clear the color buffer bit
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    // Set the view port
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    // Draw the triangle
    gl.draw_arrays(WebGlRenderingContext::POINTS, 0, 3);
    return Ok(());
}

#[allow(non_snake_case)]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();
    let scene = loader::loader::load_ply().await.expect("something went wrong in loading");

    log!("{}", scene.splats.len());
    log!("{}", scene.splats[0].x);
    log!("{}", scene.splats[1].y);
    log!("{}", scene.splats[2].z);

    // .expect("something went wrong in loading");
    // load_ply().await.expect("something went wrong in loading");
    test_js();
    // test();
    return render_webgl();
    // return Ok(())
}
