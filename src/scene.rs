use nalgebra_glm::{exp, mat3_to_quat, pi, quat_to_mat3, radians, vec3, vec4, Vec3, Vec4};
use crate::log;

use crate::{ply_splat::PlySplat, utils::sigmoid};


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
        // mat3.set(S, mod * scale[0], 0, 0, 0, mod * scale[1], 0, 0, 0, mod * scale[2]);
        let scaling_mat = glm::mat3(md*scale[0], 0.0, 0.0, 0.0, md*scale[1], 0.0, 0.0, 0.0, md*scale[2]);

        let quat = glm::Quat::new(rot.w, rot.x, rot.y, rot.z);
        let rot_mat = quat_to_mat3(&quat);
        let matrix = scaling_mat * rot_mat;
        let sigma = matrix.transpose() * matrix;
        // log!("{sigma}");
        let cov3d = [sigma[0], sigma[1], sigma[2], sigma[4], sigma[5], sigma[8]];
        // log!("{cov3d:?}");
        // 0 1 2
        // 3 4 5 
        // 6 7 8

        // const r = rot[0];
        // const x = rot[1];
        // const y = rot[2];
        // const z = rot[3];
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
            cov3d: Splat::compute_cov3_d(vec3(1.0, 2.0, 1.0), 1.0, vec4(0.0, 0.0, 0.0, 1.0))
            // cov3d: [0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        };

        // return splat;
    }
    
}

pub struct Scene{
    pub splats: Vec<Splat>,
}

impl Scene {
    pub fn new(splats: Vec<PlySplat>) -> Self {
        let splats = splats.iter().map(|splat| Splat::new(splat)).collect();
        // let splats = splats.iter().take(1).map(|splat| Splat::new(splat)).collect();

        return Scene {
            splats: splats,
        };
    }

    pub fn sort_splats_based_on_depth(&mut self, view_matrix: glm::Mat4){
        let calc_depth = |splat: &Splat| {
            splat.x * view_matrix[2] +
            splat.y * view_matrix[6] +
            splat.z * view_matrix[10]
        };

        self.splats.sort_by(|a, b| 
            calc_depth(&a).partial_cmp(&calc_depth(&b)).unwrap())
        // const calcDepth = (i) =>
        //     gaussians.positions[i * 3] * viewMatrix[2] +
        //     gaussians.positions[i * 3 + 1] * viewMatrix[6] +
        //     gaussians.positions[i * 3 + 2] * viewMatrix[10];
            
            // 0 1 2 3
            // 4 5 6 7
            // 8 9 10 11
            // 12 13 14 15
            // a.z.partial_cmp(&b.z).unwrap())
        // ;
    }
}