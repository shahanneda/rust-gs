use crate::splat::Splat;
use bytes::buf;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec3, Vec4};
// use serde::{Deserialize, Serialize};
use crate::log;

use crate::scene_object::{Line, SceneObject};
use crate::timer::Timer;
use crate::{ply_splat::PlySplat, shared_utils::sigmoid};
use nalgebra_glm as glm;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};
// use speedy::{Readable, Writable, Endianness};

#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub colors: Vec<f32>,
}

impl MeshData {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>, colors: Vec<f32>) -> Self {
        Self {
            vertices,
            indices,
            colors,
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub struct SplatData {
    pub splats: Vec<Splat>,
}

impl SplatData {
    pub async fn new_from_url(url: &str) -> Self {
        let _timer = Timer::new("loading json file");
        let loaded_file = reqwest::get(url)
            .await
            .expect("error")
            .bytes()
            .await
            .expect("went wrong when reading!");
        return SplatData::new_from_rkyv(&loaded_file);
    }

    pub fn new_from_rkyv(bytes: &[u8]) -> Self {
        let _timer = Timer::new("new scene from json");
        log!("Creating a new scene from rkyv");

        match rkyv::from_bytes::<SplatData, Error>(bytes) {
            Ok(mut scene) => {
                // only take 100 splats
                // scene.splats.truncate(5);
                log!("scene has {} splats", scene.splats.len());
                return scene;
            }
            Err(e) => {
                // Handle the error appropriately. For now, we'll panic with a message.
                panic!("Failed to deserialize scene: {:?}", e);
            }
        }
    }

    pub fn new(splats: Vec<PlySplat>) -> Self {
        let _timer = Timer::new("new scene");
        let splats = splats.iter().map(|splat| Splat::new(splat)).collect();

        return SplatData { splats: splats };
    }

    pub fn splat_count(&self) -> usize {
        return self.splats.len();
    }

    pub fn nearest_power_of_2_bigger_than(&self, x: usize) -> usize {
        let mut y = 1;
        while y < x {
            y *= 2;
        }
        return y;
    }

    pub fn compress_splats_into_buffer(&self) -> Vec<u8> {
        let num_properties_per_splat = 15;
        let mut buffer = vec![0.0; self.splat_count() * num_properties_per_splat];

        for i in 0..self.splat_count() {
            // s_color, s_center, s_cov3da, s_cov3db, s_opacity;
            let splat = &self.splats[i];

            buffer[i * num_properties_per_splat + 0] = splat.r;
            buffer[i * num_properties_per_splat + 1] = splat.g;
            buffer[i * num_properties_per_splat + 2] = splat.b;

            buffer[i * num_properties_per_splat + 3] = splat.x;
            buffer[i * num_properties_per_splat + 4] = splat.y;
            buffer[i * num_properties_per_splat + 5] = splat.z;

            buffer[i * num_properties_per_splat + 6] = splat.cov3d[0];
            buffer[i * num_properties_per_splat + 7] = splat.cov3d[1];
            buffer[i * num_properties_per_splat + 8] = splat.cov3d[2];
            buffer[i * num_properties_per_splat + 9] = splat.cov3d[3];
            buffer[i * num_properties_per_splat + 10] = splat.cov3d[4];
            buffer[i * num_properties_per_splat + 11] = splat.cov3d[5];

            buffer[i * num_properties_per_splat + 12] = splat.opacity;
            buffer[i * num_properties_per_splat + 13] = splat.nx;
            buffer[i * num_properties_per_splat + 14] = splat.ny;
        }

        let mut out: Vec<u8> = vec![0; self.nearest_power_of_2_bigger_than(buffer.len() * 4)];
        for i in 0..buffer.len() {
            f32_to_4_bytes(buffer[i])
                .iter()
                .enumerate()
                .for_each(|(j, &byte)| out[i * 4 + j] = byte);
        }
        return out;
    }

    pub fn sort_splats_based_on_depth(&mut self, view_matrix: glm::Mat4) -> Vec<u32> {
        let _timer = Timer::new("sort_splats_based_on_depth");
        // track start time

        let mut depth_list_timer = Timer::new("create depth list");
        // Precompute these values outside the loop
        let view_matrix_2 = view_matrix[2];
        let view_matrix_6 = view_matrix[6];
        let view_matrix_10 = view_matrix[10];

        let mut depth_list = Vec::with_capacity(self.splats.len());
        let mut max_depth = i32::MIN;
        let mut min_depth = i32::MAX;

        for splat in &self.splats {
            let depth =
                -((splat.x * view_matrix_2 + splat.y * view_matrix_6 + splat.z * view_matrix_10)
                    * 1000.0) as i32;

            depth_list.push(depth);
            max_depth = max_depth.max(depth);
            min_depth = min_depth.min(depth);
        }
        depth_list_timer.end();

        let mut count_array_timer = Timer::new("create count array");
        let mut count_array = vec![0; (max_depth - min_depth + 1) as usize];
        count_array_timer.end();

        // Count the number of splats at each depth
        // log!("max is {max_depth} min is {min_depth}");
        let mut count_array_timer = Timer::new("count splats at each depth");
        for i in 0..self.splats.len() {
            depth_list[i] -= min_depth;
            count_array[depth_list[i] as usize] += 1;
        }
        count_array_timer.end();
        // Do prefix sum
        let mut prefix_sum_timer = Timer::new("prefix sum");
        for i in 1..count_array.len() {
            count_array[i] += count_array[i - 1];
        }
        prefix_sum_timer.end();

        let mut output_vector_timer = Timer::new("creating output vector");
        let length = self.splats.len();
        let mut output_indices = vec![0; length];
        for i in (0..self.splats.len()).rev() {
            let depth = depth_list[i];
            let index = count_array[depth as usize] - 1;
            // want the order to be reverse
            output_indices[length - index as usize - 1] = i as u32;
            count_array[depth as usize] -= 1;
        }
        output_vector_timer.end();
        return output_indices;
    }
}


pub fn u32_to_4_bytes(x: u32) -> [u8; 4] {
    let bytes = x.to_be_bytes();
    let result = [bytes[0], bytes[1], bytes[2], bytes[3]];
    result
}

pub fn f32_to_u32(x: f32) -> u32 {
    let result = u32::from(x.to_bits());
    result
}

pub fn f32_to_4_bytes(x: f32) -> [u8; 4] {
    let bytes = f32_to_u32(x);
    u32_to_4_bytes(bytes)
}
