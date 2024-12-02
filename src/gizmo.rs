use crate::scene_geo;
use crate::scene_object::SceneObject;
use crate::{data_objects::MeshData, log};
use nalgebra_glm::{self as glm, vec3, Vec2, Vec3};

pub struct Gizmo {
    pub axis_x: SceneObject,
    pub axis_y: SceneObject,
    pub axis_z: SceneObject,
    pub target_object: Option<usize>,
    pub active_axis: Option<GizmoAxis>,
    pub drag_start_pos: Option<Vec2>,
    pub original_object_pos: Option<Vec3>,
    pub is_dragging: bool,
}
#[derive(Debug)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

impl Gizmo {
    pub fn new() -> Self {
        let axis_scale = vec3(0.5, 0.02, 0.02);

        // X axis (red)
        let x_axis = SceneObject::new(
            MeshData::new(
                scene_geo::CUBE_VERTICES.to_vec(),
                scene_geo::CUBE_INDICES.to_vec(),
                vec![1.0, 0.0, 0.0].repeat(scene_geo::CUBE_VERTICES.len() / 3), // Red color
                scene_geo::CUBE_NORMALS.to_vec(),
            ),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 0.0),
            axis_scale,
        );

        // Y axis (green)
        let y_axis = SceneObject::new(
            MeshData::new(
                scene_geo::CUBE_VERTICES.to_vec(),
                scene_geo::CUBE_INDICES.to_vec(),
                vec![0.0, 1.0, 0.0].repeat(scene_geo::CUBE_VERTICES.len() / 3), // Green color
                scene_geo::CUBE_NORMALS.to_vec(),
            ),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, glm::pi::<f32>() / 2.0),
            axis_scale,
        );

        // Z axis (blue)
        let z_axis = SceneObject::new(
            MeshData::new(
                scene_geo::CUBE_VERTICES.to_vec(),
                scene_geo::CUBE_INDICES.to_vec(),
                vec![0.0, 0.0, 1.0].repeat(scene_geo::CUBE_VERTICES.len() / 3), // Blue color
                scene_geo::CUBE_NORMALS.to_vec(),
            ),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, -glm::pi::<f32>() / 2.0, 0.0),
            axis_scale,
        );

        Self {
            axis_x: x_axis,
            axis_y: y_axis,
            axis_z: z_axis,
            target_object: None,
            active_axis: None,
            drag_start_pos: None,
            original_object_pos: None,
            is_dragging: false,
        }
    }

    pub fn update_position(&mut self, pos: Vec3) {
        self.axis_x.pos = pos;
        self.axis_y.pos = pos;
        self.axis_z.pos = pos;
    }
    pub fn clamp_delta(&self, delta: &mut Vec2) {
        if delta.x > 1.0 {
            delta.x = 1.0;
        } else if delta.x < -1.0 {
            delta.x = -1.0;
        }
        if delta.y > 1.0 {
            delta.y = 1.0;
        } else if delta.y < -1.0 {
            delta.y = -1.0;
        }
    }

    pub fn start_drag(
        &mut self,
        axis: GizmoAxis,
        object_idx: usize,
        object_pos: Vec3,
        start_pos: Vec2,
    ) {
        log!("active object idx: {:?}", object_idx);
        log!("starting drag axis: {:?}", axis);
        self.active_axis = Some(axis);
        self.target_object = Some(object_idx);
        self.drag_start_pos = Some(start_pos);
        self.original_object_pos = Some(object_pos);
        self.is_dragging = true;
    }

    pub fn update_drag(&self, current_pos: Vec2, restrict_gizmo_movement: bool) -> Option<Vec3> {
        if !self.is_dragging {
            return None;
        }

        if let (Some(start_pos), Some(orig_pos), Some(axis)) = (
            self.drag_start_pos,
            self.original_object_pos,
            &self.active_axis,
        ) {
            let delta = current_pos - start_pos;
            let movement_scale = 0.01;
            let mut scaled_delta = delta * movement_scale;
            if restrict_gizmo_movement {    
                self.clamp_delta(&mut scaled_delta);
            }

            log!("scaled delta: {:?}", scaled_delta);

            let mut new_pos = orig_pos;
            match axis {
                GizmoAxis::X => new_pos.x += scaled_delta.x,
                GizmoAxis::Y => new_pos.y += scaled_delta.y,
                GizmoAxis::Z => new_pos.z += scaled_delta.x,
            }

            Some(new_pos)
        } else {
            None
        }
    }

    pub fn end_drag(&mut self) {
        // self.active_axis = None;
        // self.target_object = None;
        // self.drag_start_pos = None;
        // self.original_object_pos = None;
        self.is_dragging = false;
    }

    pub fn get_axis_objects(&self) -> Vec<&SceneObject> {
        vec![&self.axis_x, &self.axis_y, &self.axis_z]
    }
}
