use crate::splat::Splat;
// use serde::{Deserialize, Serialize};
use crate::log;

use crate::ply_splat::PlySplat;
use crate::timer::Timer;
use nalgebra_glm::{self as glm, vec3, vec4, Vec3};
use rkyv::rancor::Error;
use rkyv::{Archive, Deserialize, Serialize};
// use speedy::{Readable, Writable, Endianness};

#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub colors: Vec<f32>,
    pub normals: Vec<f32>,
    pub min: Vec3,
    pub max: Vec3,
}

impl MeshData {
    pub fn new(vertices: Vec<f32>, indices: Vec<u32>, colors: Vec<f32>, normals: Vec<f32>) -> Self {
        let mut min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = vec3(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY);

        // go in groups of 3
        for i in (0..vertices.len()).step_by(3) {
            min.x = min.x.min(vertices[i]);
            min.y = min.y.min(vertices[i + 1]);
            min.z = min.z.min(vertices[i + 2]);

            max.x = max.x.max(vertices[i]);
            max.y = max.y.max(vertices[i + 1]);
            max.z = max.z.max(vertices[i + 2]);
        }

        Self {
            vertices,
            indices,
            colors,
            normals,
            min,
            max,
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SplatObject {
    start: u32,
    end: u32,
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
    pub objects: Vec<SplatObject>,
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
        let splats: Vec<Splat> = splats.iter().map(|splat| Splat::new(splat)).collect();
        let end = splats.len() as u32 - 1;

        return SplatData {
            splats: splats,
            objects: vec![SplatObject { start: 0, end }],
        };
    }

    pub fn merge_with_other_splatdata(&mut self, other: &SplatData) {
        let new_start = self.splats.len() as u32;
        self.splats.extend(other.splats.iter());
        let new_end = self.splats.len() as u32 - 1;
        self.objects.push(SplatObject {
            start: new_start,
            end: new_end,
        });
    }
    pub fn apply_transformation_to_object(
        &mut self,
        object_index: usize,
        translation: glm::Mat4,
        rotation: glm::Mat4,
    ) {
        let object = &mut self.objects[object_index];
        for i in object.start..object.end {
            let splat = &mut self.splats[i as usize];
            // Transform position
            let new_splat = translation * vec4(splat.x, splat.y, splat.z, 1.0);
            splat.x = new_splat[0];
            splat.y = new_splat[1];
            splat.z = new_splat[2];

            // let current_rot = glm::quat(splat.rot_0, splat.rot_1, splat.rot_2, splat.rot_3);
            // let current_rot_mat = glm::quat_to_mat4(&current_rot);
            // let combined_rot_mat = rotation * current_rot_mat;
            // let new_quat = glm::mat3_to_quat(&glm::mat4_to_mat3(&combined_rot_mat));

            // splat.rot_0 = new_quat[0]; // x
            // splat.rot_1 = new_quat[1]; // y
            // splat.rot_2 = new_quat[2]; // z
            // splat.rot_3 = new_quat[3]; // w
        }
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
