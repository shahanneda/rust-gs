use crate::splat::Splat;
use bytes::buf;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec3, Vec4};
// use serde::{Deserialize, Serialize};
use crate::log;

use crate::scene_object::SceneObject;
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
                ((splat.x * view_matrix_2 + splat.y * view_matrix_6 + splat.z * view_matrix_10)
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

pub struct OctTreeNode {
    pub children: Vec<OctTreeNode>,
    pub splats: Vec<Splat>,
    pub center: Vec3,
    pub half_width: f32,
}

pub struct OctTree {
    pub root: OctTreeNode,
}
// mapping from i to top right back, top right front, bottom right back, bottom right front, top left back, top left front, bottom left back, bottom left front
const SPLIT_LIMIT: usize = 7000;

impl OctTreeNode {
    pub fn new(splats: Vec<Splat>, center: Vec3, half_width: f32) -> Self {
        // let center = splats
        //     .iter()
        //     .map(|splat| vec3(splat.x, splat.y, splat.z))
        //     .sum::<Vec3>()
        //     / splats.len() as f32;
        let len = splats.len();
        let mut out = OctTreeNode {
            children: vec![],
            splats,
            center,
            half_width,
        };

        // let fartherst_splat = splats
        //     .iter()
        //     .map(|splat| glm::distance(&center, &vec3(splat.x, splat.y, splat.z)))
        //     .max_by(|a, b| a.partial_cmp(b).unwrap())
        //     .unwrap();

        // let half_width = fartherst_splat * 2.0;

        // log!("center is {:?}", center);
        // log!("half width is {}", half_width);
        out.propogate_splats_to_children();

        return out;
    }
    //  fn index_to_direction(index: usize) -> Vec3 {
    //     match index {
    //         0 => Vec3 { x: 1.0,  y: 1.0,  z: 1.0 },
    //         1 => Vec3 { x: 1.0,  y: 1.0,  z: -1.0 },
    //         2 => Vec3 { x: 1.0,  y: -1.0, z: 1.0 },
    //         3 => Vec3 { x: 1.0,  y: -1.0, z: -1.0 },
    //         4 => Vec3 { x: -1.0, y: 1.0,  z: 1.0 },
    //         5 => Vec3 { x: -1.0, y: 1.0,  z: -1.0 },
    //         6 => Vec3 { x: -1.0, y: -1.0, z: 1.0 },
    //         7 => Vec3 { x: -1.0, y: -1.0, z: -1.0 },
    //         _ => panic!("Invalid index"),
    //     }
    // }
    fn index_to_color(index: usize) -> Vec3 {
        match index {
            0 => vec3(1.0, 0.0, 0.0),
            1 => vec3(0.0, 1.0, 0.0),
            2 => vec3(0.0, 0.0, 1.0),
            3 => vec3(1.0, 1.0, 0.0),
            4 => vec3(1.0, 0.0, 1.0),
            5 => vec3(0.0, 1.0, 1.0),
            6 => vec3(0.1, 0.8, 0.7),
            7 => vec3(0.4, 0.4, 0.4),
            _ => panic!("Invalid index"),
        }
    }

    fn index_to_direction(index: usize) -> Vec3 {
        vec3(
            if (index & 0b100) == 0 { 1.0 } else { -1.0 },
            if (index & 0b010) == 0 { 1.0 } else { -1.0 },
            if (index & 0b001) == 0 { 1.0 } else { -1.0 },
        )
    }

    fn get_cubes_of_children(&self) -> Vec<SceneObject> {
        let mut out = vec![];
        for (i, child) in self.children.iter().enumerate() {
            let color = OctTreeNode::index_to_color(i);
            if child.children.len() != 0 {
                let cubes = child.get_cubes_of_children();
                for cube in cubes {
                    out.push(cube);
                }
            } else {
                let cube = SceneObject::new_cube(child.center, child.half_width * 2.0, color);
                out.push(cube);
            }
        }
        return out;
    }

    fn propogate_splats_to_children(&mut self) {
        let len = self.splats.len();
        if len < SPLIT_LIMIT {
            return;
        }

        assert!(self.children.len() == 0, "octreenode already has children!");

        for i in 0..8 {
            let direction = OctTreeNode::index_to_direction(i);
            let new_center = self.center + direction * self.half_width / 2.0;

            let child = OctTreeNode::new(vec![], new_center, self.half_width / 2.0);
            self.children.push(child);
        }

        for splat in &self.splats {
            if splat.x > self.center.x {
                if splat.y > self.center.y {
                    if splat.z > self.center.z {
                        // top right back
                        self.children[0].splats.push(splat.clone());
                    } else {
                        // top right front
                        self.children[1].splats.push(splat.clone());
                    }
                } else {
                    if splat.z > self.center.z {
                        // bottom right back
                        self.children[2].splats.push(splat.clone());
                    } else {
                        // bottom right front
                        self.children[3].splats.push(splat.clone());
                    }
                }
            } else {
                if splat.y > self.center.y {
                    if splat.z > self.center.z {
                        // top left back
                        self.children[4].splats.push(splat.clone());
                    } else {
                        // top left front
                        self.children[5].splats.push(splat.clone());
                    }
                } else {
                    if splat.z > self.center.z {
                        // bottom left back
                        self.children[6].splats.push(splat.clone());
                    } else {
                        // bottom left front
                        self.children[7].splats.push(splat.clone());
                    }
                }
            }
        }

        for child in &mut self.children {
            log!("child has {} splats", child.splats.len());
            child.propogate_splats_to_children();
        }
    }
}

impl OctTree {
    pub fn new(splats: Vec<Splat>) -> Self {
        // let root = OctTreeNode::new(splats);
        let root = OctTreeNode::new(splats, vec3(0.0, 0.0, 0.0), 1.0);
        return OctTree { root: root };
    }

    pub fn get_cubes(&self) -> Vec<SceneObject> {
        return self.root.get_cubes_of_children();
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
