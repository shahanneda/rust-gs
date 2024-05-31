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
use glm::Vec2;
use glm::Vec3;
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
use web_sys::Event;
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
    }

    pub fn display(&self) {
        let num = self.cells[0];
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
}

fn update_buffer_data(gl: &WebGl2RenderingContext, buffer: &WebGlBuffer, data: Float32Array) {
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &data,
        WebGl2RenderingContext::STATIC_DRAW,
    );
}

fn create_buffer(gl: &WebGl2RenderingContext) -> Result<WebGlBuffer, &'static str> {
    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    return Ok(buffer);
}


fn create_attribute_and_get_location(gl: &WebGl2RenderingContext, buffer: &WebGlBuffer, program: &WebGlProgram, name: &str, divisor: bool, size: i32) -> u32{
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

struct WebGLSetupResult {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    color_buffer: WebGlBuffer,
    position_offset_buffer: WebGlBuffer,
    cov3da_buffer: WebGlBuffer,
    cov3db_buffer: WebGlBuffer,
    opacity_buffer: WebGlBuffer
}

fn update_webgl_buffers(scene: &Scene, webgl: &WebGLSetupResult){
    let mut splat_centers = Vec::new();
    let mut splat_colors = Vec::new();
    let mut splat_cov3da = Vec::new();
    let mut splat_cov3db = Vec::new();
    let mut splat_opacities = Vec::new();


    for s in &scene.splats {
        splat_centers.extend_from_slice(&[s.x, s.y, s.z]);
        splat_colors.extend_from_slice(&[s.r, s.g, s.b]);
        splat_cov3da.extend_from_slice(&[s.cov3d[0], s.cov3d[1], s.cov3d[2]]);
        splat_cov3db.extend_from_slice(&[s.cov3d[3], s.cov3d[4], s.cov3d[5]]);
        splat_opacities.push(s.opacity);
    }

    update_buffer_data(&webgl.gl, &webgl.vertex_buffer, float32_array_from_vec(&splat_centers));
    update_buffer_data(&webgl.gl, &webgl.color_buffer, float32_array_from_vec(&splat_colors));
    update_buffer_data(&webgl.gl, &webgl.position_offset_buffer, float32_array_from_vec(&splat_centers));
    update_buffer_data(&webgl.gl, &webgl.cov3da_buffer, float32_array_from_vec(&splat_cov3da));
    update_buffer_data(&webgl.gl, &webgl.cov3db_buffer, float32_array_from_vec(&splat_cov3db));
    update_buffer_data(&webgl.gl, &webgl.opacity_buffer, float32_array_from_vec(&splat_opacities));
}


fn setup_webgl(gl: WebGl2RenderingContext, scene : &Scene) -> Result<WebGLSetupResult, JsValue> {

    /*==========Defining and storing the geometry=======*/
    let vertices: [f32; 3*4] = [
        // 300.0, 500.0, 4.0, //
        // 0.0, 0.0, 4.0, //
        // 100.0, 700.0, 4.0, //
        //
        -1.0, -1.0, 0.0, //
        1.0, -1.0, 0.0, //
        -1.0, 1.0, 0.0, //
        1.0, 1.0, 0.0, //
        // 1.0, 0.0, 1.0, //
        // 1.0, 1.0, 1.0, //
        //
        // -0.5, 0.5, 4.1, //
        // 0.0, 0.5, 4.1, //
        // -0.25, 0.25, 4.1, //
    ];
    let vertices = vertices.map(|v| v);


    // let one = splat_centers[0];
    // let two = splat_centers[0];
    // let three = splat_centers[0];
    // log!("{one} {two} {three}");
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




    let vertex_buffer = create_buffer(&gl).unwrap();
    let color_buffer = create_buffer(&gl).unwrap();
    let position_offset_buffer = create_buffer(&gl).unwrap();
    let cov3da_buffer = create_buffer(&gl).unwrap();
    let cov3db_buffer = create_buffer(&gl).unwrap();
    let opacity_buffer = create_buffer(&gl).unwrap();

    let shader_program = shader::shader::create_shader_program(&gl).unwrap();
    gl.use_program(Some(&shader_program));

    let result  = WebGLSetupResult{
        gl: gl,
        program: shader_program,
        vertex_buffer,
        color_buffer,
        position_offset_buffer,
        cov3da_buffer,
        cov3db_buffer,
        opacity_buffer
    };

    update_webgl_buffers(scene, &result);


    create_attribute_and_get_location(&result.gl, &result.vertex_buffer, &result.program, "v_pos", false, 3);
    create_attribute_and_get_location(&result.gl, &result.color_buffer, &result.program, "s_color", true, 3);
    create_attribute_and_get_location(&result.gl, &result.position_offset_buffer, &result.program, "s_center", true, 3);
    create_attribute_and_get_location(&result.gl, &result.cov3da_buffer, &result.program, "s_cov3da", true, 3);
    create_attribute_and_get_location(&result.gl, &result.cov3db_buffer, &result.program, "s_cov3db", true, 3);
    create_attribute_and_get_location(&result.gl, &result.opacity_buffer, &result.program, "s_opacity", true, 1);

    return Ok(result);

}


fn get_slider_value(id: &str) -> f32 {
    let window = window();
    let document = window.document().expect("should have a document on window");
    let element = document.get_element_by_id(id).expect("did not find {id}");
    return element.dyn_into::<HtmlInputElement>().unwrap().value().parse().unwrap();
}

fn set_float_uniform_value(shader_program: &WebGlProgram, gl: &WebGl2RenderingContext, name: &str, value: f32){ 
    // log!("name: {}", name);
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform1f(Some(&uniform_location), value);
}

fn set_vec3_uniform_value(shader_program: &WebGlProgram, gl: &WebGl2RenderingContext, name: &str, value: [f32; 3]){ 
    // log!("name: {}", name);
    let uniform_location = gl.get_uniform_location(&shader_program, name).unwrap();
    gl.uniform3fv_with_f32_array(Some(&uniform_location), value.as_slice());
}

// const invertRow = (mat, row) => {
//   mat[row + 0] = -mat[row + 0];
//   mat[row + 4] = -mat[row + 4];
//   mat[row + 8] = -mat[row + 8];
//   mat[row + 12] = -mat[row + 12];
// };
fn invertRow(mat: &mut glm::Mat4, row: usize){
  mat[row + 0] = -mat[row + 0];
  mat[row + 4] = -mat[row + 4];
  mat[row + 8] = -mat[row + 8];
  mat[row + 12] = -mat[row + 12];
}


fn draw(gl: &WebGl2RenderingContext, shader_program: &WebGlProgram, canvas: &web_sys::HtmlCanvasElement, frame_num: i32, num_vertices: i32, vpm: glm::Mat4, vm: glm::Mat4){
    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    let current_amount = (frame_num % 100) as f32/100.0;

    let slider1val = get_slider_value("slider_1");
    let slider2val = get_slider_value("slider_2");
    let slider3val = get_slider_value("slider_3");
    let slider4val = get_slider_value("slider_4");
    let slider5val = get_slider_value("slider_5");


    // let mut model: Mat4 = glm::identity();
    // let model_scale = 3.0f32;
    // model = glm::translate(&model, &glm::vec3(0.0f32, 0.0f32, 0.0f32));
    // // model = glm::rotate_y(&model, current_amount*2.0*glm::pi::<f32>());
    // model = glm::scale(&model, &glm::vec3(model_scale, model_scale, model_scale));
    // camera = glm::translate(&camera, &glm::vec3(0f32, 0f32, 0f32));
    // let mut proj = glm::ortho(0f32, 800f32, 0f32, 1000f32, 0.0f32, 10f32);
    // glm::mat4 proj = glm::perspective(glm::radians(45.0f), (float)width/(float)height, 0.1f, 100.0f);
    // proj.fill_with_identity();


    // let model_uniform_location = gl.get_uniform_location(&shader_program, "model").unwrap();
    // gl.uniform_matrix4fv_with_f32_array(Some(&model_uniform_location), false, model.as_slice());
    gl.use_program(Some(&shader_program));




    let proj_uniform_location = gl.get_uniform_location(&shader_program, "projection").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&proj_uniform_location), false, vpm.as_slice());

    let camera_uniform_location = gl.get_uniform_location(&shader_program, "camera").unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&camera_uniform_location), false, vm.as_slice());

    // gl.uniform1f(gl.getUniformLocation(program, "W"), W);
    // gl.uniform1f(gl.getUniformLocation(program, "H"), H);
    // gl.uniform1f(gl.getUniformLocation(program, "focal_x"), focal_x);
    // gl.uniform1f(gl.getUniformLocation(program, "focal_y"), focal_y);
    // gl.uniform1f(gl.getUniformLocation(program, "tan_fovx"), tan_fovx);
    // gl.uniform1f(gl.getUniformLocation(program, "tan_fovy"), tan_fovy);
    // gl.uniform1f(
    //     gl.getUniformLocation(program, "scale_modifier"),
    //     settings.scalingModifier
    // );
    // gl.uniform3fv(gl.getUniformLocation(program, "boxmin"), sceneMin);
    // gl.uniform3fv(gl.getUniformLocation(program, "boxmax"), sceneMax);
    let width = width as f32;
    let height = height as f32;
    let tan_fovy = f32::tan(0.820176 * 0.5);
    let tan_fovx = (tan_fovy * width) / height;
    let focal_y = height / (2.0 * tan_fovy);
    let focal_x = width / (2.0 * tan_fovx);
    set_float_uniform_value(shader_program, gl, "W", width as f32);
    set_float_uniform_value(shader_program, gl, "H", height as f32);
    set_float_uniform_value(shader_program, gl, "focal_x", focal_x);
    set_float_uniform_value(shader_program, gl, "focal_y", focal_y);
    set_float_uniform_value(shader_program, gl, "tan_fovx", tan_fovx);
    set_float_uniform_value(shader_program, gl, "tan_fovy", tan_fovy);
    // set_float_uniform_value(shader_program, gl, "scale_modifier", 1.0);

    // TODO: edit these
    // set_vec3_uniform_value(shader_program, gl, "boxmin", [-1.0, -1.0, -1.0]);
    // set_vec3_uniform_value(shader_program, gl, "boxmax", [1.0, 1.0, 1.0]);



 
    // let s = promptJS("eh;");
    // log!("s is {}", s);

    // Clear the canvas
    // gl.clear_color(0.5, 0.5, 0.5, 0.9);
    // gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_color(0.0, 0.0, 0.0, 0.0);

    // Set the view port
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    
    // Enable the depth test
    // gl.enable(WebGl2RenderingContext::DEPTH_TEST);

    // Clear the color buffer bit
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    // gl.clear_color(0.0, 0.0, 0.0, 1.0);

    gl.disable(WebGl2RenderingContext::DEPTH_TEST);
	gl.enable(WebGl2RenderingContext::BLEND);
	// gl.blend_func(WebGl2RenderingContext::ONE_MINUS_CONSTANT_ALPHA, WebGl2RenderingContext::ONE);
	gl.blend_func(WebGl2RenderingContext::ONE_MINUS_DST_ALPHA, WebGl2RenderingContext::ONE);

    // gl.enable(WebGl2RenderingContext::ALIASED_POINT_SIZE_RANGE);


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

pub fn get_scene_ready_for_draw(width: i32, height: i32, scene: &mut Scene, cam: &CameraInfo) -> (Mat4, Mat4){
    let mut proj = glm::perspective((width as f32)/ (height as f32), 0.820176f32, 0.1f32, 100f32);
    let mut camera: Mat4 = glm::identity();
    camera = glm::translate(&camera, &cam.pos);
    camera = glm::rotate_x(&camera, cam.rot.x);
    camera = glm::rotate_y(&camera, cam.rot.y);
    camera = camera.try_inverse().unwrap();

    let mut vm = camera;
    let mut vpm = proj * camera;
    invertRow(&mut vm, 1);
    invertRow(&mut vm, 2);
    invertRow(&mut vpm, 1);
    invertRow(&mut vm, 0);
    invertRow(&mut vpm, 0);
    scene.sort_splats_based_on_depth(vpm);
    return (vm, vpm)
}

pub struct CameraInfo{
    pub pos: Vec3,
    pub rot: Vec2, 
}

#[allow(non_snake_case)]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();
    let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    log!("Done loading!");
    let mut scene = Scene::new(ply_splat);

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    let gl = getWebGLContext();


    let webgl = setup_webgl(gl, &scene).unwrap();
    let cam_info = Rc::new(RefCell::new(CameraInfo {
        pos: Vec3::new(0.0, 0.0, 5.0),
        rot: Vec2::new(0.0, 0.0),
    }));

    let cam_info_clone = cam_info.clone();
    let cb = Closure::wrap(Box::new(move |e: Event| {
        // let input = e
        //     .current_target()
        //     .unwrap()
        //     .dyn_into::<web_sys::Document>()
        //     .unwrap();
        
        let keyboard_event = e.clone()
                        .dyn_into::<web_sys::KeyboardEvent>()
                        .unwrap();
            
        log!("Got key down!!! {}", keyboard_event.key());
        let mut cam_info = cam_info_clone.borrow_mut();
        let rot_speed = 0.01;
        let move_speed = 0.1;
        if keyboard_event.key() == "w" {
            cam_info.pos.z += move_speed;
        } else if keyboard_event.key() == "s" {
            cam_info.pos.z -= move_speed;
        }
        if keyboard_event.key() == "a" {
            cam_info.pos.x -= move_speed;
        } else if keyboard_event.key() == "d" {
            cam_info.pos.x += move_speed;
        }
        if keyboard_event.key() == "ArrowUp" {
            cam_info.rot.x += rot_speed;
        } else if keyboard_event.key() == "ArrowDown" {
            cam_info.rot.x -= rot_speed;
        }
        if keyboard_event.key() == "ArrowLeft" {
            cam_info.rot.y += rot_speed;
        } else if keyboard_event.key() == "ArrowRight" {
            cam_info.rot.y -= rot_speed;
        }

    }) as Box<dyn FnMut(_)>);
    // canvas.add_event_listener_with_callback("onkeydown", &cb.as_ref().unchecked_ref())?;
    let _ = document.add_event_listener_with_callback("keydown", &cb.as_ref().unchecked_ref());

    cb.forget();



    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;


    let cam_info_clone2 = cam_info.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        let (vm, vpm) = get_scene_ready_for_draw(width, height, &mut scene, &cam_info_clone2.borrow());
        update_webgl_buffers(&scene, &webgl);
        draw(&webgl.gl, &webgl.program, &canvas, i, scene.splats.len() as i32, vpm, vm);

        i += 1;
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    return Ok(())
}


pub struct AppState{
    scene: Scene,
    webgl: WebGLSetupResult,
    canvas: HtmlCanvasElement,
    width: i32,
    height: i32,
    i: i32,
}

// #[wasm_bindgen]
// impl AppState {
//     #[wasm_bindgen(constructor)]
//     pub async fn new() -> Result<Rc<RefCell<AppState>>, JsValue> {
//         let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
//         set_panic_hook(); 
//         let mut scene = Scene::new(ply_splat);

//         let document = web_sys::window().unwrap().document().unwrap();
//         let canvas = document.get_element_by_id("canvas").unwrap();
//         let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

//         let width = canvas.width() as i32;
//         let height = canvas.height() as i32;
//         let gl = getWebGLContext();


//         let webgl = setup_webgl(gl, &scene).unwrap();

//         Ok(Rc::new(RefCell::new(AppState {
//             scene,
//             webgl,
//             canvas,
//             width,
//             height,
//             i: 0,
//         })))
//     }

//     // pub fn handle_keyboard(&mut self, key: &str) {
//     //     log!("Key pressed: {}", key);

//     //     // Update the scene's view matrix or any other state based on the key input
//     //     match key {
//     //         "ArrowUp" => {
//     //             // Example: Adjust the view matrix for upward arrow key
//     //             self.scene.view_matrix[1][3] += 0.1;
//     //         }
//     //         "ArrowDown" => {
//     //             // Example: Adjust the view matrix for downward arrow key
//     //             self.scene.view_matrix[1][3] -= 0.1;
//     //         }
//     //         // Add more key handling as needed
//     //         _ => {}
//     //     }
//     // }

//     // pub fn render_frame(&mut self) {
//     //     let (vm, vpm) = get_scene_ready_for_draw(self.width, self.height, &mut self.scene);
//     //     update_webgl_buffers(&self.scene, &self.webgl);
//     //     draw(&self.webgl.gl, &self.webgl.program, &self.canvas, self.i, self.scene.splats.len() as i32, vpm, vm);
//     //     self.i += 1;
//     // }
// }

// #[wasm_bindgen(start)]
// pub async fn start() -> Result<Rc<RefCell<AppState>>, JsValue> {
//     set_panic_hook();
//     AppState::new().await
// }