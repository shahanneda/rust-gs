use crate::DataObjects::MeshData;
use bytes::buf;
use nalgebra_glm::Vec3;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec4};
// use serde::{Deserialize, Serialize};
use crate::log;

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

impl SceneObject {
    pub fn new(mesh_data: MeshData, pos: Vec3, rot: Vec3, scale: Vec3) -> Self {
        Self {
            mesh_data,
            pos,
            rot,
            scale,
        }
    }
}
