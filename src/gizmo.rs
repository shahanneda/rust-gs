use crate::scene_geo;
use crate::scene_object::SceneObject;
use crate::{data_objects::MeshData, log};
use nalgebra_glm::{self as glm, vec2, vec3, Mat4, Vec2, Vec3};

/// What the gizmo (and the editor selection) points at: a triangle-mesh
/// object in `scene.objects` or a gaussian-splat object in
/// `scene.splat_data.objects`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoTarget {
    Mesh(usize),
    Splat(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
    /// Center handle: uniform scale.
    Uniform,
}

impl GizmoAxis {
    pub fn world_dir(&self) -> Vec3 {
        match self {
            GizmoAxis::X => vec3(1.0, 0.0, 0.0),
            GizmoAxis::Y => vec3(0.0, 1.0, 0.0),
            GizmoAxis::Z => vec3(0.0, 0.0, 1.0),
            GizmoAxis::Uniform => vec3(0.0, 0.0, 0.0),
        }
    }
    pub fn index(&self) -> usize {
        match self {
            GizmoAxis::X => 0,
            GizmoAxis::Y => 1,
            GizmoAxis::Z => 2,
            GizmoAxis::Uniform => 3,
        }
    }
}

/// What the current mouse position means for the dragged object, all
/// relative to the state at drag start.
#[derive(Debug, Clone, Copy)]
pub enum GizmoDragUpdate {
    /// New object position.
    Translate(Vec3),
    /// Rotation around a world axis (through the object pivot), radians.
    Rotate { axis: Vec3, angle: f32 },
    /// Scale factor along one axis (or all axes for Uniform).
    Scale { axis: GizmoAxis, factor: f32 },
}

struct DragState {
    axis: GizmoAxis,
    start_mouse: Vec2,
    /// Object center projected to screen pixels at drag start.
    center_px: Vec2,
    /// Screen-space direction of the dragged axis (normalized).
    axis_dir_px: Vec2,
    /// How many pixels one world unit along the axis covers.
    px_per_unit: f32,
    /// Sign flip for rotation depending on whether the axis faces the camera.
    rot_sign: f32,
    start_object_pos: Vec3,
}

pub struct Gizmo {
    pub mode: GizmoMode,
    handles_translate: Vec<SceneObject>,
    handles_rotate: Vec<SceneObject>,
    handles_scale: Vec<SceneObject>,
    pub target_object: Option<GizmoTarget>,
    pub is_dragging: bool,
    drag: Option<DragState>,
    pos: Vec3,
    screen_scale: f32,
}

const AXIS_COLORS: [[f32; 3]; 3] = [
    [0.96, 0.26, 0.30], // X red
    [0.30, 0.85, 0.39], // Y green
    [0.25, 0.52, 0.97], // Z blue
];

fn handle_object(mesh: (Vec<f32>, Vec<u32>, Vec<f32>), color: [f32; 3], rot: Vec3) -> SceneObject {
    let (vertices, indices, normals) = mesh;
    let colors = color.repeat(vertices.len() / 3);
    SceneObject::new(
        MeshData::new(vertices, indices, colors, normals),
        vec3(0.0, 0.0, 0.0),
        rot,
        vec3(1.0, 1.0, 1.0),
    )
}

impl Gizmo {
    pub fn new() -> Self {
        let half_pi = glm::half_pi::<f32>();

        // Arrows are authored along +X.
        let handles_translate = vec![
            handle_object(scene_geo::arrow_mesh(), AXIS_COLORS[0], vec3(0.0, 0.0, 0.0)),
            handle_object(scene_geo::arrow_mesh(), AXIS_COLORS[1], vec3(0.0, 0.0, half_pi)),
            handle_object(scene_geo::arrow_mesh(), AXIS_COLORS[2], vec3(0.0, -half_pi, 0.0)),
        ];

        // Rings are authored in the XY plane (normal +Z).
        let ring = || scene_geo::torus_mesh(0.85, 0.035);
        let handles_rotate = vec![
            handle_object(ring(), AXIS_COLORS[0], vec3(0.0, half_pi, 0.0)),
            handle_object(ring(), AXIS_COLORS[1], vec3(half_pi, 0.0, 0.0)),
            handle_object(ring(), AXIS_COLORS[2], vec3(0.0, 0.0, 0.0)),
        ];

        // Scale sticks along +X plus a center cube for uniform scale.
        let handles_scale = vec![
            handle_object(scene_geo::scale_handle_mesh(), AXIS_COLORS[0], vec3(0.0, 0.0, 0.0)),
            handle_object(scene_geo::scale_handle_mesh(), AXIS_COLORS[1], vec3(0.0, 0.0, half_pi)),
            handle_object(scene_geo::scale_handle_mesh(), AXIS_COLORS[2], vec3(0.0, -half_pi, 0.0)),
            handle_object(
                scene_geo::center_cube_mesh(),
                [0.92, 0.92, 0.95],
                vec3(0.0, 0.0, 0.0),
            ),
        ];

        Self {
            mode: GizmoMode::Translate,
            handles_translate,
            handles_rotate,
            handles_scale,
            target_object: None,
            is_dragging: false,
            drag: None,
            pos: vec3(0.0, 0.0, 0.0),
            screen_scale: 1.0,
        }
    }

    /// The handle objects of the current mode, in picking-index order
    /// (0 = X, 1 = Y, 2 = Z, 3 = uniform-scale center cube).
    pub fn get_handles(&self) -> &[SceneObject] {
        match self.mode {
            GizmoMode::Translate => &self.handles_translate,
            GizmoMode::Rotate => &self.handles_rotate,
            GizmoMode::Scale => &self.handles_scale,
        }
    }

    pub fn axis_from_pick_index(&self, index: u32) -> Option<GizmoAxis> {
        match index {
            0 => Some(GizmoAxis::X),
            1 => Some(GizmoAxis::Y),
            2 => Some(GizmoAxis::Z),
            3 if self.mode == GizmoMode::Scale => Some(GizmoAxis::Uniform),
            _ => None,
        }
    }

    pub fn update_position(&mut self, pos: Vec3) {
        self.pos = pos;
        self.apply_pose();
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    /// Keep the gizmo roughly constant-size on screen: called every frame
    /// with the camera distance to the gizmo.
    pub fn update_screen_scale(&mut self, camera_distance: f32) {
        self.screen_scale = (camera_distance * 0.17).clamp(0.05, 20.0);
        self.apply_pose();
    }

    fn apply_pose(&mut self) {
        let pos = self.pos;
        let s = self.screen_scale;
        for list in [
            &mut self.handles_translate,
            &mut self.handles_rotate,
            &mut self.handles_scale,
        ] {
            for h in list.iter_mut() {
                h.pos = pos;
                h.scale = vec3(s, s, s);
            }
        }
    }

    /// Project a world point to screen pixels (top-left origin).
    fn project(vpm: &Mat4, p: Vec3, width: i32, height: i32) -> Option<Vec2> {
        let clip = vpm * glm::vec4(p.x, p.y, p.z, 1.0);
        if clip.w.abs() < 1e-6 {
            return None;
        }
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;
        Some(vec2(
            (ndc_x * 0.5 + 0.5) * width as f32,
            (1.0 - (ndc_y * 0.5 + 0.5)) * height as f32,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_drag(
        &mut self,
        axis: GizmoAxis,
        target: GizmoTarget,
        object_pos: Vec3,
        start_mouse: Vec2,
        vpm: Mat4,
        vm: Mat4,
        width: i32,
        height: i32,
    ) {
        log!("gizmo start drag: {:?} on {:?}", axis, target);
        let center_px = match Self::project(&vpm, object_pos, width, height) {
            Some(p) => p,
            None => return,
        };

        let axis_world = axis.world_dir();
        let (axis_dir_px, px_per_unit) = if axis == GizmoAxis::Uniform {
            (vec2(1.0, 0.0), 1.0)
        } else {
            let step = 0.25f32;
            match Self::project(&vpm, object_pos + axis_world * step, width, height) {
                Some(p2) => {
                    let d = p2 - center_px;
                    let len = glm::length(&d);
                    if len < 1e-4 {
                        // Axis points straight at the camera; fall back.
                        (vec2(1.0, 0.0), 100.0)
                    } else {
                        (d / len, len / step)
                    }
                }
                None => (vec2(1.0, 0.0), 100.0),
            }
        };

        // Rotation direction depends on whether the axis faces the camera.
        let view_axis = glm::mat4_to_mat3(&vm) * axis_world;
        let rot_sign = if view_axis.z > 0.0 { -1.0 } else { 1.0 };

        self.target_object = Some(target);
        self.is_dragging = true;
        self.drag = Some(DragState {
            axis,
            start_mouse,
            center_px,
            axis_dir_px,
            px_per_unit,
            rot_sign,
            start_object_pos: object_pos,
        });
    }

    pub fn update_drag(&self, current_mouse: Vec2) -> Option<GizmoDragUpdate> {
        if !self.is_dragging {
            return None;
        }
        let d = self.drag.as_ref()?;

        match self.mode {
            GizmoMode::Translate => {
                let mouse_delta = current_mouse - d.start_mouse;
                // The .max() keeps camera-facing axes (with a nearly
                // degenerate screen projection) from flinging the object.
                let along = glm::dot(&mouse_delta, &d.axis_dir_px) / d.px_per_unit.max(15.0);
                Some(GizmoDragUpdate::Translate(
                    d.start_object_pos + d.axis.world_dir() * along,
                ))
            }
            GizmoMode::Rotate => {
                let v0 = d.start_mouse - d.center_px;
                let v1 = current_mouse - d.center_px;
                if glm::length(&v0) < 4.0 || glm::length(&v1) < 4.0 {
                    return None;
                }
                let a0 = v0.y.atan2(v0.x);
                let a1 = v1.y.atan2(v1.x);
                let mut delta = a1 - a0;
                // shortest way around
                while delta > std::f32::consts::PI {
                    delta -= 2.0 * std::f32::consts::PI;
                }
                while delta < -std::f32::consts::PI {
                    delta += 2.0 * std::f32::consts::PI;
                }
                Some(GizmoDragUpdate::Rotate {
                    axis: d.axis.world_dir(),
                    angle: delta * d.rot_sign,
                })
            }
            GizmoMode::Scale => {
                // Axes that point (nearly) at the camera have a degenerate
                // screen projection — use the radial distance-from-center
                // formula for those, same as the uniform center handle.
                let use_radial = d.axis == GizmoAxis::Uniform || d.px_per_unit < 12.0;
                let factor = if use_radial {
                    let r0 = glm::length(&(d.start_mouse - d.center_px)).max(12.0);
                    let r1 = glm::length(&(current_mouse - d.center_px));
                    (r1 / r0).clamp(0.05, 20.0)
                } else {
                    // Move outward along the handle to grow, inward to shrink.
                    let mouse_delta = current_mouse - d.start_mouse;
                    let along = glm::dot(&mouse_delta, &d.axis_dir_px) / d.px_per_unit.max(1e-4);
                    // start handle length ≈ one gizmo unit on screen
                    (1.0 + along / self.screen_scale.max(1e-3)).clamp(0.05, 20.0)
                };
                Some(GizmoDragUpdate::Scale {
                    axis: d.axis,
                    factor,
                })
            }
        }
    }

    pub fn drag_axis(&self) -> Option<GizmoAxis> {
        self.drag.as_ref().map(|d| d.axis)
    }

    pub fn drag_start_pos(&self) -> Option<Vec3> {
        self.drag.as_ref().map(|d| d.start_object_pos)
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.drag = None;
    }
}
