#[allow(dead_code)]
mod utils;
mod scene;
mod loader;
mod shader;
mod gui;
mod ply_splat;



use std::cell::RefCell;
use std::num;
// use std::num;
use std::rc::Rc;

use egui::debug_text::print;
use egui::util::undoer::Settings;
// use egui::epaint::Vertex;
// use egui::frame;
use glm::log;
use glm::Mat4;
use scene::Scene;
use utils::set_panic_hook;
extern crate js_sys;
extern crate ply_rs;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate eframe;
extern crate nalgebra_glm as glm;
use js_sys::{Float32Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlInputElement;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;
use web_sys::WebGlProgram;


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

    #[wasm_bindgen(js_name = getWebGLContext)]
    fn getWebGLContext() -> WebGl2RenderingContext;

    #[wasm_bindgen(js_name = test)]
    fn test_js();

    #[wasm_bindgen(js_name = prompt)]
    fn promptJS(str : &str) -> String;

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

static mut slider1: f32 = 0.0;

#[wasm_bindgen]
pub fn set_slider_1(val: f32) {
    // TODO: don't do this
    unsafe{
        slider1 = val;
    }
    log!("testing {}", val);
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


fn create_and_bind_buffer(gl: &WebGl2RenderingContext, vertices_array: Float32Array) -> Result<WebGlBuffer, &'static str> {
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGl2RenderingContext::STATIC_DRAW,
    );
    return Ok(buffer);
}

fn create_attribute_and_get_location(gl: &WebGl2RenderingContext, buffer: WebGlBuffer, program: &WebGlProgram, name: &str, divisor: bool, size: i32) -> u32{
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    let coord = gl.get_attrib_location(&program, name) as u32;
    gl.enable_vertex_attrib_array(coord);
    gl.vertex_attrib_pointer_with_i32(coord, size, WebGl2RenderingContext::FLOAT, false, 0, 0);
    if divisor {
        gl.vertex_attrib_divisor(coord, 1);
    }
    return coord;
}


fn float32_array_from_vec(vec: &[f32]) -> Float32Array {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>().unwrap()
        .buffer();
    let location: u32 = vec.as_ptr() as u32 / 4;
    return Float32Array::new(&memory_buffer).subarray(location, location + vec.len() as u32);
}

fn setup_webgl(gl: &WebGl2RenderingContext, scene : &Scene) -> Result<WebGlProgram, JsValue> {


    /*==========Defining and storing the geometry=======*/

    let vertices: [f32; 3*4] = [
        // 300.0, 500.0, 4.0, //
        // 0.0, 0.0, 4.0, //
        // 100.0, 700.0, 4.0, //
        //
        0.0, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, //
        1.0, 1.0, 0.0, //
        // 1.0, 0.0, 1.0, //
        // 1.0, 1.0, 1.0, //
        //
        // -0.5, 0.5, 4.1, //
        // 0.0, 0.5, 4.1, //
        // -0.25, 0.25, 4.1, //
    ];
    let vertices = vertices.map(|v| v*0.01);

    let splat_centers = scene.splats.iter().map(|s| {
        let x = s.x as f32;
        let y = s.y as f32;
        let z = s.z as f32;
        return [x, y, z];
    }).flatten().collect::<Vec<f32>>();

    // let one = splat_centers[0];
    // let two = splat_centers[0];
    // let three = splat_centers[0];
    // log!("{one} {two} {three}");

    let splat_colors = scene.splats.iter().map(|s| { 
        return [s.r, s.g, s.b];
    }).flatten().collect::<Vec<f32>>();

    let splat_cov3da = scene.splats.iter().map(|s| { 
        return [s.cov3d[0], s.cov3d[1], s.cov3d[2]];
    }).flatten().collect::<Vec<f32>>();
    log!("{splat_cov3da:?}");
    let splat_cov3db = scene.splats.iter().map(|s| { 
        return [s.cov3d[3], s.cov3d[4], s.cov3d[5]];
    }).flatten().collect::<Vec<f32>>();


    // let splat_centers: [f32; 3*4] = [
    //     // 300.0, 500.0, 4.0, //
    //     // 0.0, 0.0, 4.0, //
    //     // 100.0, 700.0, 4.0, //
    //     //
    //     -0.5, -0.5, -0.5, //
    //     0.5, 1.0, 0.5, //
    //     0.0, 2.0, 0.0, //
    //     0.0, 3.0, 0.0, //
    //     // 0.0, 1.0, 0.0, //
    //     // 0.0, 1.0, 0.0, //
    //     //
    //     // -0.5, 0.5, 4.1, //
    //     // 0.0, 0.5, 4.1, //
    //     // -0.25, 0.25, 4.1, //
    // ];

    // let splat_colors: [f32; 3*4] = [
    //     // 300.0, 500.0, 4.0, //
    //     // 0.0, 0.0, 4.0, //
    //     // 100.0, 700.0, 4.0, //
    //     //
    //     1.0, 0.0, 0.0, //
    //     0.0, 1.0, 0.0, //
    //     0.0, 0.0, 1.0, //
    //     1.0, 1.0, 0.0, //
    //     // 0.0, 1.0, 0.0, //
    //     // 0.0, 1.0, 0.0, //
    //     //
    //     // -0.5, 0.5, 4.1, //
    //     // 0.0, 0.5, 4.1, //
    //     // -0.25, 0.25, 4.1, //
    // ];




    // let vertices_array = {
    //     let memory_buffer = wasm_bindgen::memory()
    //         .dyn_into::<WebAssembly::Memory>()?
    //         .buffer();
    //     let location: u32 = vertices.as_ptr() as u32 / 4;
    //     Float32Array::new(&memory_buffer).subarray(location, location + vertices.len() as u32)
    // };

    let splat_center_array = {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let location: u32 = splat_centers.as_ptr() as u32 / 4;
        Float32Array::new(&memory_buffer).subarray(location, location + splat_centers.len() as u32)
    };

    let vertices_array = float32_array_from_vec(&vertices);
    let colors_array = float32_array_from_vec(&splat_colors);
    let cov3da_array = float32_array_from_vec(&splat_cov3da);
    let cov3db_array = float32_array_from_vec(&splat_cov3db);

    // let colors_array = {
    //     let memory_buffer = wasm_bindgen::memory()
    //         .dyn_into::<WebAssembly::Memory>()?
    //         .buffer();
    //     let location: u32 = splat_colors.as_ptr() as u32 / 4;
    //     Float32Array::new(&memory_buffer).subarray(location, location + splat_colors.len() as u32)
    // };
    // let splat_center_array = float32_array_from_vec(splat_centers.to_vec());

    let vertex_buffer = create_and_bind_buffer(&gl, vertices_array).unwrap();
    let color_buffer = create_and_bind_buffer(&gl, colors_array).unwrap();
    let position_offset_buffer = create_and_bind_buffer(&gl, splat_center_array).unwrap();
    let cov3da_buffer = create_and_bind_buffer(&gl, cov3da_array).unwrap();
    let cov3db_buffer = create_and_bind_buffer(&gl, cov3db_array).unwrap();

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

    let shader_program = shader::shader::create_shader_program(&gl).unwrap();
    gl.use_program(Some(&shader_program));

    create_attribute_and_get_location(&gl, vertex_buffer, &shader_program, "v_pos", false, 3);
    create_attribute_and_get_location(&gl, color_buffer, &shader_program, "s_color", true, 3);
    create_attribute_and_get_location(&gl, position_offset_buffer, &shader_program, "s_center", true, 3);
    create_attribute_and_get_location(&gl, cov3da_buffer, &shader_program, "s_cov3da", true, 3);
    create_attribute_and_get_location(&gl, cov3db_buffer, &shader_program, "s_cov3db", true, 3);


    return Ok(shader_program);
}


fn get_slider_value(id: &str) -> f32 {
    let window = window();
    let document = window.document().expect("should have a document on window");
    let element = document.get_element_by_id(id).expect("did not find {id}");
    return element.dyn_into::<HtmlInputElement>().unwrap().value().parse().unwrap();
}

fn draw(gl: &WebGl2RenderingContext, shader_program: &WebGlProgram, canvas: &web_sys::HtmlCanvasElement, frame_num: i32, num_vertices: i32){
    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    let current_amount = (frame_num % 100) as f32/100.0;

    let slider1val = get_slider_value("slider_1");
    let slider2val = get_slider_value("slider_2");
    let slider3val = get_slider_value("slider_3");
    let slider4val = get_slider_value("slider_4");
    let slider5val = get_slider_value("slider_5");

    gl.use_program(Some(&shader_program));

    let mut model: Mat4 = glm::identity();
    let model_scale = 3.0f32;
    model = glm::translate(&model, &glm::vec3(0.0f32, 0.0f32, 0.0f32));
    // model = glm::rotate_y(&model, current_amount*2.0*glm::pi::<f32>());
    model = glm::scale(&model, &glm::vec3(model_scale, model_scale, model_scale));

    let mut camera: Mat4 = glm::identity();
    camera = glm::translate(&camera, &glm::vec3(slider1val, slider2val, slider3val));
    camera = glm::rotate_x(&camera, slider4val*glm::pi::<f32>());
    camera = glm::rotate_y(&camera, -slider5val*glm::pi::<f32>());
    camera = camera.try_inverse().unwrap();

    // camera = glm::translate(&camera, &glm::vec3(0f32, 0f32, 0f32));

    // let mut proj = glm::ortho(0f32, 800f32, 0f32, 1000f32, 0.0f32, 10f32);
    let mut proj = glm::perspective((width as f32)/ (height as f32), 0.78f32, 0.1f32, 100f32);
    // glm::mat4 proj = glm::perspective(glm::radians(45.0f), (float)width/(float)height, 0.1f, 100.0f);


    // proj.fill_with_identity();


    let model_uniform_location = gl.get_uniform_location(&shader_program, "model").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&model_uniform_location), false, model.as_slice());

    let camera_uniform_location = gl.get_uniform_location(&shader_program, "camera").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&camera_uniform_location), false, camera.as_slice());

    let proj_uniform_location = gl.get_uniform_location(&shader_program, "projection").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_uniform_location), false, proj.as_slice());

 
    // let s = promptJS("eh;");
    // log!("s is {}", s);

    // Clear the canvas
    // gl.clear_color(0.5, 0.5, 0.5, 0.9);
    gl.clear_color(0.0, 0.0, 0.0, 1.0);

    // Enable the depth test
    gl.enable(WebGl2RenderingContext::DEPTH_TEST);

    // Clear the color buffer bit
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    // gl.enable(WebGl2RenderingContext::ALIASED_POINT_SIZE_RANGE);

    // Set the view port
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    // Draw the triangle


    // gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    let gaussian_count = num_vertices;

    // gl.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 6);
    // gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, 3);
    gl.draw_arrays_instanced( WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4, gaussian_count);

    // gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, settings.maxGaussians);

    return 
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn get_canvas_context(canvas_id: &str) -> web_sys::CanvasRenderingContext2d {
    let window = window();
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id(canvas_id)
        .expect("should have a canvas on the page")
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

#[allow(non_snake_case)]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();
    let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    let scene = Scene::new(ply_splat);

    // log!("{}", scene.splats.len());
    // log!("{}", scene.splats[0].x);
    // log!("{}", scene.splats[1].y);
    // log!("{}", scene.splats[2].z);

    // .expect("something went wrong in loading");
    // load_ply().await.expect("something went wrong in loading");
    // test_js();
    // test();


    // let web_options = eframe::WebOptions::default();

    // wasm_bindgen_futures::spawn_local(async {
    //     eframe::WebRunner::new()
    //         .start(
    //             "ui", // hardcode it
    //             web_options,
    //             Box::new(|cc| Box::new(gui::gui::TemplateApp::new(cc))),
    //         )
    //         .await
    //         .expect("failed to start eframe");
    // });
    // let uicanvas = get_canvas_context("ui");

    // egui::Window::new("Hello").show(uicanvas, |ui| {
    //     ui.label("world");
    // });

    /*============ Creating a canvas =================*/

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let gl = getWebGLContext();

    // let gl = canvas
    //     .get_context("webgl2")?
    //     .unwrap()
    //     .dyn_into::<WebGl2RenderingContext>()?;
    // log!("context from js is: {context}");

    let program = setup_webgl(&gl, &scene).unwrap();


    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        // if i > 300 {
        //     log!("done");

        //     // Drop our handle to this closure so that it will get cleaned
        //     // up once we return.
        //     let _ = f.borrow_mut().take();
        //     return;
        // }
        draw(&gl, &program, &canvas, i, scene.splats.len() as i32);

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        // let text = format!("requestAnimationFrame has been called {} times.", i);
        // log!("{}", text);
        // body().set_text_content(Some(&text));

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    return Ok(())
}
