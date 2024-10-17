use eframe::Renderer;
use js_sys::Int32Array;
use js_sys::Uint32Array;
use nalgebra_glm::pi;
use serde::{Serialize, Deserialize};
use web_sys::Worker;
use web_sys::WorkerOptions;
use web_sys::WorkerType;
use std::cell::RefCell;
use std::convert::TryInto;
use std::num;
use std::ops::DerefMut;
// use std::num;
use std::rc::Rc;

use eframe::glow::TRIANGLES;
use eframe::glow::TRIANGLE_STRIP;
use egui::debug_text::print;
use egui::util::undoer::Settings;
// use egui::epaint::Vertex;
// use egui::frame;
use glm::log;
use glm::vec2;
use glm::vec3;
use glm::vec4;
use glm::vec4_to_vec3;
use glm::Mat4;
use glm::Vec2;
use glm::Vec3;
use crate::camera::Camera;
use crate::renderer;
use crate::scene::Scene;  // Use crate:: to import from your lib.rs
use crate::loader::loader;  // If you need the loader
use crate::log;
use crate::timer::Timer;
use crate::utils::set_panic_hook;
use crate::utils::invert_row;
// use crate::shader;
use crate::shader;
extern crate js_sys;
extern crate ply_rs;
extern crate wasm_bindgen;
extern crate eframe;
extern crate nalgebra_glm as glm;
use crate::scene_geo;
use js_sys::{Float32Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::Event;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlInputElement;
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;
use web_sys::WebGlProgram;
use web_sys::WebGlVertexArrayObject;
use web_sys::MouseEvent;
use web_sys::WebGlTexture;
use web_sys::WebGlUniformLocation;

#[wasm_bindgen]
extern "C" {
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


pub fn get_scene_ready_for_draw(width: i32, height: i32, scene: &mut Scene, cam: &Camera) -> (Mat4, Mat4, Vec<u32>){
    let _timer = Timer::new("get_scene_ready_for_draw");
    let mut proj = glm::perspective((width as f32)/ (height as f32), 0.820176f32, 0.1f32, 100f32);
    // camera = camera.try_inverse().unwrap();

    let mut vm = cam.get_world_to_camera_matrix();
    let mut vpm = proj * vm;
    invert_row(&mut vm, 1);
    invert_row(&mut vm, 2);
    invert_row(&mut vpm, 1);
    invert_row(&mut vm, 0);
    invert_row(&mut vpm, 0);

    let splat_indices = scene.sort_splats_based_on_depth(vpm);
    return (vm, vpm, splat_indices)
}



#[allow(non_snake_case)]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();
    // log!("web!");
    // let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    // let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    // let mut scene = Scene::new(ply_splat);
    // let serealized = serde_json::to_string(&scene).unwrap();
    // log!("serialized = {}", serealized);
    // log!("Done loading!");
    // Load the JSON file dynamically
    // let window = web_sys::window().unwrap();
    // let mut scene: Scene = Scene::new_from_url("http://127.0.0.1:5501/splats/one-corn.json").await;
    // let scene_name = "shahan_head";
    // let scene_name = "Shahan_03_id01-30000.cleaned";
    // unsafe {
    //     if !worker_initialized {
    //         // let worker_options = WorkerOptions::new();
    //         // worker_options.set_type(WorkerType::Module);

    //         //     let worker_handle = Rc::new(RefCell::new(Worker::new_with_options("./worker.js", &worker_options).unwrap()));
    //         // console::log_1(&"Created a new worker from within Wasm".into());
    //         // worker_handle.borrow_mut().post_message(&JsValue::from_str("hello from wasm")).unwrap();
    //         worker_initialized = true;
    //     }
    //     else {
    //         return Ok(());
    //     }
    // }
    log!("Starting Web!");

    let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "E7_01_id01-30000";
    // let scene_name = "Shahan_03_id01-30000.cleaned";
    // let scene_name = "Shahan_03_id01-30000-2024";
    let mut scene: Scene = Scene::new_from_url(&format!("http://127.0.0.1:5501/splats/{}.rkyv", scene_name)).await;
    // let mut scene: Scene = Scene::new_from_json(&loaded_file);
    // log!("deserialized = {:?}", scene);
    // let ply_splat = loader::loader::load_ply().await.expect("something went wrong in loading");
    // log!("Done loading!");



    let _timer = Timer::new("start function");
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    let gl = getWebGLContext();
    let renderer = renderer::Renderer::new(gl, &scene).unwrap();

    let camera = Rc::new(RefCell::new(Camera::new(vec3(0.0, 0.0, 5.0), vec2(0.0, 0.0))));
    Camera::setup_mouse_events(&camera.clone(), &canvas, &document).unwrap();


    let keys_pressed = Rc::new(RefCell::new(std::collections::HashSet::new()));
    let keys_pressed_clone = keys_pressed.clone();
    let keydown_cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        keys_pressed_clone.borrow_mut().insert(e.key());
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref())?;
    keydown_cb.forget();

    let keys_pressed_clone = keys_pressed.clone();
    let keyup_cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        keys_pressed_clone.borrow_mut().remove(&e.key());
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("keyup", keyup_cb.as_ref().unchecked_ref())?;
    keyup_cb.forget();


    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut i = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        let _timer = Timer::new("main loop");
        camera.borrow_mut().update_translation_from_keys(&keys_pressed);

        let (vm, vpm, splat_indices) = get_scene_ready_for_draw(width, height, &mut scene, &camera.borrow());

        renderer.update_splat_indices(&splat_indices);
        renderer.draw(&canvas, i, scene.splats.len() as i32, vpm, vm);

        i += 1;
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}


// pub struct AppState{
//     scene: Scene,
//     webgl: WebGLSetupResult,
//     canvas: HtmlCanvasElement,
//     width: i32,
//     height: i32,
//     i: i32,
// }