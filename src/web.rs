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
use crate::DataObjects::SplatData;
use crate::OctTree::OctTree;
use crate::ToggleBinding::ToggleBinding;
use glm::vec2;
use glm::vec3;
use std::cell::RefCell;
use std::rc::Rc;
extern crate eframe;
extern crate js_sys;
extern crate nalgebra_glm as glm;
extern crate ply_rs;
extern crate wasm_bindgen;
use eframe::egui;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext;

#[derive(Default)]
struct ClickState {
    clicked: bool,
    dragging: bool,
    x: i32,
    y: i32,
    button: i16,
}

pub struct Settings {
    pub show_octtree: bool,
    pub only_show_clicks: bool,
    pub use_octtree_for_splat_removal: bool,
}

fn handle_click(
    state: &ClickState,
    width: i32,
    height: i32,
    camera: &Camera,
    scene: &mut Scene,
    renderer: &renderer::Renderer,
    keys_pressed: &std::collections::HashSet<String>,
    oct_tree: &mut OctTree,
    settings: &Settings,
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
        // scene.add_line(
        //     pos,
        //     pos + 0.2 * ray_direction,
        //     vec3(pos.x / 100.0, pos.y / 100.0, pos.z / 100.0),
        // );

        // scene.objects.push(SceneObject::new(
        //     MeshData::new(
        //         scene_geo::CUBE_VERTICES.to_vec(),
        //         scene_geo::CUBE_INDICES.to_vec(),
        //         scene_geo::CUBE_COLORS.to_vec(),
        //     ),
        //     pos,
        //     vec3(0.0, 0.0, 0.0),
        //     vec3(0.01, 0.01, 0.01),
        // ));

        // let octree_found_splats = oct_tree.find_splats_in_radius(pos, 0.1);
        // log!("octree found splats: {:?}", octree_found_splats.len());

        if settings.use_octtree_for_splat_removal {
            log!("finding splats in radius {:?}", pos);
            let octree_found_splats = oct_tree.find_splats_in_radius(pos, 0.05);
            for splat in octree_found_splats {
                // log!("octree found splat {:?}", splat.opacity);
                if splat.opacity >= 0.8 {
                    splat_pos = vec3(splat.x, splat.y, splat.z);
                    log!("found splat {:?}!! ### EXiting", splat_pos);
                    found = true;
                    scene.redraw_from_oct_tree(oct_tree, settings.only_show_clicks);
                    break;
                }
            }
        } else {
            for splat in scene.splat_data.splats.iter_mut() {
                if glm::distance(&vec3(splat.x, splat.y, splat.z), &pos) < 0.05
                    && splat.opacity >= 0.8
                {
                    splat_pos = vec3(splat.x, splat.y, splat.z);
                    found = true;
                    break;
                }
            }
        }

        if found {
            break;
        }
    }

    if settings.use_octtree_for_splat_removal {
        let splats_near = oct_tree.find_splats_in_radius(splat_pos, 0.5);
        for splat in splats_near {
            log!("splat near {:?}", splat.opacity);
            scene.splat_data.splats[splat.index].opacity = 0.0;
        }
    } else {
        for splat in scene.splat_data.splats.iter_mut() {
            if glm::distance(&vec3(splat.x, splat.y, splat.z), &splat_pos) < 0.5 {
                splat.opacity = 0.0;
            }
        }
    }

    // let octree_found_splats = oct_tree.find_splats_in_radius(vec3(-0.8, 0.0, 0.0), 0.1);
    // log!("octree found splats: {:?}", octree_found_splats.len());

    // scene.clear_lines();
    // let lines = oct_tree.get_lines();
    // for line in lines {
    //     scene.add_line(line.start, line.end, line.color);
    // }

    // log!("octree found splats: {:?}", octree_found_splats.len());

    // log!("splat pos is {:?}", splat_pos);
    // if found {
    //     for mut splat in scene.splat_data.splats.iter_mut() {
    //         if glm::distance(&vec3(splat.x, splat.y, splat.z), &splat_pos) < 0.5 {
    //             splat.opacity = 0.0;
    //         }
    //     }
    // }

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

    let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "E7_01_id01-30000";
    // let scene_name = "corn";

    // let scene_name = "Shahan_03_id01-30000.cleaned";
    // let scene_name = "socratica_01_edited";
    log!("Loading web!");
    // let scene_name = "Week-09-Sat-Nov-16-2024";
    // let scene_name = "sci_01";
    // let scene_name = "sci_01";
    // let scene_name = "icon_01";
    // let scene_name = "soc_01_polycam";
    //
    // let scene_name = "Shahan_03_id01-30000-2024";
    let mut splat: SplatData =
        SplatData::new_from_url(&format!("http://127.0.0.1:5502/splats/{}.rkyv", scene_name)).await;
    let scene = Rc::new(RefCell::new(Scene::new(splat)));

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

    let mut settings = Settings {
        show_octtree: false,
        only_show_clicks: false,
        use_octtree_for_splat_removal: true,
    };
    let settings_ref = Rc::new(RefCell::new(settings));
    let settings_clone = settings_ref.clone();

    let document = web_sys::window().unwrap().document().unwrap();
    let checkbox = document
        .get_element_by_id("show-octtree-checkbox")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();

    // let checkbox_clone = checkbox.clone();
    // let checkbox_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
    //     let mut settings = settings_clone.borrow_mut();
    //     settings.show_octtree = checkbox_clone.checked();
    //     // You'll need to pass scene and oct_tree here if you want to redraw immediately
    // }) as Box<dyn FnMut(_)>);

    // checkbox
    //     .add_event_listener_with_callback("change", checkbox_callback.as_ref().unchecked_ref())?;
    // checkbox_callback.forget();
    // scene.objects.push(SceneObject::new(
    //     pyramid_mesh.clone(),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(1.0, 1.0, 1.0),
    // ));

    // scene.objects.push(SceneObject::new(
    //     cube_mesh.clone(),
    //     vec3(3.0, 0.0, 0.0),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(1.05, 1.05, 1.05),
    // ));

    // for i in 0..100 {
    //     scene.add_line(
    //         vec3(i as f32 - 50.0, 0.0, 1.0),
    //         vec3(i as f32 - 50.0, i as f32, 1.0),
    //         vec3(i as f32 / 100.0, 10.0, 0.0),
    //     );
    // }

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
    let renderer = renderer::Renderer::new(gl, &scene.borrow()).unwrap();
    // Camera Pos = [[-1.020468, 1.4699098, -2.7163901]]
    // gs_rust.js:547 Camera Rot = [[0.11999998, 2.8230002]]
    let camera = Rc::new(RefCell::new(Camera::new(
        // camera pos: [[-6.679095, 0.14607938, -0.32618168]]
        // final_project.js:564 camera rot: [[-0.13400005, -1.5560011]]
        // vec3(0.0, 0.0, 0.0),
        vec3(-6.679095, 0.14607938, -0.32618168),
        vec2(-0.13400005, -1.5560011),
        // vec3(-1.020468, 1.4699098, -2.7163901),
        // vec2(0.0, 3.14 / 2.0),
    )));
    Camera::setup_mouse_events(&camera.clone(), &canvas, &document).unwrap();

    let keys_pressed = Rc::new(RefCell::new(std::collections::HashSet::new()));
    let key_change_handled = Rc::new(RefCell::new(std::collections::HashSet::<String>::new()));

    let keys_pressed_clone = keys_pressed.clone();
    let key_change_handled_clone = key_change_handled.clone();
    let keydown_cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        keys_pressed_clone.borrow_mut().insert(e.key());
        key_change_handled_clone.borrow_mut().insert(e.key());
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref())?;
    keydown_cb.forget();

    let keys_pressed_clone = keys_pressed.clone();
    let key_change_handled_clone = key_change_handled.clone();
    let keyup_cb = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        keys_pressed_clone.borrow_mut().remove(&e.key());
        key_change_handled_clone.borrow_mut().insert(e.key());
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("keyup", keyup_cb.as_ref().unchecked_ref())?;
    keyup_cb.forget();

    let click_state = Rc::new(RefCell::new(ClickState::default()));
    let click_state_clone = click_state.clone();

    let click_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut state = click_state_clone.borrow_mut();
        state.clicked = true;
        state.dragging = true;
        state.x = e.client_x();
        state.y = e.client_y();
        state.button = e.button();
    }) as Box<dyn FnMut(_)>);
    // scene
    //     .objects
    //     .push(SceneObject::new_cube(vec3(0.0, 0.0, 0.0), 1.0));

    canvas.add_event_listener_with_callback("mousedown", click_cb.as_ref().unchecked_ref())?;
    click_cb.forget();

    let click_state_move = click_state.clone();
    let mousemove_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut state = click_state_move.borrow_mut();
        if state.dragging {
            state.x = e.client_x();
            state.y = e.client_y();
            state.clicked = true;
        }
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())?;
    mousemove_cb.forget();

    let click_state_up = click_state.clone();
    let mouseup_cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        let mut state = click_state_up.borrow_mut();
        state.dragging = false;
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())?;
    mouseup_cb.forget();

    let mut oct_tree = Rc::new(RefCell::new(OctTree::new(
        scene.borrow().splat_data.splats.clone(),
    )));

    let bindings: Vec<ToggleBinding> = vec![
        ToggleBinding::new(
            "show-octtree-checkbox",
            "o",
            |s| s.show_octtree,
            |s, v| s.show_octtree = v,
            |settings, scene, oct_tree| {
                scene.redraw_from_oct_tree(oct_tree, settings.only_show_clicks);
                log!("show octtree: {:?}", settings.show_octtree);
            },
        ),
        ToggleBinding::new(
            "only-show-clicks-checkbox",
            "c",
            |s| s.only_show_clicks,
            |s, v| s.only_show_clicks = v,
            |settings, scene, oct_tree| {
                scene.redraw_from_oct_tree(oct_tree, settings.only_show_clicks);
                log!("only show clicks: {:?}", settings.only_show_clicks);
            },
        ),
    ];

    for binding in &bindings {
        binding.setup_ui_listener(settings_ref.clone(), scene.clone(), oct_tree.clone())?;
    }

    // scene.add_line(
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(10.0, 0.0, 0.0),
    //     vec3(1.0, 0.0, 0.0),
    // );

    // scene.objects.push(SceneObject::new(
    //     cube_mesh.clone(),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(1.0, 1.0, 1.0),
    // ));
    scene.borrow_mut().redraw_from_oct_tree(
        &oct_tree.borrow(),
        settings_ref.clone().borrow().only_show_clicks,
    );

    // let cubes = oct_tree.get_cubes();
    // for cube in cubes {
    //     scene.objects.push(cube);
    // }
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
    // let mut orbit_angle = 0.0f32;
    // let orbit_radius = 0.9f32;
    // let orbit_height = 0.2f32;
    // let orbit_speed = 0.005f32;
    *g.borrow_mut() = Some(Closure::new(move || {
        let _timer = Timer::new("main loop");
        let mut cam_mut = camera.borrow_mut();
        let settings = settings_ref.clone();
        // // Update orbit angle and camera position
        // orbit_angle += orbit_speed;
        // cam_mut.pos.x = orbit_radius * orbit_angle.cos();
        // cam_mut.pos.z = orbit_radius * orbit_angle.sin();
        // cam_mut.pos.y = orbit_height;

        // // Make the camera look at the center
        // cam_mut.rot.x = -0.13400005; // Keep original x rotation
        // cam_mut.rot.y = orbit_angle + std::f32::consts::PI / 2.0; // Make camera face center

        if click_state.borrow().clicked {
            let state = click_state.borrow();
            handle_click(
                &state,
                width,
                height,
                &cam_mut,
                &mut scene.borrow_mut(),
                &renderer,
                &keys_pressed.borrow(),
                &mut oct_tree.borrow_mut(),
                &settings.borrow(),
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

        // if keys_pressed.borrow().contains(&"o".to_string())
        //     && key_change_handled.borrow().contains(&"o".to_string())
        // {
        //     // settings.borrow_mut().show_octtree = !settings.borrow().show_octtree;
        //     let current_value = settings.borrow().show_octtree;
        //     let mut settings = settings_ref.clone();
        //     settings.borrow_mut().show_octtree = !current_value;
        //     key_change_handled.borrow_mut().remove(&"o".to_string());
        //     log!("show octtree: {:?}", settings.borrow().show_octtree);
        //     let document = web_sys::window().unwrap().document().unwrap();
        //     if let Ok(checkbox) = document
        //         .get_element_by_id("show-octtree-checkbox")
        //         .unwrap()
        //         .dyn_into::<web_sys::HtmlInputElement>()
        //     {
        //         checkbox.set_checked(!current_value);
        //     }
        //     scene.redraw_from_oct_tree(&oct_tree, settings.borrow().only_show_clicks);
        // }

        for binding in &bindings {
            if keys_pressed.borrow().contains(&binding.key)
                && key_change_handled.borrow().contains(&binding.key)
            {
                let settings = settings_ref.clone();
                binding.handle_key_press(&mut settings.borrow_mut());
                binding.update_ui(&settings.borrow());

                (binding.on_toggle)(
                    &settings.borrow(),
                    &mut scene.borrow_mut(),
                    &oct_tree.borrow(),
                );

                key_change_handled.borrow_mut().remove(&binding.key);
            }
        }

        if keys_pressed.borrow().contains(&"t".to_string())
            && key_change_handled.borrow().contains(&"t".to_string())
        {
            let mut settings = settings_ref.clone();
            settings.borrow_mut().use_octtree_for_splat_removal =
                !settings.borrow().use_octtree_for_splat_removal;
            key_change_handled.borrow_mut().remove(&"t".to_string());
            log!(
                "use octtree for splat removal: {:?}",
                settings.borrow().use_octtree_for_splat_removal
            );
        }

        // if keys_pressed.borrow().contains(&"c".to_string())
        //     && key_change_handled.borrow().contains(&"c".to_string())
        // {
        //     let mut settings = settings_ref.clone();
        //     settings.borrow_mut().only_show_clicks = !settings.borrow().only_show_clicks;
        //     key_change_handled.borrow_mut().remove(&"c".to_string());
        //     log!("only show clicks: {:?}", settings.borrow().only_show_clicks);
        //     scene
        //         .borrow_mut()
        //         .redraw_from_oct_tree(&oct_tree.borrow(), settings.borrow().only_show_clicks);
        // }

        cam_mut.update_translation_from_keys(&keys_pressed.borrow());
        // log!("camera pos: {:?}", cam_mut.pos);
        // log!("camera rot: {:?}", cam_mut.rot);
        let (vm, vpm) = cam_mut.get_vm_and_vpm(width, height);

        let splat_indices = scene
            .borrow_mut()
            .splat_data
            .sort_splats_based_on_depth(vpm);

        renderer.update_splat_indices(&splat_indices);
        let (normal_projection_matrix, normal_view_matrix) =
            cam_mut.get_normal_projection_and_view_matrices(width, height);

        renderer.draw_scene(
            &canvas,
            &scene.borrow(),
            vpm,
            vm,
            normal_projection_matrix,
            normal_view_matrix,
            &settings.borrow(),
        );

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
