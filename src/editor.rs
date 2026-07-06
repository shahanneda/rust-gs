//! Editor API exposed to JavaScript.
//!
//! `start()` registers the live app handles here; the `editor_*` functions
//! exported below are called from the HTML UI (object panel, add-object
//! library, selection card, eraser/slice popovers, SAM segmentation).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use nalgebra_glm as glm;
use nalgebra_glm::{vec3, Vec3};
use wasm_bindgen::prelude::*;

use crate::camera::Camera;
use crate::data_objects::{MeshData, SplatData};
use crate::gizmo::GizmoTarget;
use crate::log;
use crate::obj_reader;
use crate::renderer::Renderer;
use crate::scene::{EditOp, Scene, SplatObjectMeta};
use crate::scene_geo;
use crate::scene_object::SceneObject;
use crate::web::Settings;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "showLoadingOverlay")]
    fn show_loading_overlay(message: &str);
    #[wasm_bindgen(js_name = "hideLoadingOverlay")]
    fn hide_loading_overlay();
    #[wasm_bindgen(js_name = "refreshEditorPanels")]
    pub fn refresh_editor_panels();
    #[wasm_bindgen(js_name = "showToast")]
    fn show_toast(message: &str, ms: i32);
}

pub struct App {
    pub scene: Rc<RefCell<Scene>>,
    pub renderer: Rc<RefCell<Renderer>>,
    pub camera: Rc<RefCell<Camera>>,
    pub settings: Rc<RefCell<Settings>>,
    pub canvas: web_sys::HtmlCanvasElement,
    pub width: i32,
    pub height: i32,
}

thread_local! {
    static APP: RefCell<Option<App>> = RefCell::new(None);
    static SPLAT_CACHE: RefCell<HashMap<String, Rc<SplatData>>> = RefCell::new(HashMap::new());
}

pub fn register_app(app: App) {
    APP.with(|a| *a.borrow_mut() = Some(app));
}

fn with_app<R>(f: impl FnOnce(&App) -> R) -> Option<R> {
    APP.with(|a| a.borrow().as_ref().map(f))
}

/// Full refresh after any operation that changed splat geometry or object
/// ranges: octree, debug lines, depth sort and all five data textures.
fn refresh_all(app: &App) {
    let mut scene = app.scene.borrow_mut();
    let renderer = app.renderer.borrow();
    let camera = app.camera.borrow();
    let settings = app.settings.borrow();

    scene.sync_object_meta();
    if scene.splat_data.splats.len() < 5_000_000 {
        scene.recalculate_octtree();
        scene.octree_dirty = false;
    } else {
        scene.octree_dirty = true;
    }
    scene.redraw_from_oct_tree(settings.only_show_clicks);
    let (_vm, vpm) = camera.get_vm_and_vpm(app.width, app.height);
    let indices = scene.splat_data.sort_splats_based_on_depth(vpm);
    renderer.update_splat_indices(&indices);
    renderer
        .update_webgl_textures(&scene, 0)
        .expect("texture refresh failed");
}

fn placement_in_front_of_camera(app: &App) -> Vec3 {
    let camera = app.camera.borrow();
    let (origin, direction) =
        camera.get_ray_origin_and_direction(app.width, app.height, 0.0, -0.15);
    origin + direction * 3.0
}

// ============================================================
// Object listing / hierarchy
// ============================================================

#[wasm_bindgen]
pub fn editor_get_objects() -> String {
    with_app(|app| {
        let scene = app.scene.borrow();
        let selected = scene.gizmo.target_object;
        let mut items = Vec::new();
        for (i, o) in scene.splat_data.objects.iter().enumerate() {
            if o.end < o.start {
                continue; // emptied-out object
            }
            let meta = scene.object_meta.get(i);
            items.push(serde_json::json!({
                "kind": "splat",
                "idx": i,
                "name": meta.map(|m| m.name.clone()).unwrap_or_else(|| format!("Object {}", i)),
                "count": o.end - o.start + 1,
                "hidden": meta.map(|m| m.hidden).unwrap_or(false),
                "selected": selected == Some(GizmoTarget::Splat(i)),
            }));
        }
        for (i, o) in scene.objects.iter().enumerate() {
            items.push(serde_json::json!({
                "kind": "mesh",
                "idx": i,
                "name": o.name.clone(),
                "count": o.mesh_data.vertices.len() / 3,
                "hidden": o.hidden,
                "selected": selected == Some(GizmoTarget::Mesh(i)),
            }));
        }
        serde_json::json!({
            "items": items,
            "undoDepth": scene.undo_stack.len(),
        })
        .to_string()
    })
    .unwrap_or_else(|| String::from("{\"items\":[]}"))
}

fn parse_target(kind: &str, idx: usize) -> GizmoTarget {
    if kind == "mesh" {
        GizmoTarget::Mesh(idx)
    } else {
        GizmoTarget::Splat(idx)
    }
}

#[wasm_bindgen]
pub fn editor_select(kind: String, idx: usize) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        scene.select_target(parse_target(&kind, idx));
    });
}

#[wasm_bindgen]
pub fn editor_deselect() {
    with_app(|app| {
        app.scene.borrow_mut().hide_gizmo();
    });
}

#[wasm_bindgen]
pub fn editor_set_hidden(kind: String, idx: usize, hidden: bool) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        match parse_target(&kind, idx) {
            GizmoTarget::Splat(i) => {
                if let Some(meta) = scene.object_meta.get_mut(i) {
                    meta.hidden = hidden;
                }
            }
            GizmoTarget::Mesh(i) => {
                if let Some(o) = scene.objects.get_mut(i) {
                    o.hidden = hidden;
                }
            }
        }
    });
}

#[wasm_bindgen]
pub fn editor_rename(kind: String, idx: usize, name: String) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        match parse_target(&kind, idx) {
            GizmoTarget::Splat(i) => {
                if let Some(meta) = scene.object_meta.get_mut(i) {
                    meta.name = name.clone();
                }
            }
            GizmoTarget::Mesh(i) => {
                if let Some(o) = scene.objects.get_mut(i) {
                    o.name = name.clone();
                }
            }
        }
    });
}

#[wasm_bindgen]
pub fn editor_delete(kind: String, idx: usize) -> bool {
    with_app(|app| {
        {
            let mut scene = app.scene.borrow_mut();
            match parse_target(&kind, idx) {
                GizmoTarget::Splat(i) => {
                    if scene.splat_data.objects.len() <= 1 || i >= scene.splat_data.objects.len() {
                        return false;
                    }
                    scene.splat_data.remove_object(i);
                    scene.object_meta.remove(i);
                    // Splat indices shifted: previous undo entries are invalid.
                    scene.undo_stack.clear();
                    match scene.gizmo.target_object {
                        Some(GizmoTarget::Splat(s)) if s == i => scene.gizmo.target_object = None,
                        Some(GizmoTarget::Splat(s)) if s > i => {
                            scene.gizmo.target_object = Some(GizmoTarget::Splat(s - 1))
                        }
                        _ => {}
                    }
                    for meta in scene.object_meta.iter_mut() {
                        meta.centroid_valid = false;
                    }
                }
                GizmoTarget::Mesh(i) => {
                    if i >= scene.objects.len() {
                        return false;
                    }
                    scene.objects.remove(i);
                    match scene.gizmo.target_object {
                        Some(GizmoTarget::Mesh(s)) if s == i => scene.gizmo.target_object = None,
                        Some(GizmoTarget::Mesh(s)) if s > i => {
                            scene.gizmo.target_object = Some(GizmoTarget::Mesh(s - 1))
                        }
                        _ => {}
                    }
                    return true; // meshes don't need a splat refresh
                }
            }
        }
        refresh_all(app);
        true
    })
    .unwrap_or(false)
}

#[wasm_bindgen]
pub fn editor_duplicate(kind: String, idx: usize) -> bool {
    with_app(|app| {
        {
            let mut scene = app.scene.borrow_mut();
            match parse_target(&kind, idx) {
                GizmoTarget::Splat(i) => {
                    if i >= scene.splat_data.objects.len() {
                        return false;
                    }
                    let new_idx = scene.splat_data.duplicate_object(i);
                    // Nudge the copy so it doesn't perfectly overlap.
                    scene
                        .splat_data
                        .translate_object(new_idx, vec3(0.3, 0.0, 0.3));
                    let base_name = scene
                        .object_meta
                        .get(i)
                        .map(|m| m.name.clone())
                        .unwrap_or_else(|| String::from("Object"));
                    scene
                        .object_meta
                        .push(SplatObjectMeta::named(format!("{} copy", base_name)));
                    scene.gizmo.target_object = Some(GizmoTarget::Splat(new_idx));
                }
                GizmoTarget::Mesh(i) => {
                    if i >= scene.objects.len() {
                        return false;
                    }
                    let mut copy = scene.objects[i].clone();
                    copy.pos += vec3(0.3, 0.0, 0.3);
                    copy.name = format!("{} copy", copy.name);
                    scene.objects.push(copy);
                    let new_idx = scene.objects.len() - 1;
                    scene.gizmo.target_object = Some(GizmoTarget::Mesh(new_idx));
                    return true;
                }
            }
        }
        refresh_all(app);
        // Re-aim the gizmo at the (new) selection.
        {
            let mut scene = app.scene.borrow_mut();
            if let Some(t) = scene.gizmo.target_object {
                scene.select_target(t);
            }
        }
        true
    })
    .unwrap_or(false)
}

// ============================================================
// Color editing
// ============================================================

/// Live (shader-only) tint preview for a splat object.
#[wasm_bindgen]
pub fn editor_set_tint_preview(idx: usize, r: f32, g: f32, b: f32, strength: f32) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        if let Some(meta) = scene.object_meta.get_mut(idx) {
            meta.tint = vec3(r, g, b);
            meta.tint_strength = strength.clamp(0.0, 1.0);
        }
    });
}

/// Bake the current preview tint into the splat colors (undoable).
#[wasm_bindgen]
pub fn editor_apply_tint(idx: usize) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        let (tint, strength) = match scene.object_meta.get(idx) {
            Some(m) if m.tint_strength > 0.001 => (m.tint, m.tint_strength),
            _ => return,
        };
        let old = scene.splat_data.recolor_object(idx, tint, strength);
        scene.push_undo(EditOp::Recolor(old));
        if let Some(meta) = scene.object_meta.get_mut(idx) {
            meta.tint_strength = 0.0;
        }
        app.renderer
            .borrow()
            .update_color_texture(&scene, 0)
            .expect("color texture update failed");
    });
}

/// Set the flat color of a mesh object.
#[wasm_bindgen]
pub fn editor_set_mesh_color(idx: usize, r: f32, g: f32, b: f32) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        if let Some(o) = scene.objects.get_mut(idx) {
            let n = o.mesh_data.colors.len() / 3;
            for i in 0..n {
                o.mesh_data.colors[i * 3] = r;
                o.mesh_data.colors[i * 3 + 1] = g;
                o.mesh_data.colors[i * 3 + 2] = b;
            }
        }
    });
}

// ============================================================
// Eraser / slice configuration + undo
// ============================================================

#[wasm_bindgen]
pub fn editor_set_eraser_config(radius: f32, preview: bool) {
    with_app(|app| {
        app.scene.borrow_mut().eraser.radius = radius.clamp(0.02, 3.0);
        app.settings.borrow_mut().eraser_preview = preview;
    });
}

#[wasm_bindgen]
pub fn editor_set_slice_config(separation: f32, mode: u32) {
    with_app(|app| {
        let mut settings = app.settings.borrow_mut();
        settings.slice_separation = separation.clamp(0.0, 3.0);
        settings.slice_mode = mode;
    });
}

/// Switch the gizmo between "translate", "rotate" and "scale".
#[wasm_bindgen]
pub fn editor_set_gizmo_mode(mode: String) {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        scene.gizmo.mode = match mode.as_str() {
            "rotate" => crate::gizmo::GizmoMode::Rotate,
            "scale" => crate::gizmo::GizmoMode::Scale,
            _ => crate::gizmo::GizmoMode::Translate,
        };
    });
}

#[wasm_bindgen]
pub fn editor_get_gizmo_mode() -> String {
    with_app(|app| {
        match app.scene.borrow().gizmo.mode {
            crate::gizmo::GizmoMode::Rotate => "rotate",
            crate::gizmo::GizmoMode::Scale => "scale",
            crate::gizmo::GizmoMode::Translate => "translate",
        }
        .to_string()
    })
    .unwrap_or_else(|| String::from("translate"))
}

#[wasm_bindgen]
pub fn editor_undo() -> bool {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        let renderer = app.renderer.borrow();
        let op = match scene.undo_stack.pop() {
            Some(op) => op,
            None => return false,
        };
        match op {
            EditOp::Erase(changes) => {
                for (i, opacity) in changes {
                    if let Some(s) = scene.splat_data.splats.get_mut(i) {
                        s.opacity = opacity;
                    }
                }
                renderer.update_opacity_texture(&scene, 0).ok();
            }
            EditOp::Recolor(changes) => {
                for (i, rgb) in changes {
                    if let Some(s) = scene.splat_data.splats.get_mut(i) {
                        s.r = rgb[0];
                        s.g = rgb[1];
                        s.b = rgb[2];
                    }
                }
                renderer.update_color_texture(&scene, 0).ok();
            }
            EditOp::Transform {
                object,
                pivot,
                rotation,
                scale,
                translation,
            } => {
                // Inverse of p' = R·s·(p − c) + c + t in the same form:
                // pivot' = c + t, R' = R⁻¹, s' = 1/s, t' = −t.
                scene.splat_data.transform_object(
                    object,
                    pivot + translation,
                    glm::quat_inverse(&rotation),
                    1.0 / scale.max(1e-6),
                    -translation,
                );
                if let Some(meta) = scene.object_meta.get_mut(object) {
                    meta.centroid -= translation;
                }
                scene.octree_dirty = true;
                if let Some(GizmoTarget::Splat(s)) = scene.gizmo.target_object {
                    if s == object {
                        let pos = scene.splat_object_position(object);
                        scene.gizmo.update_position(pos);
                    }
                }
                renderer.update_webgl_textures(&scene, 0).ok();
            }
        }
        true
    })
    .unwrap_or(false)
}

// ============================================================
// Adding objects
// ============================================================

#[wasm_bindgen]
pub async fn editor_add_splat_from_url(url: String, name: String) -> Result<(), JsValue> {
    let cached = SPLAT_CACHE.with(|c| c.borrow().get(&url).cloned());
    let data = match cached {
        Some(d) => d,
        None => {
            show_loading_overlay(&format!("Downloading {}…", name));
            let d = Rc::new(SplatData::new_from_url(&url).await);
            SPLAT_CACHE.with(|c| c.borrow_mut().insert(url.clone(), d.clone()));
            d
        }
    };

    show_loading_overlay(&format!("Adding {} to scene…", name));
    with_app(|app| {
        {
            let mut scene = app.scene.borrow_mut();
            scene.splat_data.merge_with_other_splatdata(&data);
            let new_idx = scene.splat_data.objects.len() - 1;
            scene.sync_object_meta();
            scene.object_meta[new_idx] = SplatObjectMeta::named(name.clone());

            // Land the object in front of the camera: shift its centroid there.
            let target = placement_in_front_of_camera(app);
            let centroid = scene.splat_data.centroid_of_object(new_idx);
            scene.splat_data.translate_object(new_idx, target - centroid);
            scene.gizmo.target_object = Some(GizmoTarget::Splat(new_idx));
        }
        refresh_all(app);
        {
            let mut scene = app.scene.borrow_mut();
            if let Some(t) = scene.gizmo.target_object {
                scene.select_target(t);
            }
        }
    });
    hide_loading_overlay();
    refresh_editor_panels();
    Ok(())
}

#[wasm_bindgen]
pub async fn editor_add_mesh_from_url(url: String, name: String, scale: f32) -> Result<(), JsValue> {
    show_loading_overlay(&format!("Downloading {}…", name));
    let mesh = obj_reader::read_obj(&url).await;
    with_app(|app| {
        let pos = placement_in_front_of_camera(app);
        let mut scene = app.scene.borrow_mut();
        let mut object = SceneObject::new(
            mesh.clone(),
            pos,
            vec3(3.14, 0.0, 0.0),
            vec3(scale, scale, scale),
        );
        object.name = name.clone();
        scene.objects.push(object);
        let new_idx = scene.objects.len() - 1;
        scene.select_target(GizmoTarget::Mesh(new_idx));
    });
    hide_loading_overlay();
    refresh_editor_panels();
    Ok(())
}

#[wasm_bindgen]
pub fn editor_add_primitive(kind: String, r: f32, g: f32, b: f32, size: f32) {
    with_app(|app| {
        let pos = placement_in_front_of_camera(app);
        let mut scene = app.scene.borrow_mut();
        let mut object = match kind.as_str() {
            "sphere" => {
                let (verts, indices, normals) = scene_geo::sphere_mesh(28, 20);
                let colors = vec![0.0f32; verts.len()]
                    .chunks(3)
                    .flat_map(|_| [r, g, b])
                    .collect::<Vec<f32>>();
                SceneObject::new(
                    MeshData::new(verts, indices, colors, normals),
                    pos,
                    vec3(0.0, 0.0, 0.0),
                    vec3(size, size, size),
                )
            }
            _ => SceneObject::new_cube(pos, size * 2.0, vec3(r, g, b)),
        };
        object.name = if kind == "sphere" {
            String::from("Sphere")
        } else {
            String::from("Cube")
        };
        scene.objects.push(object);
        let new_idx = scene.objects.len() - 1;
        scene.select_target(GizmoTarget::Mesh(new_idx));
    });
    refresh_editor_panels();
}

// ============================================================
// Segmentation (SAM) support
// ============================================================

/// Render one clean frame and return its RGBA pixels (bottom-up, WebGL
/// convention — the JS side flips). Also remembers the view-projection
/// matrix so a mask computed on this frame can be projected back onto the
/// splats later, even if the camera has since moved.
#[wasm_bindgen]
pub fn editor_capture_frame() -> Vec<u8> {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        let renderer = app.renderer.borrow();
        let camera = app.camera.borrow();
        let settings = app.settings.borrow();

        let (vm, vpm) = camera.get_vm_and_vpm(app.width, app.height);
        let (np, nv) = camera.get_normal_projection_and_view_matrices(app.width, app.height);
        scene.capture_vpm = Some(vpm);
        let eraser_was_active = scene.eraser.active;
        scene.eraser.active = false;
        renderer.draw_scene(&app.canvas, &scene, vpm, vm, np, nv, &settings, true);
        scene.eraser.active = eraser_was_active;
        renderer.read_pixels_rgba(app.width, app.height)
    })
    .unwrap_or_default()
}

/// Given a segmentation mask over the captured frame (row-major, top-down,
/// one byte per pixel, non-zero = selected), find the matching splats.
/// mode 0: extract them into a new object (returns its index).
/// mode 1: erase them (undoable; returns number of erased splats).
/// Returns -1 when nothing matched.
#[wasm_bindgen]
pub fn editor_apply_mask(mask: &[u8], mask_w: u32, mask_h: u32, mode: u32) -> i32 {
    with_app(|app| {
        let vpm = match app.scene.borrow().capture_vpm {
            Some(m) => m,
            None => return -1,
        };

        let selected: HashSet<usize> = {
            let scene = app.scene.borrow();
            // Splats belonging to hidden objects can't be segmented (they
            // weren't visible in the captured frame).
            let mut hidden_ranges: Vec<(u32, u32)> = Vec::new();
            for (i, o) in scene.splat_data.objects.iter().enumerate() {
                if scene.object_meta.get(i).map(|m| m.hidden).unwrap_or(false) {
                    hidden_ranges.push((o.start, o.end));
                }
            }

            let mut hits: Vec<(usize, f32)> = Vec::new(); // (index, view depth)
            for (i, s) in scene.splat_data.splats.iter().enumerate() {
                if s.opacity <= 0.05 {
                    continue;
                }
                let iu = i as u32;
                if hidden_ranges.iter().any(|&(a, b)| iu >= a && iu <= b) {
                    continue;
                }
                let clip = vpm * glm::vec4(s.x, s.y, s.z, 1.0);
                if clip.w <= 0.0 {
                    continue;
                }
                let ndc_x = clip.x / clip.w;
                let ndc_y = clip.y / clip.w;
                let px = ((ndc_x * 0.5 + 0.5) * mask_w as f32) as i64;
                let py = ((1.0 - (ndc_y * 0.5 + 0.5)) * mask_h as f32) as i64;
                if px < 0 || py < 0 || px >= mask_w as i64 || py >= mask_h as i64 {
                    continue;
                }
                if mask[(py * mask_w as i64 + px) as usize] > 0 {
                    hits.push((i, clip.w));
                }
            }

            if hits.is_empty() {
                return -1;
            }

            // Keep only the front-most depth cluster so splats far behind
            // the clicked object (walls, floor) aren't dragged along.
            hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let mut cutoff = hits.len();
            const GAP: f32 = 0.4;
            for i in 1..hits.len() {
                if hits[i].1 - hits[i - 1].1 > GAP {
                    cutoff = i;
                    break;
                }
            }
            hits[..cutoff].iter().map(|&(i, _)| i).collect()
        };

        if mode == 1 {
            // Erase the selection.
            let count;
            {
                let mut scene = app.scene.borrow_mut();
                let mut changes = Vec::with_capacity(selected.len());
                for &i in &selected {
                    let s = &mut scene.splat_data.splats[i];
                    if s.opacity > 0.0 {
                        changes.push((i, s.opacity));
                        s.opacity = 0.0;
                    }
                }
                count = changes.len();
                scene.push_undo(EditOp::Erase(changes));
                scene.octree_dirty = true;
                app.renderer
                    .borrow()
                    .update_opacity_texture(&scene, 0)
                    .ok();
            }
            refresh_editor_panels();
            return count as i32;
        }

        // Extract to a new object.
        let new_idx = {
            let mut scene = app.scene.borrow_mut();
            let new_idx = match scene.splat_data.extract_indices_to_object(&selected) {
                Some(i) => i,
                None => return -1,
            };
            scene.undo_stack.clear(); // splat indices were remapped
            scene.sync_object_meta();
            let n = scene
                .object_meta
                .iter()
                .filter(|m| m.name.starts_with("Segment"))
                .count();
            scene.object_meta[new_idx] = SplatObjectMeta::named(format!("Segment {}", n + 1));
            for meta in scene.object_meta.iter_mut() {
                meta.centroid_valid = false;
            }
            scene.gizmo.target_object = Some(GizmoTarget::Splat(new_idx));
            new_idx
        };
        refresh_all(app);
        {
            let mut scene = app.scene.borrow_mut();
            scene.select_target(GizmoTarget::Splat(new_idx));
        }
        refresh_editor_panels();
        new_idx as i32
    })
    .unwrap_or(-1)
}

/// Number of visible splats the eraser would delete at its current position.
#[wasm_bindgen]
pub fn editor_pending_erase_count() -> u32 {
    with_app(|app| {
        let mut scene = app.scene.borrow_mut();
        if !scene.eraser.active {
            return 0;
        }
        let center = scene.eraser.center;
        let radius = scene.eraser.radius;
        scene.ensure_octree();
        scene
            .find_splats_in_radius(center, radius)
            .iter()
            .filter(|s| s.opacity > 0.02)
            .count() as u32
    })
    .unwrap_or(0)
}
