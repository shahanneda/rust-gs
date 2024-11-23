use bytes::buf;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec3, Vec4};
// use serde::{Deserialize, Serialize};
use crate::log;

use crate::scene_object::SceneObject;
use crate::splat::Splat;
use crate::timer::Timer;
use crate::DataObjects::{MeshData, SplatData};
use crate::{ply_splat::PlySplat, shared_utils::sigmoid};
use nalgebra_glm as glm;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};
// use speedy::{Readable, Writable, Endianness};

// #[derive(Serialize, Deserialize, Debug)]
pub struct Scene {
    pub splat_data: SplatData,
    pub objects: Vec<SceneObject>,
    pub line_mesh: SceneObject,
}

impl Scene {
    pub fn new(splat_data: SplatData) -> Self {
        Self {
            splat_data,
            objects: Vec::new(),
            line_mesh: SceneObject::new(
                MeshData::new(vec![], vec![], vec![]),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(1.0, 1.0, 1.0),
            ),
            // line_verts: Vec::new(),
            // line_colors: Vec::new(),
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
    }
}
