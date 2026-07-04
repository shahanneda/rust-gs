//! Compact binary splat format ("GSZ1"), ~26 bytes per splat instead of the
//! ~92 bytes rkyv uses for a full `Splat`.
//!
//! Layout (little-endian, structure-of-arrays):
//!   bytes 0..4   magic b"GSZ1"
//!   bytes 4..8   splat count `n` (u32)
//!   positions    n * 3 * f32   (full precision — needed for octree/editing)
//!   scales       n * 3 * f16   (linear scale, already exp()'d)
//!   rotations    n * 4 * u8    (normalized quaternion, q * 127 + 128)
//!   colors       n * 4 * u8    (r, g, b, opacity, all [0,1] * 255)
//!
//! Normals are dropped (unused by the renderer) and cov3d is recomputed from
//! scale+rotation at load time.

use crate::splat::Splat;
use half::f16;
use nalgebra_glm::{vec3, vec4};
use std::convert::TryInto;

pub const MAGIC: &[u8; 4] = b"GSZ1";
const HEADER_SIZE: usize = 8;

pub fn is_packed_format(bytes: &[u8]) -> bool {
    bytes.len() >= HEADER_SIZE && &bytes[0..4] == MAGIC
}

fn quantize_unit(v: f32) -> u8 {
    ((v * 127.0) + 128.0).round().clamp(0.0, 255.0) as u8
}

fn dequantize_unit(b: u8) -> f32 {
    (b as f32 - 128.0) / 127.0
}

fn quantize_01(v: f32) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn encode(splats: &[Splat]) -> Vec<u8> {
    let n = splats.len();
    let total = HEADER_SIZE + n * (12 + 6 + 4 + 4);
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(n as u32).to_le_bytes());

    for s in splats {
        out.extend_from_slice(&s.x.to_le_bytes());
        out.extend_from_slice(&s.y.to_le_bytes());
        out.extend_from_slice(&s.z.to_le_bytes());
    }
    for s in splats {
        out.extend_from_slice(&f16::from_f32(s.scale_0).to_le_bytes());
        out.extend_from_slice(&f16::from_f32(s.scale_1).to_le_bytes());
        out.extend_from_slice(&f16::from_f32(s.scale_2).to_le_bytes());
    }
    for s in splats {
        // Renormalize so quantization error stays small even if the source
        // quaternion drifted from unit length.
        let q = vec4(s.rot_0, s.rot_1, s.rot_2, s.rot_3);
        let len = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
        let q = if len > 1e-8 { q / len } else { vec4(1.0, 0.0, 0.0, 0.0) };
        out.push(quantize_unit(q.x));
        out.push(quantize_unit(q.y));
        out.push(quantize_unit(q.z));
        out.push(quantize_unit(q.w));
    }
    for s in splats {
        out.push(quantize_01(s.r));
        out.push(quantize_01(s.g));
        out.push(quantize_01(s.b));
        out.push(quantize_01(s.opacity));
    }
    out
}

pub fn decode(bytes: &[u8]) -> Result<Vec<Splat>, String> {
    if !is_packed_format(bytes) {
        return Err("not a GSZ1 file".to_string());
    }
    let n = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;

    let pos_start = HEADER_SIZE;
    let scale_start = pos_start + n * 12;
    let rot_start = scale_start + n * 6;
    let color_start = rot_start + n * 4;
    let end = color_start + n * 4;
    if bytes.len() < end {
        return Err(format!(
            "GSZ1 file truncated: expected {} bytes, got {}",
            end,
            bytes.len()
        ));
    }

    let positions = &bytes[pos_start..scale_start];
    let scales = &bytes[scale_start..rot_start];
    let rotations = &bytes[rot_start..color_start];
    let colors = &bytes[color_start..end];

    let read_f32 = |buf: &[u8], i: usize| -> f32 {
        f32::from_le_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap())
    };
    let read_f16 = |buf: &[u8], i: usize| -> f32 {
        f16::from_le_bytes(buf[i * 2..i * 2 + 2].try_into().unwrap()).to_f32()
    };

    let mut splats = Vec::with_capacity(n);
    for i in 0..n {
        let x = read_f32(positions, i * 3);
        let y = read_f32(positions, i * 3 + 1);
        let z = read_f32(positions, i * 3 + 2);

        let scale = vec3(
            read_f16(scales, i * 3),
            read_f16(scales, i * 3 + 1),
            read_f16(scales, i * 3 + 2),
        );

        let mut rot = vec4(
            dequantize_unit(rotations[i * 4]),
            dequantize_unit(rotations[i * 4 + 1]),
            dequantize_unit(rotations[i * 4 + 2]),
            dequantize_unit(rotations[i * 4 + 3]),
        );
        let len = (rot.x * rot.x + rot.y * rot.y + rot.z * rot.z + rot.w * rot.w).sqrt();
        if len > 1e-8 {
            rot /= len;
        } else {
            rot = vec4(1.0, 0.0, 0.0, 0.0);
        }

        let r = colors[i * 4] as f32 / 255.0;
        let g = colors[i * 4 + 1] as f32 / 255.0;
        let b = colors[i * 4 + 2] as f32 / 255.0;
        let opacity = colors[i * 4 + 3] as f32 / 255.0;

        splats.push(Splat::from_decoded(x, y, z, scale, rot, r, g, b, opacity));
    }
    Ok(splats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let splat = Splat::from_decoded(
            1.5,
            -2.25,
            0.125,
            vec3(0.01, 0.02, 0.03),
            vec4(0.5f32.sqrt(), 0.5f32.sqrt(), 0.0, 0.0),
            0.25,
            0.5,
            0.75,
            0.9,
        );
        let bytes = encode(&[splat]);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(decoded.len(), 1);
        let d = &decoded[0];
        assert_eq!(d.x, 1.5);
        assert_eq!(d.y, -2.25);
        assert_eq!(d.z, 0.125);
        assert!((d.scale_0 - 0.01).abs() < 1e-4);
        assert!((d.r - 0.25).abs() < 0.005);
        assert!((d.opacity - 0.9).abs() < 0.005);
        assert!((d.rot_0 - 0.5f32.sqrt()).abs() < 0.01);
    }
}
