use crate::camera::Camera;
use crate::gizmo::GizmoAxis;
use crate::log;
use crate::obj_reader;
use crate::renderer;
use crate::renderer::Renderer;
use crate::scene::Scene;
use crate::scene_geo;
use crate::scene_object::SceneObject;
// Use crate:: to import from your lib.rs
use crate::data_objects::MeshData;
use crate::data_objects::SplatData;
use crate::oct_tree::OctTree;
use crate::timer::Timer;
use crate::toggle_binding::ToggleBinding;
use crate::utils::set_panic_hook;
use glm::vec2;
use glm::vec3;
use nalgebra_glm::vec1;
use nalgebra_glm::Vec2;
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
use web_sys::{Document, HtmlElement};

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
    pub view_individual_splats: bool,
    pub do_sorting: bool,
    pub do_blending: bool,
    pub move_down: bool,
    pub selected_object: Option<usize>,
    pub restrict_gizmo_movement: bool,
}

fn handle_splat_delete_click(
    state: &ClickState,
    width: i32,
    height: i32,
    camera: &Camera,
    scene: &mut Scene,
    renderer: &renderer::Renderer,
    keys_pressed: &std::collections::HashSet<String>,
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
            let oct_tree = &mut scene.oct_tree;
            log!("finding splats in radius {:?}", pos);
            let octree_found_splats = oct_tree.find_splats_in_radius(pos, 0.05);
            for splat in octree_found_splats {
                // log!("octree found splat {:?}", splat.opacity);
                if splat.opacity >= 0.5 {
                    splat_pos = vec3(splat.x, splat.y, splat.z);
                    log!("found splat {:?}!! ### EXiting", splat_pos);
                    found = true;
                    scene.redraw_from_oct_tree(settings.only_show_clicks);
                    break;
                }
            }
        } else {
            for splat in scene.splat_data.splats.iter_mut() {
                if glm::distance(&vec3(splat.x, splat.y, splat.z), &pos) < 0.05
                    && splat.opacity >= 0.5
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
        let oct_tree = &mut scene.oct_tree;
        let splats_near = oct_tree.find_splats_in_radius(splat_pos, 0.5);
        for splat in splats_near {
            log!("splat near {:?}", splat.opacity);
            scene.splat_data.splats[splat.index].opacity = 0.0;
            //     scene.splat_data.splats[splat.index].r -= 0.1;
            //     scene.splat_data.splats[splat.index].g -= 0.1;
            //     scene.splat_data.splats[splat.index].b -= 0.1;
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
        .update_webgl_textures(&scene, 0)
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
    // check URL, if there is a

    // let scene_name = "socratica_01_edited";
    // log!("Loading web!");
    // let scene_name = "Week-09-Sat-Nov-16-2024";
    // let scene_name = "sci_01";
    // let scene_name = "sci_01";
    // let scene_name = "icon_01";
    //
    // let scene_name = "Shahan_03_id01-30000-2024";
    // let scene_name = "Shahan_03_id01-30000.cleaned";
    // let scene_name = "soc_01_polycam";
    // let scene_name = "soc_02";

    let window = web_sys::window().unwrap();
    let search = window.location().search().unwrap();
    let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();

    // If there's a url parameter, use that, otherwise use the default
    let scene_url = match params.get("url") {
        Some(url) => url,
        // None => String::from("http://127.0.0.1:5502/splats/soc_01_polycam.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/sci_01_edited.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/Shahan_03_id01-30000.cleaned.rkyv"),
        None => String::from("http://127.0.0.1:5502/splats/Shahan_03_id01-30000.rkyv"),
    };
    // let scene_name = "soc_02_edited";
    let mut splat: SplatData = SplatData::new_from_url(&scene_url).await;

    // let scene_name = "Shahan_03_id01-30000.cleaned";
    // let splat2: SplatData =
    //     SplatData::new_from_url(&format!("http://127.0.0.1:5502/splats/{}.rkyv", scene_name)).await;
    // splat.merge_with_other_splatdata(splat2);
    // splat.apply_transformation_to_object(
    //     1,
    //     glm::translate(&glm::Mat4::identity(), &vec3(1.0, -2.0, 1.0)),
    //     glm::rotate(
    //         &glm::Mat4::identity(),
    //         glm::radians(&vec1(90.0))[0],
    //         &vec3(0.0, 1.0, 0.0),
    //     ),
    // );
    let scene = Rc::new(RefCell::new(Scene::new(splat)));

    // let scene_2 = Rc::new(RefCell::new(Scene::new(splat)));

    // scene_2.borrow_mut().model_transform =
    //     glm::translate(&glm::Mat4::identity(), &vec3(0.0, -1.5, 0.0));

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
        scene_geo::CUBE_NORMALS.to_vec(),
    );
    let cube_object = SceneObject::new(
        cube_mesh.clone(),
        vec3(-0.0, -2.0, -0.2),
        vec3(0.0, 0.0, 0.0),
        // vec3(1.0, 0.1, 0.1),
        vec3(0.1, 0.1, 0.1),
    );
    // let min = cube_object.min;
    // let max = cube_object.max;
    scene.borrow_mut().objects.push(cube_object);
    let cube_object_2 = SceneObject::new_cube(vec3(2.0, -2.0, -0.2), 0.1, vec3(0.0, 1.0, 0.1));
    scene.borrow_mut().objects.push(cube_object_2);

    let obj_name = "teapot.obj";
    let teapot_mesh =
        obj_reader::read_obj(&format!("http://127.0.0.1:5502/obj/{}", obj_name)).await;
    let teapot_object = SceneObject::new(
        teapot_mesh,
        vec3(0.2, -0.2, 0.0),
        vec3(3.14, 0.0, 0.0),
        vec3(0.01, 0.01, 0.01),
    );
    scene.borrow_mut().objects.push(teapot_object);

    // {
    //     let mut scene_mut = scene.borrow_mut();
    //     if let Some(first_object) = scene_mut.objects.first() {
    //         let pos = first_object.pos;
    //         scene_mut.gizmo.update_position(pos);
    //         scene_mut.gizmo.target_object = Some(0);
    //     }
    // }

    let mut settings = Settings {
        show_octtree: false,
        only_show_clicks: false,
        use_octtree_for_splat_removal: true,
        view_individual_splats: false,
        do_sorting: true,
        do_blending: true,
        move_down: false,
        restrict_gizmo_movement: false,
        selected_object: None,
    };
    let settings_ref = Rc::new(RefCell::new(settings));
    let settings_clone = settings_ref.clone();

    scene
        .borrow_mut()
        .redraw_from_oct_tree(settings_ref.clone().borrow_mut().only_show_clicks);

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
    let renderer = Rc::new(RefCell::new(
        Renderer::new(gl, &scene.borrow_mut()).unwrap(),
    ));

    // // setup scene 2
    // renderer.borrow_mut().bind_splat_textures(1);
    // renderer
    //     .borrow_mut()
    //     .update_webgl_textures(&scene_2.borrow(), 1)
    //     .unwrap();

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
    Camera::setup_mouse_events(&camera.clone(), &canvas, &document, &scene)?;

    let shahan_remote_url =
        "https://zimpmodels.s3.us-east-2.amazonaws.com/splats/Shahan_03_id01-30000.cleaned.rkyv";
    let shahan_local_url = "http://127.0.0.1:5502/splats/Shahan_03_id01-30000.rkyv";
    let shahan_splat_data = SplatData::new_from_url(&shahan_remote_url).await;

    let obj_name = "teapot.obj";
    let teapot_mesh =
        obj_reader::read_obj(&format!("http://127.0.0.1:5502/obj/{}", obj_name)).await;

    setup_button_callbacks(
        scene.clone(),
        &renderer.clone(),
        settings_ref.clone(),
        camera.clone(),
        shahan_splat_data,
        teapot_mesh,
        width,
        height,
    )?;

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

    let scene_clone = scene.clone();
    let camera_clone = camera.clone();
    let renderer_clone = renderer.clone();
    let settings_ref_clone = settings_ref.clone();
    let click_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut scene = scene_clone.borrow_mut();
        let mut state = click_state_clone.borrow_mut();
        let renderer = renderer_clone.borrow();
        state.clicked = true;
        state.dragging = true;
        state.x = e.client_x();
        state.y = e.client_y();
        state.button = e.button();
        // handle_mouse_down(
        //     e,
        //     scene_clone.clone(),
        //     camera_clone.clone(),
        //     settings_ref_clone.clone(),
        // );

        let (vm, vpm) = camera_clone.borrow().get_vm_and_vpm(width, height);
        let (index, is_gizmo, hit_object) =
            renderer.get_at_mouse_position(width, height, state.x, state.y, vpm, vm, &scene);
        log!("just got mouse down!");
        log!(
            "index: {:?}, is_gizmo: {:?}, hit_object: {:?}",
            index,
            is_gizmo,
            hit_object
        );
        if hit_object {
            if !is_gizmo {
                scene.update_gizmo_position(index);
                scene.end_gizmo_drag();
            } else {
                let axis = match index {
                    0 => GizmoAxis::X,
                    1 => GizmoAxis::Y,
                    2 => GizmoAxis::Z,
                    _ => return,
                };
                log!("starting gizmo drag!");
                scene.start_gizmo_drag(axis, Vec2::new(state.x as f32, state.y as f32));
            }
        } else {
            log!("hiding gizmo!");
            scene.hide_gizmo();
        }
    }) as Box<dyn FnMut(_)>);
    // scene
    //     .objects
    //     .push(SceneObject::new_cube(vec3(0.0, 0.0, 0.0), 1.0));

    canvas.add_event_listener_with_callback("mousedown", click_cb.as_ref().unchecked_ref())?;
    click_cb.forget();

    let click_state_move = click_state.clone();
    let scene_clone = scene.clone();
    let camera_clone = camera.clone();
    let mousemove_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut state = click_state_move.borrow_mut();
        if state.dragging {
            state.x = e.client_x();
            state.y = e.client_y();
            state.clicked = true;
            // handle_mouse_move(e, scene_clone.clone(), camera_clone.clone());
        }
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())?;
    mousemove_cb.forget();

    let click_state_up = click_state.clone();
    let scene_clone = scene.clone();
    let mouseup_cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        let mut state = click_state_up.borrow_mut();
        state.dragging = false;
        scene_clone.borrow_mut().end_gizmo_drag();
        // handle_mouse_up(scene_clone.clone());
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())?;
    mouseup_cb.forget();

    let bindings: Vec<ToggleBinding> = vec![
        ToggleBinding::new(
            "show-octtree-checkbox",
            "o",
            |s| s.show_octtree,
            |s, v| s.show_octtree = v,
            |settings, scene| {
                scene.redraw_from_oct_tree(settings.only_show_clicks);
                log!("show octtree: {:?}", settings.show_octtree);
            },
        ),
        ToggleBinding::new(
            "only-show-clicks-checkbox",
            "c",
            |s| s.only_show_clicks,
            |s, v| s.only_show_clicks = v,
            |settings, scene| {
                scene.redraw_from_oct_tree(settings.only_show_clicks);
                log!("only show clicks: {:?}", settings.only_show_clicks);
            },
        ),
        ToggleBinding::new(
            "use-octtree-for-editing-checkbox",
            "f",
            |s| s.use_octtree_for_splat_removal,
            |s, v| s.use_octtree_for_splat_removal = v,
            |settings, scene| {},
        ),
        ToggleBinding::new(
            "view-individual-splats-checkbox",
            "v",
            |s| s.view_individual_splats,
            |s, v| s.view_individual_splats = v,
            |settings, scene| {},
        ),
        ToggleBinding::new(
            "do-sorting-checkbox",
            "m",
            |s| s.do_sorting,
            |s, v| s.do_sorting = v,
            |settings, scene| {},
        ),
        ToggleBinding::new(
            "do-blending-checkbox",
            "b",
            |s| s.do_blending,
            |s, v| s.do_blending = v,
            |settings, scene| {},
        ),
        ToggleBinding::new(
            "move-down-checkbox",
            "_",
            |s| s.move_down,
            |s, v| s.move_down = v,
            |settings, scene| {},
        ),
        ToggleBinding::new(
            "restrict-gizmo-movement-checkbox",
            "r",
            |s| s.restrict_gizmo_movement,
            |s, v| s.restrict_gizmo_movement = v,
            |settings, scene| {},
        ),
    ];

    for binding in &bindings {
        binding.setup_ui_listener(settings_ref.clone(), scene.clone())?;
    }

    // scene.add_line(
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(10.0, 0.0, 0.0),
    //     vec3(1.0, 0.0, 0.0),
    // );

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

        scene.borrow_mut().objects[0].recalculate_min_max();
        if settings.borrow().move_down {
            scene.borrow_mut().move_down();
        }
        // log!("min: {:?}, max: {:?}", min, max);

        cam_mut.update_translation_from_keys(&keys_pressed.borrow());
        let (vm, vpm) = cam_mut.get_vm_and_vpm(width, height);

        if click_state.borrow().clicked {
            let state = click_state.borrow();
            handle_splat_delete_click(
                &state,
                width,
                height,
                &cam_mut,
                &mut scene.borrow_mut(),
                &renderer.borrow(),
                &keys_pressed.borrow(),
                &settings.borrow(),
            );

            if !click_state.borrow().dragging {
                drop(state);
                click_state.borrow_mut().clicked = false;
                click_state.borrow_mut().dragging = false;
            } else {
                if scene.borrow().gizmo.is_dragging {
                    log!("updating gizmo drag!");
                    scene.borrow_mut().update_gizmo_drag(
                        Vec2::new(state.x as f32, state.y as f32),
                        settings.borrow().restrict_gizmo_movement,
                    );
                }
            }
        }

        for binding in &bindings {
            if keys_pressed.borrow().contains(&binding.key)
                && key_change_handled.borrow().contains(&binding.key)
            {
                let settings = settings_ref.clone();
                binding.handle_key_press(&mut settings.borrow_mut());
                binding.update_ui(&settings.borrow());

                (binding.on_toggle)(&settings.borrow(), &mut scene.borrow_mut());

                key_change_handled.borrow_mut().remove(&binding.key);
            }
        }

        let (normal_projection_matrix, normal_view_matrix) =
            cam_mut.get_normal_projection_and_view_matrices(width, height);
        // if settings.borrow().do_sorting {
        //     let splat_indices = scene
        //         .borrow_mut()
        //         .splat_data
        //         .sort_splats_based_on_depth(vpm);
        //     renderer.borrow_mut().update_splat_indices(&splat_indices);
        // }

        if settings.borrow().do_sorting {
            let splat_indices = scene
                .borrow_mut()
                .splat_data
                .sort_splats_based_on_depth(vpm);
            // renderer
            //     .borrow_mut()
            //     .update_webgl_textures(&scene.borrow())
            //     .unwrap();
            renderer.borrow_mut().update_splat_indices(&splat_indices);
        }
        renderer.borrow_mut().bind_splat_textures(0);
        renderer.borrow_mut().draw_scene(
            &canvas,
            &scene.borrow(),
            vpm,
            vm,
            normal_projection_matrix,
            normal_view_matrix,
            &settings.borrow(),
            true,
        );

        // renderer.borrow_mut().bind_splat_textures(1);
        // let splat_indices = scene_2
        //     .borrow_mut()
        //     .splat_data
        //     .sort_splats_based_on_depth(vpm);
        // renderer.borrow_mut().update_splat_indices(&splat_indices);

        // renderer
        //     .borrow_mut()
        //     .update_webgl_textures(&scene_2.borrow())
        //     .unwrap();
        // renderer.borrow_mut().draw_scene(
        //     &canvas,
        //     &scene_2.borrow(),
        //     vpm,
        //     vm,
        //     normal_projection_matrix,
        //     normal_view_matrix,
        //     &settings.borrow(),
        //     false,
        // );

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

    #[wasm_bindgen(js_name = setCollisionDetected)]
    pub fn setCollisionDetected();

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

fn setup_button_callbacks(
    scene: Rc<RefCell<Scene>>,
    renderer: &Rc<RefCell<Renderer>>,
    settings: Rc<RefCell<Settings>>,
    camera: Rc<RefCell<Camera>>,
    shahan_splat_data: SplatData,
    teapot_mesh: MeshData,
    width: i32,
    height: i32,
) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Move Down button
    // let move_down_btn = document
    //     .get_element_by_id("move-down-btn")
    //     .unwrap()
    //     .dyn_into::<HtmlElement>()?;

    // let scene_clone = scene.clone();
    // let move_down_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
    //     let mut scene = scene_clone.borrow_mut();
    //     // Add your move down logic here
    //     log!("Moving down!");
    // }) as Box<dyn FnMut(_)>);
    // move_down_btn.set_onclick(Some(move_down_callback.as_ref().unchecked_ref()));
    // move_down_callback.forget(); // Prevent callback from being dropped

    // Calculate Shadows button
    let shadows_btn = document
        .get_element_by_id("calculate-shadows-btn")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    let scene_clone = scene.clone();
    let renderer_clone = renderer.clone();
    let shadows_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        let renderer = renderer_clone.borrow();
        let mut scene = scene_clone.borrow_mut();
        scene.calculate_shadows();
        renderer
            .update_webgl_textures(&scene, 0)
            .expect("failed to update webgl textures when editing");
    }) as Box<dyn FnMut(_)>);

    shadows_btn.set_onclick(Some(shadows_callback.as_ref().unchecked_ref()));
    shadows_callback.forget();

    // Recalculate Octtree button
    let recalculate_octtree_btn = document
        .get_element_by_id("recalculate-octtree-btn")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    let scene_clone = scene.clone();
    let settings_clone = settings.clone();
    let recalculate_octtree_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        let settings = settings_clone.borrow();
        let mut scene = scene_clone.borrow_mut();
        scene.recalculate_octtree();
        scene.redraw_from_oct_tree(settings.only_show_clicks);
    }) as Box<dyn FnMut(_)>);
    recalculate_octtree_btn
        .set_onclick(Some(recalculate_octtree_callback.as_ref().unchecked_ref()));
    recalculate_octtree_callback.forget();

    // Add Shahan button
    let add_shahan_btn = document
        .get_element_by_id("add-shahan-btn")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    let scene_clone = scene.clone();
    let renderer_clone = renderer.clone();
    let shahan_splat_data = Rc::new(shahan_splat_data);
    let shahan_splat_data_clone = shahan_splat_data.clone();
    let settings_clone = settings.clone();
    let camera_clone = camera.clone();
    let add_shahan_callback = Closure::wrap(Box::new(move || {
        let renderer: &Renderer = &*renderer_clone.borrow();
        let settings = settings_clone.borrow();
        let mut scene = scene_clone.borrow_mut();
        let camera = camera_clone.borrow();
        scene
            .splat_data
            .merge_with_other_splatdata(&shahan_splat_data_clone); // Clone the data here
        let num_splats = scene.splat_data.objects.len();
        let (origin, direction) = camera.get_ray_origin_and_direction(width, height, 0.0, 0.0);
        let pos = origin + direction * 3.0;
        scene.splat_data.apply_transformation_to_object(
            num_splats - 1,
            glm::translate(&glm::Mat4::identity(), &pos),
            glm::Mat4::identity(),
            // glm::rotate(
            //     &glm::Mat4::identity(),
            //     glm::radians(&vec1(90.0))[0],
            //     &vec3(0.0, 1.0, 0.0),
            // ),
        );
        scene.recalculate_octtree();
        scene.redraw_from_oct_tree(settings.only_show_clicks);
        renderer.update_webgl_textures(&scene, 0).unwrap();
    }) as Box<dyn FnMut()>);
    add_shahan_btn.set_onclick(Some(add_shahan_callback.as_ref().unchecked_ref()));
    add_shahan_callback.forget();

    // Add Teapot button
    let add_teapot_btn = document
        .get_element_by_id("add-teapot-btn")
        .unwrap()
        .dyn_into::<HtmlElement>()?;

    let scene_clone = scene.clone();
    let camera_clone = camera.clone();
    let teapot_mesh = Rc::new(teapot_mesh);
    let teapot_mesh_clone = teapot_mesh.clone();
    let add_teapot_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        let mut scene = scene_clone.borrow_mut();
        let camera = camera_clone.borrow();
        let (origin, direction) = camera.get_ray_origin_and_direction(width, height, 0.0, 0.0);
        let pos = origin + direction * 3.0;
        let teapot_object = SceneObject::new(
            teapot_mesh_clone.as_ref().clone(),
            pos,
            vec3(3.14, 0.0, 0.0),
            vec3(0.01, 0.01, 0.01),
        );
        scene.objects.push(teapot_object);
    }) as Box<dyn FnMut(_)>);
    add_teapot_btn.set_onclick(Some(add_teapot_callback.as_ref().unchecked_ref()));
    add_teapot_callback.forget();

    Ok(())
}
