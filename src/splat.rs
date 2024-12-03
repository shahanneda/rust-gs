use nalgebra_glm::{exp, mat3, mat3_to_quat, quat_to_mat3, vec3, vec4, Quat, Vec3, Vec4};
use rkyv::{Archive, Deserialize, Serialize};

use crate::{ply_splat::PlySplat, shared_utils::sigmoid};
use nalgebra_glm as glm;

#[derive(Clone, Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub struct Splat {
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
    pub cov3d: [f32; 6],
}

impl Copy for Splat {}

impl Splat {
    fn compute_cov3_d(scale: Vec3, md: f32, rot: Vec4) -> [f32; 6] {
        let scaling_mat = mat3(
            md * scale[0],
            0.0,
            0.0,
            0.0,
            md * scale[1],
            0.0,
            0.0,
            0.0,
            md * scale[2],
        );
        let w = rot[0];
        let x = rot[1];
        let y = rot[2];
        let z = rot[3];

        let quat = Quat::new(w, x, y, z);
        let rot_mat = quat_to_mat3(&quat);

        let matrix = scaling_mat * rot_mat.transpose();
        let sigma = matrix.transpose() * matrix;
        let cov3d = [sigma[0], sigma[1], sigma[2], sigma[4], sigma[5], sigma[8]];
        return cov3d;
    }

    fn rgb_from_sh(f_dc_0: f32, f_dc_1: f32, f_dc_2: f32) -> [f32; 3] {
        const SH_C0: f32 = 0.28209479177387814;
        let r = 0.5 + SH_C0 * f_dc_0;
        let g = 0.5 + SH_C0 * f_dc_1;
        let b = 0.5 + SH_C0 * f_dc_2;
        return [r, g, b];
    }

    pub fn new(ply_splat: &PlySplat) -> Self {
        // log!("new individual splat");
        // let _timer = Timer::new("new individual splat");
        let rgb = Splat::rgb_from_sh(ply_splat.f_dc_0, ply_splat.f_dc_1, ply_splat.f_dc_2);

        let rot = vec4(
            ply_splat.rot_0,
            ply_splat.rot_1,
            ply_splat.rot_2,
            ply_splat.rot_3,
        )
        .normalize();
        let scale = exp(&vec3(
            ply_splat.scale_0,
            ply_splat.scale_1,
            ply_splat.scale_2,
        ));

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
            cov3d: Splat::compute_cov3_d(scale, 1.0, rot),
        };
        return splat;
    }
}
