use bytes::buf;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec3, Vec4};
// use serde::{Deserialize, Serialize};
use crate::log;

use crate::scene_object::SceneObject;
use crate::splat::Splat;
use crate::timer::Timer;
use crate::DataObjects::SplatData;
use crate::{ply_splat::PlySplat, shared_utils::sigmoid};
use nalgebra_glm as glm;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};
// use speedy::{Readable, Writable, Endianness};

// #[derive(Serialize, Deserialize, Debug)]
pub struct Scene {
    pub splat_data: SplatData,
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new(splat_data: SplatData) -> Self {
        Self {
            splat_data,
            objects: Vec::new(),
        }
    }
}
