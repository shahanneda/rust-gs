use crate::camera::Camera;
use crate::editor;
use crate::gizmo::{GizmoAxis, GizmoTarget};
use crate::log;
use crate::scene::EditOp;
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
use crate::utils::debug_memory;
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
use rayon::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, Element, HtmlElement, HtmlSelectElement, MouseEvent, Node, WebGl2RenderingContext,
    Window,
};

pub use wasm_bindgen_rayon::init_thread_pool;

// JavaScript function bindings for slicing UI
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "showSliceMode")]
    fn show_slice_mode();

    #[wasm_bindgen(js_name = "hideSliceMode")]
    fn hide_slice_mode();

    #[wasm_bindgen(js_name = "showSliceLine")]
    fn show_slice_line(start_x: i32, start_y: i32, end_x: i32, end_y: i32);

    #[wasm_bindgen(js_name = "hideSliceLine")]
    fn hide_slice_line();

    #[wasm_bindgen(js_name = "showDeleteMode")]
    fn show_delete_mode();

    #[wasm_bindgen(js_name = "hideDeleteMode")]
    fn hide_delete_mode();

    #[wasm_bindgen(js_name = "showLoadingOverlay")]
    fn show_loading_overlay(message: &str);

    #[wasm_bindgen(js_name = "hideLoadingOverlay")]
    fn hide_loading_overlay();

    #[wasm_bindgen(js_name = "updateLoadingMessage")]
    fn update_loading_message(message: &str);

    // Model loading status bindings
    #[wasm_bindgen(js_name = "setModelLoading")]
    fn set_model_loading(is_loading: bool, message: &str);

    // JavaScript setTimeout binding
    #[wasm_bindgen(js_name = "setTimeout")]
    fn set_timeout(closure: &Closure<dyn FnMut()>, delay: i32) -> i32;

    // Live "N splats will be erased" readout in the erase toolbar.
    #[wasm_bindgen(js_name = "updateEraseCount")]
    fn update_erase_count(count: u32);
}

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
    /// Show the red "will be erased" highlight while hovering in erase mode.
    pub eraser_preview: bool,
    /// How far apart the two halves move after a slice.
    pub slice_separation: f32,
    /// Slice the selected splat object instead of the whole first object.
    pub slice_target_selected: bool,
    /// 0 = split apart, 1 = cut & erase the positive side.
    pub slice_mode: u32,
}

/// March the mouse ray into the scene and return the point on the first
/// visible splat (used to position the eraser brush).
fn eraser_center_from_mouse(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    camera: &Camera,
    scene: &mut Scene,
) -> Option<glm::Vec3> {
    let ndc_x = (x as f32 / width as f32) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y as f32 / height as f32) * 2.0;
    let (ray_origin, ray_direction) =
        camera.get_ray_origin_and_direction(width, height, ndc_x, ndc_y);
    let idx = scene.pick_splat(ray_origin, ray_direction)?;
    let s = &scene.splat_data.splats[idx];
    Some(vec3(s.x, s.y, s.z))
}

/// Erase all visible splats within `radius` of `center`. Previous opacities
/// are appended to `changes` so a whole brush stroke can be undone at once.
fn erase_at(
    scene: &mut Scene,
    center: glm::Vec3,
    radius: f32,
    changes: &mut Vec<(usize, f32)>,
) -> usize {
    scene.ensure_octree();
    let found = scene.find_splats_in_radius(center, radius);
    let hidden: Vec<(u32, u32)> = scene
        .splat_data
        .objects
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            scene
                .object_meta
                .get(*i)
                .map(|m| m.hidden)
                .unwrap_or(false)
        })
        .map(|(_, o)| (o.start, o.end))
        .collect();

    let mut erased = 0;
    for splat in found {
        let i = splat.index;
        if hidden
            .iter()
            .any(|&(a, b)| (i as u32) >= a && (i as u32) <= b)
        {
            continue;
        }
        let s = &mut scene.splat_data.splats[i];
        if s.opacity > 0.0 {
            changes.push((i, s.opacity));
            s.opacity = 0.0;
            erased += 1;
        }
    }
    erased
}

/// Slice the target object with a plane, honoring the slice configuration:
/// either partition it into two separate objects and push them apart, or
/// erase everything on the positive side of the plane.
fn perform_slice(
    scene: &Rc<RefCell<Scene>>,
    renderer: &Rc<RefCell<Renderer>>,
    camera: &Rc<RefCell<Camera>>,
    settings: &Rc<RefCell<Settings>>,
    plane_point: glm::Vec3,
    plane_normal: glm::Vec3,
    width: i32,
    height: i32,
) {
    let settings_b = settings.borrow();
    let mut scene_mut = scene.borrow_mut();

    let target = if settings_b.slice_target_selected {
        match scene_mut.gizmo.target_object {
            Some(GizmoTarget::Splat(i)) => i,
            _ => 0,
        }
    } else {
        0
    };
    if target >= scene_mut.splat_data.objects.len() {
        return;
    }

    if settings_b.slice_mode == 1 {
        // Cut & erase one side of the plane (undoable).
        let obj = scene_mut.splat_data.objects[target];
        let end = (obj.end as usize).min(scene_mut.splat_data.splats.len().saturating_sub(1));
        let mut changes = Vec::new();
        for i in obj.start as usize..=end {
            let s = &mut scene_mut.splat_data.splats[i];
            if s.opacity > 0.0
                && glm::dot(&(vec3(s.x, s.y, s.z) - plane_point), &plane_normal) >= 0.0
            {
                changes.push((i, s.opacity));
                s.opacity = 0.0;
            }
        }
        if changes.is_empty() {
            return;
        }
        scene_mut.push_undo(EditOp::Erase(changes));
        renderer
            .borrow()
            .update_opacity_texture(&scene_mut, 0)
            .ok();
        return;
    }

    // Split apart: partition the object's splats in two along the plane and
    // separate the halves.
    let separation = settings_b.slice_separation;
    let new_idx = match scene_mut.splat_data.partition_object(target, |s| {
        glm::dot(&(vec3(s.x, s.y, s.z) - plane_point), &plane_normal) >= 0.0
    }) {
        Some(i) => i,
        None => return,
    };
    if separation > 0.0 {
        scene_mut
            .splat_data
            .translate_object(target, -plane_normal * separation * 0.5);
        scene_mut
            .splat_data
            .translate_object(new_idx, plane_normal * separation * 0.5);
    }
    scene_mut.undo_stack.clear(); // splat order was remapped
    scene_mut.sync_object_meta();
    let base = scene_mut.object_meta[target].name.clone();
    scene_mut.object_meta[new_idx].name = format!("{} (cut)", base);
    for meta in scene_mut.object_meta.iter_mut() {
        meta.centroid_valid = false;
    }

    // Full refresh: octree, debug lines, depth sort, textures.
    if scene_mut.splat_data.splats.len() < 5_000_000 {
        scene_mut.recalculate_octtree();
        scene_mut.octree_dirty = false;
    } else {
        scene_mut.octree_dirty = true;
    }
    scene_mut.redraw_from_oct_tree(settings_b.only_show_clicks);
    let (_vm, vpm) = camera.borrow().get_vm_and_vpm(width, height);
    let indices = scene_mut.splat_data.sort_splats_based_on_depth(vpm);
    let renderer_b = renderer.borrow();
    renderer_b.update_splat_indices(&indices);
    renderer_b.update_webgl_textures(&scene_mut, 0).ok();

    // Select the new half so it can immediately be moved with the gizmo.
    scene_mut.select_target(GizmoTarget::Splat(new_idx));
}

#[allow(non_snake_case)]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();
    // let scene_name = "Shahan_03_id01-30000";
    // let scene_name = "E7_01_id01-30000";
    // let scene_name = "corn";
    // let scene_name = "socratica_01_edited";
    // let scene_name = "Week-09-Sat-Nov-16-2024";
    // let scene_name = "sci_01";
    // let scene_name = "sci_01";
    // let scene_name = "icon_01";
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
        None => String::from("https://zimpmodels.s3.us-east-2.amazonaws.com/splats/v2/Shahan_03_id01-30000.cleaned.gsz"),
        // None => String::from("http://127.0.0.1:5502/splats/soc_01_polycam.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/sci_01_edited.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/Shahan_03_id01-30000.cleaned.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/apple.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/apple_rotate.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/watermelon.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/pomegranate.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/pomegranate_simplified.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/apple_simplified.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/apple_voxel.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/apple_voxel_medium.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/orange_extra_full.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/Shahan_03_id01-30000.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/cake.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/orange.rkyv"),
        // None => String::from("http://127.0.0.1:5502/splats/ninja/bread.rkyv"),
    };
    // let scene_name = "soc_02_edited";
    let mut splat: SplatData = SplatData::new_from_url(&scene_url).await;
    let scene = Rc::new(RefCell::new(Scene::new(splat)));

    // let cube_mesh = MeshData::new(
    //     scene_geo::CUBE_VERTICES.to_vec(),
    //     scene_geo::CUBE_INDICES.to_vec(),
    //     scene_geo::CUBE_COLORS.to_vec(),
    //     scene_geo::CUBE_NORMALS.to_vec(),
    // );
    // let cube_object = SceneObject::new(
    //     cube_mesh.clone(),
    //     vec3(-0.0, -2.0, -0.2),
    //     vec3(0.0, 0.0, 0.0),
    //     // vec3(1.0, 0.1, 0.1),
    //     vec3(0.1, 0.1, 0.1),
    // );
    // scene.borrow_mut().objects.push(cube_object);
    // let cube_object_2 = SceneObject::new_cube(vec3(2.0, -2.0, -0.25), 0.1, vec3(0.0, 1.0, 0.1));
    // scene.borrow_mut().objects.push(cube_object_2);
    // let teapot_mesh =
    //     obj_reader::read_obj("https://zimpmodels.s3.us-east-2.amazonaws.com/splats/teapot.obj")
    //         .await;
    // let teapot_object = SceneObject::new(
    //     teapot_mesh,
    //     vec3(0.2, -0.2, 0.0),
    //     vec3(3.15, 0.0, 0.0),
    //     vec3(0.01, 0.01, 0.01),
    // );
    // scene.borrow_mut().objects.push(teapot_object);

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
        eraser_preview: true,
        slice_separation: 0.5,
        slice_target_selected: false,
        slice_mode: 0,
    };
    let blending_param = params.get("blending");
    if let Some(blending_param) = blending_param {
        settings.do_blending = blending_param == "true";
    }
    let settings_ref = Rc::new(RefCell::new(settings));
    scene
        .borrow_mut()
        .redraw_from_oct_tree(settings_ref.clone().borrow_mut().only_show_clicks);

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

    // Get camera parameters from URL if available
    let default_camera_pos = vec3(-6.679095, 3.44607938, -0.32618168);
    let default_camera_rot = vec2(-0.13400005, -1.5560011);

    let camera_pos;
    let camera_rot;

    if let Some(camera_str) = params.get("camera") {
        match serde_json::from_str::<serde_json::Value>(&camera_str) {
            Ok(camera_json) => {
                if let (Some(pos_arr), Some(rot_arr)) = (
                    camera_json.get("pos").and_then(|p| p.as_array()),
                    camera_json.get("rot").and_then(|r| r.as_array()),
                ) {
                    // Parse position array
                    if pos_arr.len() >= 3 {
                        let pos_x = pos_arr[0].as_f64().unwrap_or(-6.679095) as f32;
                        let pos_y = pos_arr[1].as_f64().unwrap_or(0.14607938) as f32;
                        let pos_z = pos_arr[2].as_f64().unwrap_or(-0.32618168) as f32;
                        camera_pos = vec3(pos_x, pos_y, pos_z);
                    } else {
                        camera_pos = default_camera_pos;
                    }

                    // Parse rotation array
                    if rot_arr.len() >= 2 {
                        let rot_x = rot_arr[0].as_f64().unwrap_or(-0.13400005) as f32;
                        let rot_y = rot_arr[1].as_f64().unwrap_or(-1.5560011) as f32;
                        camera_rot = vec2(rot_x, rot_y);
                    } else {
                        camera_rot = default_camera_rot;
                    }
                } else {
                    camera_pos = default_camera_pos;
                    camera_rot = default_camera_rot;
                }
            }
            Err(_) => {
                camera_pos = default_camera_pos;
                camera_rot = default_camera_rot;
            }
        }
    } else {
        camera_pos = default_camera_pos;
        camera_rot = default_camera_rot;
    }

    // Create the camera with the position and rotation
    let camera = Rc::new(RefCell::new(Camera::new(camera_pos, camera_rot)));
    Camera::setup_mouse_events(&camera.clone(), &canvas, &document, &scene)?;

    // The Shahan model and teapot mesh are only needed if their buttons are
    // clicked, so they are fetched lazily on first use instead of blocking
    // startup with two extra downloads.
    setup_button_callbacks(
        scene.clone(),
        &renderer.clone(),
        settings_ref.clone(),
        camera.clone(),
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

    // Stores the first mouse position when the user is drawing a cutting line. We key this off of the
    // "p" key being held (for "plane"), so: hold the "p" key, click-drag to draw the line, then
    // release the mouse button to perform the split.
    let line_draw_start: Rc<RefCell<Option<(i32, i32)>>> = Rc::new(RefCell::new(None));

    // For click-vs-drag detection (a short click selects the splat object
    // under the cursor; a drag rotates the camera).
    let mousedown_pos: Rc<RefCell<(i32, i32)>> = Rc::new(RefCell::new((0, 0)));
    let mousedown_hit_mesh: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    // Opacity changes of the current erase stroke, grouped into one undo op.
    let erase_group: Rc<RefCell<Vec<(usize, f32)>>> = Rc::new(RefCell::new(Vec::new()));

    let scene_clone = scene.clone();
    let camera_clone = camera.clone();
    let renderer_clone = renderer.clone();
    let line_draw_start_clone = line_draw_start.clone();
    let keys_pressed_click = keys_pressed.clone();
    let mousedown_pos_clone = mousedown_pos.clone();
    let mousedown_hit_clone = mousedown_hit_mesh.clone();
    let click_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut scene = scene_clone.borrow_mut();
        let mut state = click_state_clone.borrow_mut();
        let renderer = renderer_clone.borrow();
        state.clicked = true;
        state.dragging = true;
        state.x = e.client_x();
        state.y = e.client_y();
        state.button = e.button();
        *mousedown_pos_clone.borrow_mut() = (state.x, state.y);
        *mousedown_hit_clone.borrow_mut() = false;

        // If the user is holding the "p" key, we start recording the line for plane splitting
        if keys_pressed_click.borrow().contains("p") {
            *line_draw_start_clone.borrow_mut() = Some((state.x, state.y));
            // Prevent camera rotation while drawing the cutting plane; use limited scope to release borrow early
            {
                let mut cam_mut = camera_clone.borrow_mut();
                cam_mut.is_dragging = false;
            }
        }

        let (vm, vpm) = camera_clone.borrow().get_vm_and_vpm(width, height);
        let (index, is_gizmo, hit_object) =
            renderer.get_at_mouse_position(width, height, state.x, state.y, vpm, vm, &scene);
        if hit_object {
            *mousedown_hit_clone.borrow_mut() = true;
            if !is_gizmo {
                scene.update_gizmo_position(index);
                let _ = scene.end_gizmo_drag();
                editor::refresh_editor_panels();
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
        }
        // Deselection now happens on mouseup, and only for real clicks —
        // starting a camera drag no longer hides the gizmo.
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mousedown", click_cb.as_ref().unchecked_ref())?;
    click_cb.forget();

    let click_state_move = click_state.clone();
    let scene_clone = scene.clone();
    let camera_clone = camera.clone();
    let line_draw_start_move = line_draw_start.clone();
    let keys_pressed_move = keys_pressed.clone();
    let mousemove_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let mut state = click_state_move.borrow_mut();
        // Always track the cursor so the eraser brush preview can follow it
        // even when no button is held.
        state.x = e.client_x();
        state.y = e.client_y();
        if state.dragging {
            state.clicked = true;

            // Show slice line if in slice mode and we have a start point
            if keys_pressed_move.borrow().contains("p") {
                if let Some((start_x, start_y)) = *line_draw_start_move.borrow() {
                    show_slice_line(start_x, start_y, state.x, state.y);
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())?;
    mousemove_cb.forget();

    let click_state_up = click_state.clone();
    let scene_clone = scene.clone();
    let camera_clone_up = camera.clone();
    let renderer_clone_up = renderer.clone();
    let settings_clone_for_split = settings_ref.clone();
    let line_draw_start_up = line_draw_start.clone();
    let keys_pressed_mouseup = keys_pressed.clone();
    let erase_group_up = erase_group.clone();
    let mousedown_pos_up = mousedown_pos.clone();
    let mousedown_hit_up = mousedown_hit_mesh.clone();
    let mouseup_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        {
            let mut state = click_state_up.borrow_mut();
            state.dragging = false;
        }

        let was_dragging_gizmo = scene_clone.borrow().gizmo.is_dragging;

        // End the gizmo drag; if a splat object was moved, its translation
        // was just baked into the CPU positions — refresh the GPU copies.
        let baked = scene_clone.borrow_mut().end_gizmo_drag();
        if baked.is_some() {
            let renderer = renderer_clone_up.borrow();
            renderer
                .update_position_texture(&scene_clone.borrow(), 0)
                .ok();
            let (_vm, vpm) = camera_clone_up.borrow().get_vm_and_vpm(width, height);
            let indices = scene_clone
                .borrow_mut()
                .splat_data
                .sort_splats_based_on_depth(vpm);
            renderer.update_splat_indices(&indices);
            editor::refresh_editor_panels();
        }

        // Close out an erase stroke as one undoable operation.
        {
            let mut group = erase_group_up.borrow_mut();
            if !group.is_empty() {
                let changes = std::mem::take(&mut *group);
                scene_clone.borrow_mut().push_undo(EditOp::Erase(changes));
                editor::refresh_editor_panels();
            }
        }

        // A short click (no drag, no tool key held) selects the splat object
        // under the cursor, or deselects when clicking empty space.
        {
            let (dx, dy) = {
                let down = mousedown_pos_up.borrow();
                (e.client_x() - down.0, e.client_y() - down.1)
            };
            let keys = keys_pressed_mouseup.borrow();
            let is_click = dx * dx + dy * dy <= 36;
            if is_click
                && !was_dragging_gizmo
                && !*mousedown_hit_up.borrow()
                && !keys.contains("Alt")
                && !keys.contains("p")
            {
                let ndc_x = (e.client_x() as f32 / width as f32) * 2.0 - 1.0;
                let ndc_y = 1.0 - (e.client_y() as f32 / height as f32) * 2.0;
                let cam = camera_clone_up.borrow();
                let (ray_origin, ray_direction) =
                    cam.get_ray_origin_and_direction(width, height, ndc_x, ndc_y);
                let mut scene_mut = scene_clone.borrow_mut();
                match scene_mut.pick_splat(ray_origin, ray_direction) {
                    Some(splat_idx) => {
                        if let Some(obj_idx) =
                            scene_mut.splat_data.object_containing(splat_idx)
                        {
                            scene_mut.select_target(GizmoTarget::Splat(obj_idx));
                        }
                    }
                    None => scene_mut.hide_gizmo(),
                }
                drop(scene_mut);
                editor::refresh_editor_panels();
            }
        }

        // Hide slice line when mouse is released
        hide_slice_line();

        // If we were in "plane draw" mode (p key held) and had a start point recorded, perform the split now.
        if let Some((sx, sy)) = line_draw_start_up.borrow_mut().take() {
            if keys_pressed_mouseup.borrow().contains("p") {
                let ex = e.client_x();
                let ey = e.client_y();

                // Convert the two screen points to NDC
                let ndc_sx = (sx as f32 / width as f32) * 2.0 - 1.0;
                let ndc_sy = 1.0 - (sy as f32 / height as f32) * 2.0;
                let ndc_ex = (ex as f32 / width as f32) * 2.0 - 1.0;
                let ndc_ey = 1.0 - (ey as f32 / height as f32) * 2.0;

                let cam = camera_clone_up.borrow();
                let (_ray_origin_s, ray_dir_s) =
                    cam.get_ray_origin_and_direction(width, height, ndc_sx, ndc_sy);
                let (ray_origin_e, ray_dir_e) =
                    cam.get_ray_origin_and_direction(width, height, ndc_ex, ndc_ey);

                // Pick a reasonably sized t to get points in front of the camera
                let t = 3.0f32;
                let p1 = _ray_origin_s + ray_dir_s * t;
                let p2 = ray_origin_e + ray_dir_e * t;

                // Primary plane normal: perpendicular to the two rays that go through the screen points.
                let mut plane_normal = glm::cross(&ray_dir_s, &ray_dir_e);

                // If the user drew a very short line the above cross product can be close to zero, fall back to the
                // previous method that uses camera-forward and the 3-D line direction.
                if glm::length(&plane_normal) < 1e-5 {
                    let line_dir = glm::normalize(&(p2 - p1));
                    let (_ray_origin_c, cam_forward) =
                        cam.get_ray_origin_and_direction(width, height, 0.0, 0.0);
                    plane_normal = glm::cross(&cam_forward, &line_dir);
                }

                // Normalise and ensure it isn't the zero vector.
                if glm::length(&plane_normal) > 1e-5 {
                    let plane_normal = glm::normalize(&plane_normal);

                    show_loading_overlay("Slicing…");

                    let scene_op = scene_clone.clone();
                    let renderer_op = renderer_clone_up.clone();
                    let camera_op = camera_clone_up.clone();
                    let settings_op = settings_clone_for_split.clone();

                    let slice_operation = Closure::wrap(Box::new(move || {
                        perform_slice(
                            &scene_op,
                            &renderer_op,
                            &camera_op,
                            &settings_op,
                            p1,
                            plane_normal,
                            width,
                            height,
                        );
                        hide_loading_overlay();
                        editor::refresh_editor_panels();
                    }) as Box<dyn FnMut()>);

                    set_timeout(&slice_operation, 60);
                    slice_operation.forget();
                } else {
                    // Hide slice line if plane calculation failed
                    hide_slice_line();
                }
            }
        }
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

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    // let mut orbit_angle = 0.0f32;
    // let orbit_radius = 0.9f32;
    // let orbit_height = 0.2f32;
    // let orbit_speed = 0.005f32;
    renderer.borrow_mut().bind_splat_textures(0);

    // Clone document and camera for the animation frame closure
    let document_for_loop = document.clone();
    let camera_for_loop = camera.clone();
    let last_sort_cam_rot = Rc::new(RefCell::new(vec2(std::f32::NAN, std::f32::NAN)));
    let erase_group_loop = erase_group.clone();

    // Register the live handles with the editor API so the HTML UI can
    // drive the scene (object panel, add-object library, segmentation, …).
    editor::register_app(editor::App {
        scene: scene.clone(),
        renderer: renderer.clone(),
        camera: camera.clone(),
        settings: settings_ref.clone(),
        canvas: canvas.clone(),
        width,
        height,
    });

    *g.borrow_mut() = Some(Closure::new(move || {
        let _timer = Timer::new("main loop");
        let mut cam_mut = camera_for_loop.borrow_mut();
        let settings = settings_ref.clone();
        // // update orbit angle and camera position
        // orbit_angle += orbit_speed;
        // cam_mut.pos.x = orbit_radius * orbit_angle.cos();
        // cam_mut.pos.z = orbit_radius * orbit_angle.sin();
        // cam_mut.pos.y = orbit_height;

        // // make the camera look at the center
        // cam_mut.rot.x = -0.13400005; // Keep original x rotation
        // cam_mut.rot.y = orbit_angle + std::f32::consts::PI / 2.0; // Make camera face center

        // scene.borrow_mut().objects[0].recalculate_min_max();
        if settings.borrow().move_down {
            scene.borrow_mut().move_down();
        }

        cam_mut.update_translation_from_keys(&keys_pressed.borrow());
        let (vm, vpm) = cam_mut.get_vm_and_vpm(width, height);

        // Handle slice mode UI updates
        static mut SLICE_MODE_ACTIVE: bool = false;
        let p_key_pressed = keys_pressed.borrow().contains("p");
        unsafe {
            if p_key_pressed && !SLICE_MODE_ACTIVE {
                show_slice_mode();
                SLICE_MODE_ACTIVE = true;
            } else if !p_key_pressed && SLICE_MODE_ACTIVE {
                hide_slice_mode();
                SLICE_MODE_ACTIVE = false;
            }
        }

        // Handle delete mode UI updates
        static mut DELETE_MODE_ACTIVE: bool = false;
        let alt_key_pressed = keys_pressed.borrow().contains("Alt");
        unsafe {
            if alt_key_pressed && !DELETE_MODE_ACTIVE {
                show_delete_mode();
                DELETE_MODE_ACTIVE = true;
            } else if !alt_key_pressed && DELETE_MODE_ACTIVE {
                hide_delete_mode();
                DELETE_MODE_ACTIVE = false;
            }
        }

        // Update camera position display if it exists
        if let Some(pos_div) = document_for_loop.get_element_by_id("camera-position") {
            if let Ok(pos_div) = pos_div.dyn_into::<HtmlElement>() {
                // Format the camera position and rotation as JSON
                let pos_str = format!(
                    "[{:.6}, {:.6}, {:.6}]",
                    cam_mut.pos.x, cam_mut.pos.y, cam_mut.pos.z
                );
                let rot_str = format!("[{:.6}, {:.6}]", cam_mut.rot.x, cam_mut.rot.y);

                let json_str = format!("camera: {{ pos: {}, rot: {} }}", pos_str, rot_str);

                pos_div.set_inner_text(&json_str);
            }
        }

        // ---- Eraser: hover brush preview + continuous erase while held ----
        {
            let (mx, my) = {
                let s = click_state.borrow();
                (s.x, s.y)
            };
            if alt_key_pressed {
                let center = eraser_center_from_mouse(
                    mx,
                    my,
                    width,
                    height,
                    &cam_mut,
                    &mut scene.borrow_mut(),
                );
                {
                    let mut scene_mut = scene.borrow_mut();
                    match center {
                        Some(c) => {
                            scene_mut.eraser.center = c;
                            scene_mut.eraser.active = settings.borrow().eraser_preview;
                        }
                        None => scene_mut.eraser.active = false,
                    }
                }
                match center {
                    Some(c) => {
                        let radius = scene.borrow().eraser.radius;
                        // Live count of what a click would erase.
                        let count = {
                            let mut scene_mut = scene.borrow_mut();
                            scene_mut.ensure_octree();
                            scene_mut
                                .find_splats_in_radius(c, radius)
                                .iter()
                                .filter(|s| s.opacity > 0.02)
                                .count()
                        };
                        update_erase_count(count as u32);

                        if click_state.borrow().dragging {
                            let mut scene_mut = scene.borrow_mut();
                            let erased = erase_at(
                                &mut scene_mut,
                                c,
                                radius,
                                &mut erase_group_loop.borrow_mut(),
                            );
                            if erased > 0 {
                                renderer
                                    .borrow()
                                    .update_opacity_texture(&scene_mut, 0)
                                    .ok();
                            }
                        }
                    }
                    None => update_erase_count(0),
                }
            } else if scene.borrow().eraser.active {
                scene.borrow_mut().eraser.active = false;
            }
        }

        if click_state.borrow().clicked {
            let state = click_state.borrow();
            if !click_state.borrow().dragging {
                drop(state);
                click_state.borrow_mut().clicked = false;
                click_state.borrow_mut().dragging = false;
            } else {
                if scene.borrow().gizmo.is_dragging {
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

        let rot_threshold: f32 = 0.3; // ~2.8 degrees in radians
        let mut do_sort_now = false;
        if settings.borrow().do_sorting {
            let last_rot = last_sort_cam_rot.borrow().clone();
            if last_rot.x.is_nan() {
                // First frame (or after reset) – force a sort
                do_sort_now = true;
            } else {
                let rot_diff = glm::distance(&cam_mut.rot, &last_rot);
                if rot_diff > rot_threshold {
                    do_sort_now = true;
                }
            }
        }

        if do_sort_now {
            let splat_indices = scene
                .borrow_mut()
                .splat_data
                .sort_splats_based_on_depth(vpm);
            renderer.borrow_mut().update_splat_indices(&splat_indices);
            // Remember the rotation at which we just sorted
            *last_sort_cam_rot.borrow_mut() = cam_mut.rot;
        }

        // renderer.borrow_mut().bind_splat_textures(0);
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
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    // Add print camera button functionality
    let print_camera_btn = document.get_element_by_id("print-camera-btn");
    if let Some(btn) = print_camera_btn {
        let btn = btn.dyn_into::<HtmlElement>()?;
        let cam_clone = camera.clone();
        let document_clone = document.clone();

        let print_callback = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            let cam = cam_clone.borrow();

            // Format the camera position and rotation
            let pos_str = format!("[{:.6}, {:.6}, {:.6}]", cam.pos.x, cam.pos.y, cam.pos.z);
            let rot_str = format!("[{:.6}, {:.6}]", cam.rot.x, cam.rot.y);

            // Create a JSON representation
            let json_str = format!("camera: {{ pos: {}, rot: {} }}", pos_str, rot_str);

            // Update the display in the DOM
            if let Some(pos_div) = document_clone.get_element_by_id("camera-position") {
                let pos_div = pos_div.dyn_into::<HtmlElement>().unwrap();
                pos_div.set_inner_html(&json_str);
            }

            // Log to console
            log!("Current camera: {}", json_str);
        }) as Box<dyn FnMut(_)>);

        btn.set_onclick(Some(print_callback.as_ref().unchecked_ref()));
        print_callback.forget();
    }

    // Signal that model loading and initialization is complete
    set_model_loading(false, "");

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
    _camera: Rc<RefCell<Camera>>,
    _width: i32,
    _height: i32,
) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Calculate Shadows button (optional in the DOM)
    if let Some(el) = document.get_element_by_id("calculate-shadows-btn") {
        let shadows_btn = el.dyn_into::<HtmlElement>()?;
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
    }

    // Recalculate Octtree button (optional in the DOM)
    if let Some(el) = document.get_element_by_id("recalculate-octtree-btn") {
        let recalculate_octtree_btn = el.dyn_into::<HtmlElement>()?;
        let scene_clone = scene.clone();
        let settings_clone = settings.clone();
        let recalculate_octtree_callback =
            Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                let settings = settings_clone.borrow();
                let mut scene = scene_clone.borrow_mut();
                scene.recalculate_octtree();
                scene.redraw_from_oct_tree(settings.only_show_clicks);
            }) as Box<dyn FnMut(_)>);
        recalculate_octtree_btn
            .set_onclick(Some(recalculate_octtree_callback.as_ref().unchecked_ref()));
        recalculate_octtree_callback.forget();
    }

    // The old "Add Shahan" / "Add Teapot" / "Split object" buttons are
    // superseded by the editor API (see src/editor.rs): the Add-object
    // library and the object panel in index.html call the exported
    // editor_* functions directly.

    Ok(())
}
