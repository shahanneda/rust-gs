use crate::camera::Camera;
use crate::log;
use crate::renderer;
use crate::scene::Scene;
use crate::scene_geo;
use crate::scene_object::SceneObject;
// Use crate:: to import from your lib.rs
use crate::timer::Timer;
use crate::utils::set_panic_hook;
use crate::DataObjects::MeshData;
use crate::DataObjects::OctTree;
use crate::DataObjects::SplatData;
use glm::vec2;
use glm::vec3;
use std::cell::RefCell;
use std::rc::Rc;
extern crate eframe;
extern crate js_sys;
extern crate nalgebra_glm as glm;
extern crate ply_rs;
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

#[derive(Default)]
struct ClickState {
    clicked: bool,
    x: i32,
    y: i32,
    button: i16,
}

fn handle_click(
    state: &ClickState,
    width: i32,
    height: i32,
    camera: &Camera,
    scene: &mut Scene,
    renderer: &renderer::Renderer,
    keys_pressed: &std::collections::HashSet<String>,
) {
    let ndc_x = (state.x as f32 / width as f32) * 2.0 - 1.0;
    let ndc_y = 1.0 - (state.y as f32 / height as f32) * 2.0;
    if !keys_pressed.contains("Alt") {
        return;
    }

    let (ray_origin, ray_direction) =
        camera.get_ray_origin_and_direction(width, height, ndc_x, ndc_y);

    log!("Click detected at x: {}, y: {}", state.x, state.y);
    log!("Unprojected: x: {}, y: {}", state.x, state.y);
    log!("Ray origin: {:?}", ray_origin);
    log!("Ray direction: {:?}", ray_direction);

    // Remove splats near the ray
    let mut splat_pos = vec3(0.0, 0.0, 0.0);
    let mut found = false;
    for t in 0..100 {
        let t = t as f32 / 10.0;
        let pos = ray_origin + ray_direction * t;

        // scene.objects.push(SceneObject::new(
        //     MeshData::new(
        //         scene_geo::PYRAMID_VERTICES.to_vec(),
        //         vec![],
        //         scene_geo::PYRAMID_COLORS.to_vec(),
        //     ),
        //     pos,
        //     vec3(0.0, 0.0, 0.0),
        //     vec3(0.05, 0.05, 0.05),
        // ));

        for mut splat in scene.splat_data.splats.iter_mut() {
            if glm::distance(&vec3(splat.x, splat.y, splat.z), &pos) < 0.1 && splat.opacity > 0.1 {
                splat_pos = vec3(splat.x, splat.y, splat.z);
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    if found {
        for mut splat in scene.splat_data.splats.iter_mut() {
            if glm::distance(&vec3(splat.x, splat.y, splat.z), &splat_pos) < 0.5 {
                splat.opacity = 0.0;
            }
        }
    }

    renderer
        .update_webgl_textures(&scene)
        .expect("failed to update webgl textures when editing");

    match state.button {
        0 => log!("Left click"),
        1 => log!("Middle click"),
        2 => log!("Right click"),
        _ => log!("Other button"),
    }
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
    // log!("Starting Web!");

    // let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "E7_01_id01-30000";
    // let scene_name = "corn";
    let scene_name = "Shahan_03_id01-30000.cleaned";
    // let scene_name = "sci_01";
    // let scene_name = "soc_01_polycam";
    //
    // let scene_name = "Shahan_03_id01-30000-2024";
    let mut splat: SplatData =
        SplatData::new_from_url(&format!("http://127.0.0.1:5502/splats/{}.rkyv", scene_name)).await;
    let mut scene = Scene::new(splat);
    // let pyramid_mesh = MeshData::new(
    //     scene_geo::PYRAMID_VERTICES.to_vec(),
    //     // scene_geo::PYRAMID_INDICES,
    //     vec![],
    //     scene_geo::PYRAMID_COLORS.to_vec(),
    // );
    let cube_mesh = MeshData::new(
        scene_geo::CUBE_VERTICES.to_vec(),
        scene_geo::CUBE_INDICES.to_vec(),
        scene_geo::CUBE_COLORS.to_vec(),
    );
    // scene.objects.push(SceneObject::new(
    //     pyramid_mesh.clone(),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(1.0, 1.0, 1.0),
    // ));

    scene.objects.push(SceneObject::new(
        cube_mesh.clone(),
        vec3(3.0, 0.0, 0.0),
        vec3(0.0, 0.0, 0.0),
        vec3(1.05, 1.05, 1.05),
    ));

    // scene.objects.push(SceneObject::new(
    //     MeshData::new(
    //         scene_geo::CUBE_VERTICES.to_vec(),
    //         vec![],
    //         scene_geo::CUBE_COLORS.to_vec(),
    //     ),
    //     vec3(-5.0, 0.0, 0.0),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(1.0, 1.0, 1.0),
    // ));
    // let mut scene: Scene =
    //     Scene::new_from_url(&format!("http://127.0.0.1:5502/splats/{}.rkyv", scene_name)).await;
    // let mut scene: Scene = Scene::new_from_url(
    //     "https://zimpmodels.s3.us-east-2.amazonaws.com/splats/e7eb4bda-1d7c-4ca8-ac6b-a4c2c722f014.rkyv",
    // )
    // .await;
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
    // Camera Pos = [[-1.020468, 1.4699098, -2.7163901]]
    // gs_rust.js:547 Camera Rot = [[0.11999998, 2.8230002]]
    let camera = Rc::new(RefCell::new(Camera::new(
        vec3(0.0, 0.0, 0.0),
        // vec3(-1.020468, 1.4699098, -2.7163901),
        // vec2(0.11999998, 2.8230002),
        vec2(0.0, 3.14 / 2.0),
    )));
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

    let click_state = Rc::new(RefCell::new(ClickState::default()));
    let click_state_clone = click_state.clone();

    let click_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut state = click_state_clone.borrow_mut();
        state.clicked = true;
        state.x = e.client_x();
        state.y = e.client_y();
        state.button = e.button();
    }) as Box<dyn FnMut(_)>);
    // scene
    //     .objects
    //     .push(SceneObject::new_cube(vec3(0.0, 0.0, 0.0), 1.0));

    canvas.add_event_listener_with_callback("mousedown", click_cb.as_ref().unchecked_ref())?;
    click_cb.forget();

    let oct_tree = OctTree::new(scene.splat_data.splats.clone());
    let cubes = oct_tree.get_cubes();
    for cube in cubes {
        scene.objects.push(cube);
    }
    // let cube =
    // scene.objects.push(SceneObject::new_cube(
    //     vec3(0.0, 0.0, 0.0),
    //     1.0,
    //     vec3(0.0, 1.0, 0.0),
    // ));
    // scene.objects.push(SceneObject::new_cube(
    //     vec3(1.0, 0.0, 0.0),
    //     1.0,
    //     vec3(0.0, 0.0, 1.0),
    // ));
    // scene.objects.push(SceneObject::new_cube(
    //     vec3(-1.0, 0.0, 0.0),
    //     0.5,
    //     vec3(0.0, 1.0, 1.0),
    // ));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut i = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        let _timer = Timer::new("main loop");
        let mut cam_mut = camera.borrow_mut();

        if click_state.borrow().clicked {
            let state = click_state.borrow();
            handle_click(
                &state,
                width,
                height,
                &cam_mut,
                &mut scene,
                &renderer,
                &keys_pressed.borrow(),
            );

            // Reset the click state
            drop(state);
            click_state.borrow_mut().clicked = false;
        }
        // if i % 1000 < 500 {
        //     // scene.objects[0].rot.y += 0.01;
        //     scene.objects[0].pos.y += 0.01;
        // } else {
        //     // scene.objects[0].rot.y -= 0.01;
        //     scene.objects[0].pos.y -= 0.01;
        // }

        cam_mut.update_translation_from_keys(&keys_pressed);
        log!("camera pos: {:?}", cam_mut.pos);
        let (vm, vpm) = cam_mut.get_vm_and_vpm(width, height);

        let splat_indices = scene.splat_data.sort_splats_based_on_depth(vpm);
        renderer.update_splat_indices(&splat_indices);
        renderer.draw_scene(&canvas, &scene, vpm, vm);

        i += 1;
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = getWebGLContext)]
    fn getWebGLContext() -> WebGl2RenderingContext;

    #[wasm_bindgen(js_name = test)]
    fn test_js();

    #[wasm_bindgen(js_name = prompt)]
    fn promptJS(str: &str) -> String;

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
