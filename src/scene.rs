use bytes::buf;
use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec3, Vec4};
// use serde::{Deserialize, Serialize};
use crate::log;

use crate::timer::Timer;
use crate::{ply_splat::PlySplat, shared_utils::sigmoid};
use nalgebra_glm as glm;
use rkyv::{deserialize, rancor::Error, Archive, Deserialize, Serialize};
// use speedy::{Readable, Writable, Endianness};




// #[derive(Clone, Serialize, Deserialize, Debug)]
#[derive(Clone, Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
// #[derive(Clone, PartialEq, Debug, Readable, Writable)]
pub struct Splat{
        pub nx: f32,
        pub ny: f32,
        pub nz: f32,
        pub opacity: f32,
        pub rot_0: f32,
        pub rot_1: f32,
        pub rot_2: f32,
        pub rot_3: f32,
        pub scale_0: f32,
        pub scale_1: f32,
        pub scale_2: f32,
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub cov3d: [f32; 6]
}

// TODO: Remove copying of splats when sorting 
impl Copy for Splat {}


impl Splat{
// function computeCov3D(scale, mod, rot) {
//   //   console.log("computing cov 3d");
//   // Create scaling matrix
//   mat3.set(S, mod * scale[0], 0, 0, 0, mod * scale[1], 0, 0, 0, mod * scale[2]);
//   const r = rot[0];
//   const x = rot[1];
//   const y = rot[2];
//   const z = rot[3];

//   // Compute rotation matrix from quaternion
//   mat3.set(
//     R,
//     1 - 2 * (y * y + z * z),
//     2 * (x * y - r * z),
//     2 * (x * z + r * y),
//     2 * (x * y + r * z),
//     1 - 2 * (x * x + z * z),
//     2 * (y * z - r * x),
//     2 * (x * z - r * y),
//     2 * (y * z + r * x),
//     1 - 2 * (x * x + y * y)
//   );

//   mat3.multiply(M, S, R); // M = S * R

//   // Compute 3D world covariance matrix Sigma
//   mat3.multiply(Sigma, mat3.transpose(tmp, M), M); // Sigma = transpose(M) * M

//   // Covariance is symmetric, only store upper right
//   const cov3D = [Sigma[0], Sigma[1], Sigma[2], Sigma[4], Sigma[5], Sigma[8]];

//   return cov3D;
// }

    fn compute_cov3_d(scale: Vec3, md: f32, rot: Vec4) -> [f32; 6] {
        log!("scale is {:?}", scale);
        log!("md is {:?}", md);
        log!("rot is {:?}", rot);
        // mat3.set(S, mod * scale[0], 0, 0, 0, mod * scale[1], 0, 0, 0, mod * scale[2]);
        let scaling_mat = glm::mat3(md*scale[0], 0.0, 0.0, 0.0, md*scale[1], 0.0, 0.0, 0.0, md*scale[2]);
        let w = rot[0];
        let x = rot[1];
        let y = rot[2];
        let z = rot[3];

        let quat = glm::Quat::new(w, x, y, z);
        let rot_mat = quat_to_mat3(&quat);
        log!("rot_mat is {:?}", rot_mat);
        let matrix = scaling_mat * rot_mat.transpose();
        log!("scaling_mat is {:?}", scaling_mat);
        log!("matrix is {:?}", matrix);
        let sigma = matrix.transpose() * matrix;
        // let sigma = matrix * matrix.transpose();
        // log!("{sigma}");
        // Only store upper right since it's symmetric
        let cov3d = [sigma[0], sigma[1], sigma[2], sigma[4], sigma[5], sigma[8]];
        // log!("{cov3d:?}");
        // 0 1 2
        // 3 4 5 
        // 6 7 8

        // const r = rot[0];
        // const x = rot[1];
        // const y = rot[2];
        // const z = rot[3];
        log!("cov3d is {:?}", cov3d);
        return cov3d;
    }

    fn rgb_from_sh(f_dc_0: f32, f_dc_1: f32, f_dc_2: f32) -> [f32; 3]{
        const SH_C0: f32 = 0.28209479177387814;
        let r = 0.5 + SH_C0* f_dc_0;
        let g = 0.5 + SH_C0* f_dc_1;
        let b = 0.5 + SH_C0 *f_dc_2;
        return [r, g, b];
    }

    pub fn new(ply_splat: &PlySplat) -> Self {
        // log!("new individual splat");
        // let _timer = Timer::new("new individual splat");
        let rgb = Splat::rgb_from_sh(ply_splat.f_dc_0, ply_splat.f_dc_1, ply_splat.f_dc_2);

        let rot = vec4(ply_splat.rot_0, ply_splat.rot_1, ply_splat.rot_2, ply_splat.rot_3).normalize();
        let scale = exp(&vec3(ply_splat.scale_0, ply_splat.scale_1, ply_splat.scale_2));
        
        let splat = Splat {
            nx: ply_splat.nx,
            ny: ply_splat.ny,
            nz: ply_splat.nz,
            opacity: sigmoid(ply_splat.opacity),
            rot_0: rot.x,
            rot_1: rot.y,
            rot_2: rot.z,
            rot_3: rot.w,
            scale_0: scale.x,
            scale_1: scale.y,
            scale_2: scale.z,
            x: ply_splat.x,
            y: ply_splat.y,
            z: ply_splat.z,
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            cov3d: Splat::compute_cov3_d(scale, 1.0, rot)
        };
        return splat;

        let rad = 120.0 * std::f32::consts::PI / 180.0;
        let quat = mat3_to_quat(&glm::mat4_to_mat3(&glm::rotation(rad, &glm::vec3(0.0, 0.0, 1.0))));
        // quat.coords()
        // quat.as_vector()
        // log!("cov is");
        // log!("{:?}", Splat::compute_cov3_d(vec3(1.0, 2.0, 1.0), 1.0, vec4(0.0, 0.0, 0.0, 1.0)));
        return Splat{
            nx: ply_splat.nx,
            ny: ply_splat.ny,
            nz: ply_splat.nz,
            // opacity: sigmoid(ply_splat.opacity),
            opacity: 1.0,
            rot_0: rot.x,
            rot_1: rot.y,
            rot_2: rot.z,
            rot_3: rot.w,
            scale_0: scale.x,
            scale_1: scale.y,
            scale_2: scale.z,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            r: 1.0,
            g: 0.0,
            b: 0.0,
            cov3d: Splat::compute_cov3_d(vec3(0.01, 0.05, 0.01), 1.0, vec4(0.0, 0.0, 0.0, 1.0))
            // cov3d: [0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        };

        // return splat;
    }
    
}

// #[derive(Serialize, Deserialize, Debug)]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub struct Scene{
    pub splats: Vec<Splat>,
}

impl Scene {
    pub async fn new_from_url(url: &str) -> Self {

        let _timer = Timer::new("loading json file");
        let loaded_file = reqwest::get(url)
            .await
            .expect("error")
            .bytes()
            .await
            .expect("went wrong when reading!");
        return Scene::new_from_json(&loaded_file);
    }

    pub fn new_from_json(bytes: &[u8]) -> Self {
        let _timer = Timer::new("new scene from json");
        log!("Creating a new scene from json");
        
        match rkyv::from_bytes::<Scene, Error>(bytes) {
            Ok(mut scene) => {
                // only take 100 splats
                // scene.splats.truncate(5);
                log!("scene has {} splats", scene.splats.len());
                log!("scene has {:?}", scene.splats);
                return scene;
            },
            Err(e) => {
                // Handle the error appropriately. For now, we'll panic with a message.
                panic!("Failed to deserialize scene: {:?}", e);
            }
        }
    }

    pub fn new(splats: Vec<PlySplat>) -> Self {
        let _timer = Timer::new("new scene");
        log!("Creating a new scene");
        let splats = splats.iter().map(|splat| Splat::new(splat)).collect();
        // let splats = splats.iter().take(1).map(|splat| Splat::new(splat)).collect();

        return Scene {
            splats: splats,
        };
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

    pub fn compress_splats_into_buffer(&self) -> Vec<u8>{
        let num_properties_per_splat = 15;
        let mut buffer = vec![0.0; self.splat_count() * num_properties_per_splat];

        for i in 0..self.splat_count(){

            // s_color, s_center, s_cov3da, s_cov3db, s_opacity;
            let splat = &self.splats[i];

            buffer[i*num_properties_per_splat + 0] = splat.r;
            buffer[i*num_properties_per_splat + 1] = splat.g;
            buffer[i*num_properties_per_splat + 2] = splat.b;

            buffer[i*num_properties_per_splat + 3] = splat.x;
            buffer[i*num_properties_per_splat + 4] = splat.y;
            buffer[i*num_properties_per_splat + 5] = splat.z;

            buffer[i*num_properties_per_splat + 6] = splat.cov3d[0];
            buffer[i*num_properties_per_splat + 7] = splat.cov3d[1];
            buffer[i*num_properties_per_splat + 8] = splat.cov3d[2];
            buffer[i*num_properties_per_splat + 9] = splat.cov3d[3];
            buffer[i*num_properties_per_splat + 10] = splat.cov3d[4];
            buffer[i*num_properties_per_splat + 11] = splat.cov3d[5];

            buffer[i*num_properties_per_splat + 12] = splat.opacity;
            buffer[i*num_properties_per_splat + 13] = splat.nx;
            buffer[i*num_properties_per_splat + 14] = splat.ny;
        }

        let mut out : Vec<u8> = vec![0; self.nearest_power_of_2_bigger_than(buffer.len()*4)];
        for i in 0..buffer.len(){
            f32_to_4_bytes(buffer[i]).iter()
                .enumerate()
                .for_each(|(j, &byte)| out[i*4 + j] = byte);
        }
        return out;
    }


    pub fn sort_splats_based_on_depth(&mut self, view_matrix: glm::Mat4){
        let _timer = Timer::new("sort_splats_based_on_depth");
        // track start time
        let calc_depth = |splat: &Splat| {
            ((splat.x * view_matrix[2] +
            splat.y * view_matrix[6] +
            splat.z * view_matrix[10])*1_000.0) as i32
        };

        // let mut pos_count = 0;
        // let mut neg_count = 0;
        // for splat in &self.splats{
        //     let depth = calc_depth(&splat);
        //     if depth > 0{
        //         pos_count += 1;
        //     } else {
        //         neg_count += 1;
        //     }
        // }

        // log!("pos count is {pos_count} neg count is {neg_count}");


        // let mut max_depth = i32::MIN;
        // let mut min_depth = 0;
        // let splat_count = self.splats.len();

        // let mut depth_list = self.splats.iter().map(|splat| {
        //     let depth = calc_depth(&splat);
        //     max_depth = max_depth.max(depth as i32);
        //     min_depth = min_depth.min(depth as i32);
        //     depth
        // }).collect::<Vec<i32>>();

        // let mut count_array = vec![0; (max_depth - min_depth +1) as usize];

        // // Count the number of splats at each depth
        // // log!("max is {max_depth} min is {min_depth}");
        // for i in 0..splat_count{
        //     depth_list[i] -= min_depth;
        //     count_array[depth_list[i] as usize] += 1;
        // }

        // // Do prefix sum
        // for i in 1..count_array.len(){
        //     count_array[i] += count_array[i-1];
        // }

        // {
        //     let _timer = Timer::new("creating output vector");
        //     let mut output : Vec<Splat> = vec![Splat{
        //         nx: 0.0,
        //         ny: 0.0,
        //         nz: 0.0,
        //         opacity: 0.0,
        //         rot_0: 0.0,
        //         rot_1: 0.0,
        //         rot_2: 0.0,
        //         rot_3: 0.0,
        //         scale_0: 0.0,
        //         scale_1: 0.0,
        //         scale_2: 0.0,
        //         x: 0.0,
        //         y: 0.0,
        //         z: 0.0,
        //         r: 0.0,
        //         g: 0.0,
        //         b: 0.0,
        //         cov3d: [0.0; 6]
        //     }; self.splats.len()];

        //     for i in (0..self.splats.len()).rev(){
        //         let depth = depth_list[i];
        //         // if depth > 0 {
        //         //     self.splats[i].opacity = 0.0;
        //         // }

        //         let index = count_array[depth as usize] - 1;
        //         // log!("depth is {depth} index is {index} i is {i}");
        //         // TODO: Remove copying of splats when sorting
        //         output[index as usize] = self.splats[i];
        //         count_array[depth as usize] -= 1;
        //     }

        //     // output.reverse();
        //     self.splats = output;
        // }


        self.splats.sort_by(|a, b| 
            calc_depth(&b).partial_cmp(&calc_depth(&a)).unwrap());
        // const calcDepth = (i) =>
        //     gaussians.positions[i * 3] * viewMatrix[2] +
        //     gaussians.positions[i * 3 + 1] * viewMatrix[6] +
        //     gaussians.positions[i * 3 + 2] * viewMatrix[10];
            
        //     0 1 2 3
        //     4 5 6 7
        //     8 9 10 11
        //     12 13 14 15
        //     a.z.partial_cmp(&b.z).unwrap())
        // ;
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
