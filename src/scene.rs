use std::collections::{HashMap, HashSet};

use crate::data_objects::MeshData;
use crate::gizmo::{Gizmo, GizmoAxis};
use crate::log;
use crate::oct_tree::{OctTreeNode, OctTreeSplat};
use crate::scene_object::SceneObject;
use crate::{data_objects::SplatData, oct_tree::OctTree};
use nalgebra_glm::{self as glm, Vec2};
use nalgebra_glm::{vec3, Vec3};

pub struct Scene {
    pub splat_data: SplatData,
    pub objects: Vec<SceneObject>,
    pub line_mesh: SceneObject,
    pub light_pos: Vec3,
    pub original_shadow_splat_colors: HashMap<usize, Vec3>,
    pub gizmo: Gizmo,
    pub oct_tree: OctTree,
}

impl Scene {
    pub fn new(splat_data: SplatData) -> Self {
        let oct_tree = OctTree::new(splat_data.splats.clone());
        Self {
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
        }
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
        if node.splats.len() > min_splats && node.children.len() > 0 {
            for child in &node.children {
                self.find_shadow_splats(&child, out_set);
            }
            return;
        }

        // if this node is fine grain enough, just check if the center is in shadow
        if self.is_point_in_shadow(pos, self.light_pos) {
            for splat in &node.splats {
                // if self.is_point_in_shadow(vec3(splat.x, splat.y, splat.z), self.light_pos) {
                out_set.insert(splat.index);
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
        if node.splats.len() > min_splats && node.children.len() > 0 {
            for child in &node.children {
                out_set.extend(self.get_shadow_splat_indices(child));
            }
            return out_set;
        }

        // if this node is fine grain enough, check if the center is in shadow
        if self.is_point_in_shadow(pos, self.light_pos) {
            for splat in &node.splats {
                out_set.insert(splat.index);
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

        // let shadow_points: Vec<_> = self
        //     .splat_data
        //     .splats
        //     .iter()
        //     .map(|splat| self.is_point_in_shadow(vec3(splat.x, splat.y, splat.z), self.light_pos))
        //     .collect();

        // for (splat, is_shadowed) in self.splat_data.splats.iter_mut().zip(shadow_points) {
        //     if is_shadowed {
        //         // splat.opacity = 0.0;
        //         splat.r -= 0.4;
        //         splat.g -= 0.4;
        //         splat.b -= 0.4;
        //     }
        // }
    }

    pub fn update_gizmo_position(&mut self, object_idx: u32) {
        if let Some(object) = self.objects.get(object_idx as usize) {
            self.gizmo.update_position(object.pos);
            self.gizmo.target_object = Some(object_idx as usize);
        }
    }
    pub fn hide_gizmo(&mut self) {
        self.gizmo.target_object = None;
    }

    pub fn start_gizmo_drag(&mut self, axis: GizmoAxis, start_pos: Vec2) {
        let target_idx = if let Some(idx) = self.gizmo.target_object {
            idx
        } else {
            log!("No target object for gizmo");
            return;
        };
        log!("start drag target idx: {:?}", target_idx);

        let object_pos = if let Some(object) = self.objects.get(target_idx) {
            object.pos
        } else {
            log!("Target object not found");
            return;
        };

        self.gizmo
            .start_drag(axis, target_idx, object_pos, start_pos);
    }

    pub fn update_gizmo_drag(&mut self, current_pos: Vec2) {
        // if let Some(target_idx) = self.gizmo.target_object {
        //     // Project ray onto the active axis plane
        //     if let Some(axis) = self.gizmo.active_axis {
        //         let plane_normal = match axis {
        //             GizmoAxis::X => vec3(1.0, 0.0, 0.0),
        //             GizmoAxis::Y => vec3(0.0, 1.0, 0.0),
        //             GizmoAxis::Z => vec3(0.0, 0.0, 1.0),
        //             _ => return,
        //         };

        //         let t = glm::dot(
        //             &(self.gizmo.drag_start_pos.unwrap_or(vec3(0.0, 0.0, 0.0)) - ray_origin),
        //             &plane_normal,
        //         ) / glm::dot(&ray_direction, &plane_normal);
        //         let intersection_point = ray_origin + ray_direction * t;

        //         if let Some(new_pos) = self.gizmo.update_drag(intersection_point) {
        //             if let Some(object) = self.objects.get_mut(target_idx) {
        //                 object.pos = new_pos;
        //                 self.gizmo.update_position(new_pos);
        //             }
        //         }
        // }

        if let Some(new_pos) = self.gizmo.update_drag(current_pos) {
            let target_idx = self.gizmo.target_object.unwrap();
            log!("target idx: {:?}", target_idx);
            if let Some(object) = self.objects.get_mut(target_idx) {
                object.pos = new_pos;
                self.gizmo.update_position(new_pos);
            }
        }
    }

    pub fn end_gizmo_drag(&mut self) {
        log!("ending gizmo drag!");
        self.gizmo.end_drag();
    }
    pub fn move_down(&mut self) {
        for object in &mut self.objects {
            object.recalculate_min_max();
            let min = object.min;
            let max = object.max;
            let points_to_check = vec![min, max];
            let mut collision = false;
            for point in points_to_check {
                let splats = self.oct_tree.find_splats_in_radius(point, 0.05);
                let visible_splats = splats.iter().filter(|splat| splat.opacity >= 0.5);
                let visible_splats_count = visible_splats.count();

                if visible_splats_count >= 1 {
                    log!("collision !");
                    collision = true;
                }

                if collision {
                    log!("collision, so not moving down!");
                    break;
                } else {
                    object.pos.y += 0.01;
                }
                // scene.borrow_mut().calculate_shadows(&oct_tree.borrow());
                // renderer
                //     .borrow_mut()
                //     .update_webgl_textures(&scene.borrow())
                //     .expect("failed to update webgl textures when moving down");
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
