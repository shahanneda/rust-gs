use std::collections::{HashMap, HashSet};

use crate::data_objects::MeshData;
use crate::gizmo::{Gizmo, GizmoAxis, GizmoTarget};
use crate::log;
use crate::oct_tree::{OctTreeNode, OctTreeSplat};
use crate::scene_object::SceneObject;

#[cfg(target_arch = "wasm32")]
use crate::web::setCollisionDetected;

use crate::{data_objects::SplatData, oct_tree::OctTree};
use nalgebra_glm::{self as glm, Vec2};
use nalgebra_glm::{vec3, Vec3};

/// Editor-side state for one splat object (parallel to
/// `splat_data.objects`). `translation` is the live, not-yet-baked offset
/// applied in the vertex shader while a gizmo drag is in progress.
#[derive(Debug, Clone)]
pub struct SplatObjectMeta {
    pub name: String,
    pub hidden: bool,
    pub tint: Vec3,
    pub tint_strength: f32,
    pub translation: Vec3,
    pub centroid: Vec3,
    pub centroid_valid: bool,
}

impl SplatObjectMeta {
    pub fn named(name: String) -> Self {
        Self {
            name,
            hidden: false,
            tint: vec3(1.0, 1.0, 1.0),
            tint_strength: 0.0,
            translation: vec3(0.0, 0.0, 0.0),
            centroid: vec3(0.0, 0.0, 0.0),
            centroid_valid: false,
        }
    }
}

/// Live eraser-brush state, visualized by the splat shader.
pub struct EraserState {
    pub active: bool,
    pub center: Vec3,
    pub radius: f32,
}

/// One undoable edit.
pub enum EditOp {
    /// (splat index, previous opacity)
    Erase(Vec<(usize, f32)>),
    /// (splat index, previous rgb)
    Recolor(Vec<(usize, [f32; 3])>),
    /// A baked gizmo move of a splat object.
    Move { object: usize, delta: Vec3 },
}

const MAX_UNDO: usize = 64;

pub struct Scene {
    pub splat_data: SplatData,
    pub objects: Vec<SceneObject>,
    pub line_mesh: SceneObject,
    pub light_pos: Vec3,
    pub original_shadow_splat_colors: HashMap<usize, Vec3>,
    pub gizmo: Gizmo,
    pub oct_tree: OctTree,
    pub octree_dirty: bool,
    pub model_transform: glm::Mat4,
    pub object_meta: Vec<SplatObjectMeta>,
    pub eraser: EraserState,
    pub undo_stack: Vec<EditOp>,
    /// View-projection matrix of the last frame captured for segmentation.
    pub capture_vpm: Option<glm::Mat4>,
}

impl Scene {
    pub fn new(splat_data: SplatData) -> Self {
        let oct_tree = OctTree::new(&splat_data.splats);
        let mut scene = Self {
            splat_data,
            objects: Vec::new(),
            line_mesh: SceneObject::new(
                MeshData::new(vec![], vec![], vec![], vec![]),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 1.0, 1.0),
            ),
            light_pos: vec3(1.0, -3.0, 0.0),
            original_shadow_splat_colors: HashMap::new(),
            gizmo: Gizmo::new(),
            oct_tree,
            octree_dirty: false,
            model_transform: glm::Mat4::identity(),
            object_meta: Vec::new(),
            eraser: EraserState {
                active: false,
                center: vec3(0.0, 0.0, 0.0),
                radius: 0.35,
            },
            undo_stack: Vec::new(),
            capture_vpm: None,
        };
        scene.sync_object_meta();
        if let Some(meta) = scene.object_meta.get_mut(0) {
            meta.name = String::from("Scene");
        }
        scene
    }

    /// Keep `object_meta` aligned with `splat_data.objects`, appending
    /// default entries for newly created objects.
    pub fn sync_object_meta(&mut self) {
        while self.object_meta.len() < self.splat_data.objects.len() {
            let n = self.object_meta.len();
            self.object_meta
                .push(SplatObjectMeta::named(format!("Object {}", n)));
        }
        self.object_meta.truncate(self.splat_data.objects.len());
    }

    pub fn push_undo(&mut self, op: EditOp) {
        self.undo_stack.push(op);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
    }

    /// Recalculate the octree if edits invalidated it (skipped for huge
    /// scenes, where a slightly stale octree is preferable to a long stall).
    pub fn ensure_octree(&mut self) {
        if self.octree_dirty && self.splat_data.splats.len() < 5_000_000 {
            self.recalculate_octtree();
        }
        self.octree_dirty = false;
    }

    /// Centroid of a splat object plus its live (unbaked) translation.
    pub fn splat_object_position(&mut self, idx: usize) -> Vec3 {
        if !self.object_meta[idx].centroid_valid {
            self.object_meta[idx].centroid = self.splat_data.centroid_of_object(idx);
            self.object_meta[idx].centroid_valid = true;
        }
        let meta = &self.object_meta[idx];
        meta.centroid + meta.translation
    }

    /// March the ray through the scene and return the first visible splat.
    pub fn pick_splat(&mut self, ray_origin: Vec3, ray_direction: Vec3) -> Option<usize> {
        self.ensure_octree();
        let steps = 300;
        let step_size = 0.05;
        for t in 1..steps {
            let pos = ray_origin + ray_direction * (t as f32 * step_size);
            let found = self
                .oct_tree
                .find_splats_in_radius(pos, 0.06, &self.splat_data.splats);
            for splat in found {
                if splat.opacity >= 0.5 {
                    return Some(splat.index);
                }
            }
        }
        None
    }

    pub fn add_line(&mut self, start: Vec3, end: Vec3, color: Vec3) {
        self.line_mesh.mesh_data.vertices.push(start.x);
        self.line_mesh.mesh_data.vertices.push(start.y);
        self.line_mesh.mesh_data.vertices.push(start.z);

        self.line_mesh.mesh_data.vertices.push(end.x);
        self.line_mesh.mesh_data.vertices.push(end.y);
        self.line_mesh.mesh_data.vertices.push(end.z);

        self.line_mesh.mesh_data.colors.push(color.x);
        self.line_mesh.mesh_data.colors.push(color.y);
        self.line_mesh.mesh_data.colors.push(color.z);

        self.line_mesh.mesh_data.colors.push(color.x);
        self.line_mesh.mesh_data.colors.push(color.y);
        self.line_mesh.mesh_data.colors.push(color.z);

        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(1.0);

        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(0.0);
        self.line_mesh.mesh_data.normals.push(1.0);
    }
    pub fn clear_lines(&mut self) {
        self.line_mesh.mesh_data.vertices.clear();
        self.line_mesh.mesh_data.colors.clear();
    }

    pub fn recalculate_octtree(&mut self) {
        self.oct_tree = OctTree::new(&self.splat_data.splats);
    }

    /// Octree radius query against the live splat data.
    pub fn find_splats_in_radius(&mut self, center: Vec3, radius: f32) -> Vec<OctTreeSplat> {
        self.oct_tree
            .find_splats_in_radius(center, radius, &self.splat_data.splats)
    }

    pub fn redraw_from_oct_tree(&mut self, only_clicks: bool) {
        self.clear_lines();
        let lines = self.oct_tree.get_lines(only_clicks);
        for line in lines {
            self.add_line(line.start, line.end, line.color);
        }
    }

    pub fn is_point_in_shadow(&self, point: Vec3, light_pos: Vec3) -> bool {
        let ray_origin = point;
        let dir = light_pos - point;
        let ray_direction = dir.normalize();

        let number_of_iterations = 100;
        // let dir_amount = dir / number_of_iterations as f32;

        for object in &self.objects {
            let intersection = Intersection {
                intersection_point: Vec3::zeros(),
                normal: Vec3::zeros(),
                t: 0.0,
            };

            if object.intersection(ray_origin, ray_direction).is_some() {
                return true;
            }

            // if sphere_intersection(
            //     ray_origin,
            //     ray_direction,
            //     object.pos,
            //     1.0,
            //     &mut intersection,
            // ) {

            //     // return true;
            // }
        }

        // for t in 0..number_of_iterations {
        //     let test_point = point + (t as f32) * dir_amount;

        // if self.object_at_point(test_point).is_some() {
        //     return true;
        // }
        // }
        false
    }

    pub fn find_shadow_splats(&mut self, node: &OctTreeNode, out_set: &mut HashSet<usize>) {
        let pos = node.center;
        let min_splats = 10;

        // if this node is not fine grain enough, try to go deeper first
        if node.splat_indices.len() > min_splats && node.children.len() > 0 {
            for child in &node.children {
                self.find_shadow_splats(&child, out_set);
            }
            return;
        }

        // if this node is fine grain enough, just check if the center is in shadow
        if self.is_point_in_shadow(pos, self.light_pos) {
            for &splat_index in &node.splat_indices {
                // if self.is_point_in_shadow(vec3(splat.x, splat.y, splat.z), self.light_pos) {
                out_set.insert(splat_index);
                // }
            }
            // TODO: now actually check each individual splat
        }
    }

    fn get_shadow_splat_indices(&self, node: &OctTreeNode) -> HashSet<usize> {
        let mut out_set = HashSet::new();
        let pos = node.center;
        let min_splats = 10;

        // if this node is not fine grain enough, try to go deeper first
        if node.splat_indices.len() > min_splats && node.children.len() > 0 {
            for child in &node.children {
                out_set.extend(self.get_shadow_splat_indices(child));
            }
            return out_set;
        }

        // if this node is fine grain enough, check if the center is in shadow
        if self.is_point_in_shadow(pos, self.light_pos) {
            for &splat_index in &node.splat_indices {
                out_set.insert(splat_index);
            }
        }
        out_set
    }

    pub fn calculate_shadows(&mut self) {
        for (index, color) in self.original_shadow_splat_colors.iter() {
            self.splat_data.splats[*index].r = color.x;
            self.splat_data.splats[*index].g = color.y;
            self.splat_data.splats[*index].b = color.z;
        }
        self.original_shadow_splat_colors.clear();

        let shadow_indices = self.get_shadow_splat_indices(&self.oct_tree.root);

        for index in shadow_indices {
            self.original_shadow_splat_colors.insert(
                index,
                vec3(
                    self.splat_data.splats[index].r,
                    self.splat_data.splats[index].g,
                    self.splat_data.splats[index].b,
                ),
            );

            self.splat_data.splats[index].r -= 0.4;
            self.splat_data.splats[index].g -= 0.4;
            self.splat_data.splats[index].b -= 0.4;

            self.splat_data.splats[index].r = self.splat_data.splats[index].r.max(0.0);
            self.splat_data.splats[index].g = self.splat_data.splats[index].g.max(0.0);
            self.splat_data.splats[index].b = self.splat_data.splats[index].b.max(0.0);
        }
    }

    /// Tint previews are transient: drop them whenever the selection moves
    /// away without the user hitting "apply".
    pub fn clear_tint_previews(&mut self) {
        for meta in &mut self.object_meta {
            meta.tint_strength = 0.0;
        }
    }

    pub fn select_target(&mut self, target: GizmoTarget) {
        if self.gizmo.target_object != Some(target) {
            self.clear_tint_previews();
        }
        match target {
            GizmoTarget::Mesh(idx) => {
                if let Some(object) = self.objects.get(idx) {
                    let pos = object.pos;
                    self.gizmo.update_position(pos);
                    self.gizmo.target_object = Some(target);
                }
            }
            GizmoTarget::Splat(idx) => {
                if idx < self.splat_data.objects.len() {
                    let pos = self.splat_object_position(idx);
                    self.gizmo.update_position(pos);
                    self.gizmo.target_object = Some(target);
                }
            }
        }
    }

    pub fn update_gizmo_position(&mut self, object_idx: u32) {
        self.select_target(GizmoTarget::Mesh(object_idx as usize));
    }

    pub fn hide_gizmo(&mut self) {
        log!("hiding gizmo!");
        self.gizmo.target_object = None;
        self.clear_tint_previews();
    }

    pub fn start_gizmo_drag(&mut self, axis: GizmoAxis, start_pos: Vec2) {
        let target = if let Some(target) = self.gizmo.target_object {
            target
        } else {
            log!("No target object for gizmo");
            return;
        };

        let object_pos = match target {
            GizmoTarget::Mesh(idx) => match self.objects.get(idx) {
                Some(object) => object.pos,
                None => return,
            },
            GizmoTarget::Splat(idx) => {
                if idx >= self.splat_data.objects.len() {
                    return;
                }
                self.splat_object_position(idx)
            }
        };

        self.gizmo.start_drag(axis, target, object_pos, start_pos);
    }

    pub fn update_gizmo_drag(&mut self, current_pos: Vec2, restrict_gizmo_movement: bool) {
        let new_pos = match self.gizmo.update_drag(current_pos, restrict_gizmo_movement) {
            Some(p) => p,
            None => return,
        };
        let target = match self.gizmo.target_object {
            Some(t) => t,
            None => return,
        };

        match target {
            GizmoTarget::Splat(idx) => {
                if idx >= self.object_meta.len() {
                    return;
                }
                let centroid = self.object_meta[idx].centroid;
                self.object_meta[idx].translation = new_pos - centroid;
                self.gizmo.update_position(new_pos);
            }
            GizmoTarget::Mesh(target_idx) => {
                if let Some(object) = self.objects.get_mut(target_idx) {
                    if !restrict_gizmo_movement {
                        object.pos = new_pos;
                        self.gizmo.update_position(new_pos);
                        return;
                    }

                    let old_pos = object.pos;
                    object.pos = new_pos;
                    self.gizmo.update_position(new_pos);

                    object.recalculate_min_max();
                    let min = object.min;
                    let max = object.max;
                    let points_to_check = vec![min, max];
                    for point in points_to_check {
                        let splats = self
                            .oct_tree
                            .find_splats_in_radius(point, 0.1, &self.splat_data.splats);
                        let visible_splats = splats.iter().filter(|splat| splat.opacity >= 0.5);
                        let visible_splats_count = visible_splats.count();
                        if visible_splats_count >= 1 {
                            object.pos = old_pos;
                            self.gizmo.update_position(old_pos);
                            log!("collision !");
                            #[cfg(target_arch = "wasm32")]
                            setCollisionDetected();
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn new_object_position_is_safe(&mut self, object: &mut SceneObject, pos: Vec3) -> bool {
        let old_pos = object.pos;
        object.pos = pos;

        let min = object.min;
        let max = object.max;
        let points_to_check = vec![min, max];
        let mut collision = false;
        for point in points_to_check {
            let splats = self
                .oct_tree
                .find_splats_in_radius(point, 0.05, &self.splat_data.splats);
            let visible_splats = splats.iter().filter(|splat| splat.opacity >= 0.5);
            let visible_splats_count = visible_splats.count();

            if visible_splats_count >= 1 {
                log!("collision !");
                collision = true;
            }

            if collision {
                object.pos = old_pos;
                return false;
            }
        }
        return true;
    }

    /// Ends a gizmo drag. If a splat object was being dragged, its live
    /// translation is baked into the splat positions; returns the object
    /// index and delta so the caller can refresh GPU data and record undo.
    pub fn end_gizmo_drag(&mut self) -> Option<(usize, Vec3)> {
        let was_dragging = self.gizmo.is_dragging;
        self.gizmo.end_drag();
        if !was_dragging {
            return None;
        }
        if let Some(GizmoTarget::Splat(idx)) = self.gizmo.target_object {
            if idx < self.object_meta.len() {
                let delta = self.object_meta[idx].translation;
                if glm::length(&delta) > 1e-6 {
                    self.splat_data.translate_object(idx, delta);
                    self.object_meta[idx].centroid += delta;
                    self.object_meta[idx].translation = vec3(0.0, 0.0, 0.0);
                    self.octree_dirty = true;
                    self.push_undo(EditOp::Move { object: idx, delta });
                    return Some((idx, delta));
                }
            }
        }
        None
    }

    pub fn move_down(&mut self) {
        for index in 0..self.objects.len() {
            let object = &mut self.objects[index];
            object.recalculate_min_max();
            let min = object.min;
            let max = object.max;
            let points_to_check = vec![min, max];
            let mut collision = false;
            for point in points_to_check {
                let splats = self
                    .oct_tree
                    .find_splats_in_radius(point, 0.05, &self.splat_data.splats);
                let visible_splats = splats.iter().filter(|splat| splat.opacity >= 0.5);
                let visible_splats_count = visible_splats.count();

                if visible_splats_count >= 1 {
                    log!("collision !");
                    collision = true;
                }

                if collision {
                    log!("collision, so not moving down!");
                    if let Some(GizmoTarget::Mesh(target_idx)) = self.gizmo.target_object {
                        if target_idx == index {
                            self.gizmo.update_position(object.pos);
                        }
                    }
                    break;
                } else {
                    object.pos.y += 0.01;

                    if let Some(GizmoTarget::Mesh(target_idx)) = self.gizmo.target_object {
                        if target_idx == index {
                            self.gizmo.update_position(object.pos);
                        }
                    }
                }
            }
        }
    }
}

pub fn sphere_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    sphere_pos: Vec3,
    sphere_radius: f32,
    intersection: &mut Intersection,
) -> bool {
    let tmp = ray_origin - sphere_pos;
    let a = glm::dot(&ray_direction, &ray_direction);
    let b = 2.0 * glm::dot(&ray_direction, &tmp);
    let c = glm::dot(&tmp, &tmp) - sphere_radius * sphere_radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant >= 0.0 {
        let root1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let root2 = (-b + discriminant.sqrt()) / (2.0 * a);

        let root = if root1 >= 0.0 {
            root1
        } else if root2 >= 0.0 {
            root2
        } else {
            return false;
        };

        intersection.intersection_point = ray_origin + root * ray_direction;
        intersection.normal = (intersection.intersection_point - sphere_pos).normalize();
        intersection.t = root;

        const EPSILON: f32 = 1e-5;
        if intersection.t > 1000.0 * EPSILON {
            return true;
        }
    }
    false
}

pub struct Intersection {
    pub intersection_point: Vec3,
    pub normal: Vec3,
    pub t: f32,
}
