use crate::DataObjects::MeshData;
use bytes::buf;
use nalgebra_glm::Vec3;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec4};
// use serde::{Deserialize, Serialize};
use crate::{log, scene_geo};

use crate::splat::Splat;
use crate::timer::Timer;
use crate::{ply_splat::PlySplat, shared_utils::sigmoid};
use nalgebra_glm as glm;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};

#[derive(Debug)]
pub struct SceneObject {
    pub mesh_data: MeshData,
    pub pos: Vec3,
    pub rot: Vec3,
    pub scale: Vec3,
}
#[derive(Debug)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec3,
}

impl SceneObject {
    pub fn new(mesh_data: MeshData, pos: Vec3, rot: Vec3, scale: Vec3) -> Self {
        Self {
            mesh_data,
            pos,
            rot,
            scale,
        }
    }

    pub fn new_cube(center: Vec3, width: f32, color: Vec3) -> Self {
        let mut colors = scene_geo::CUBE_COLORS.to_vec();
        let indices = scene_geo::CUBE_INDICES.to_vec();
        let vertices = scene_geo::CUBE_VERTICES.to_vec();

        for i in 0..colors.len() / 3 {
            colors[i * 3] = color.x;
            colors[i * 3 + 1] = color.y;
            colors[i * 3 + 2] = color.z;
        }

        let mesh_data = MeshData {
            colors,
            indices: scene_geo::CUBE_INDICES.to_vec(),
            vertices: scene_geo::CUBE_VERTICES.to_vec(),
        };

        Self::new(
            mesh_data,
            center,
            vec3(0.0, 0.0, 0.0),
            vec3(width / 2.0, width / 2.0, width / 2.0),
        )
    }
}
