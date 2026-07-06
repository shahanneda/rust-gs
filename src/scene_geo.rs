pub static CUBE_INDICES: [u32; 36] = [
    // Top face (+Y)
    2, 3, 7, 7, 6, 2, // Bottom face (-Y)
    0, 4, 5, 5, 1, 0, // Left face (-X)
    4, 0, 2, 2, 6, 4, // Right face (+X)
    1, 5, 7, 7, 3, 1, // Front face (+Z)
    0, 1, 3, 3, 2, 0, // Back face (-Z)
    5, 4, 6, 6, 7, 5,
];

// pub static CUBE_VERTICES: [f32; 72] = [
pub static CUBE_VERTICES: [f32; 24] = [
    -1.0, -1.0, 1.0, //0
    1.0, -1.0, 1.0, //1
    -1.0, 1.0, 1.0, //2
    1.0, 1.0, 1.0, //3
    -1.0, -1.0, -1.0, //4
    1.0, -1.0, -1.0, //5
    -1.0, 1.0, -1.0, //6
    1.0, 1.0, -1.0, //7
];

pub static CUBE_NORMALS: [f32; 24] = [
    -0.57735027,
    -0.57735027,
    0.57735027, // Vertex 0
    0.57735027,
    -0.57735027,
    0.57735027, // Vertex 1
    -0.57735027,
    0.57735027,
    0.57735027, // Vertex 2
    0.57735027,
    0.57735027,
    0.57735027, // Vertex 3
    -0.57735027,
    -0.57735027,
    -0.57735027, // Vertex 4
    0.57735027,
    -0.57735027,
    -0.57735027, // Vertex 5
    -0.57735027,
    0.57735027,
    -0.57735027, // Vertex 6
    0.57735027,
    0.57735027,
    -0.57735027, // Vertex 7
];
/// Generate a UV sphere of radius 1. Returns (vertices, indices, normals).
pub fn sphere_mesh(sectors: u32, stacks: u32) -> (Vec<f32>, Vec<u32>, Vec<f32>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0
            - (i as f32) * std::f32::consts::PI / (stacks as f32);
        let xy = stack_angle.cos();
        let z = stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = (j as f32) * 2.0 * std::f32::consts::PI / (sectors as f32);
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();
            vertices.extend_from_slice(&[x, y, z]);
            normals.extend_from_slice(&[x, y, z]);
        }
    }

    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;
        for _ in 0..sectors {
            if i != 0 {
                indices.extend_from_slice(&[k1, k2, k1 + 1]);
            }
            if i != stacks - 1 {
                indices.extend_from_slice(&[k1 + 1, k2, k2 + 1]);
            }
            k1 += 1;
            k2 += 1;
        }
    }

    (vertices, indices, normals)
}

/// Append a cylinder along +X from x0 to x1 with the given radius.
fn push_cylinder_x(
    vertices: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<f32>,
    x0: f32,
    x1: f32,
    radius: f32,
    sectors: u32,
) {
    let base = (vertices.len() / 3) as u32;
    for j in 0..=sectors {
        let a = (j as f32) * 2.0 * std::f32::consts::PI / (sectors as f32);
        let (y, z) = (a.cos() * radius, a.sin() * radius);
        let (ny, nz) = (a.cos(), a.sin());
        vertices.extend_from_slice(&[x0, y, z]);
        normals.extend_from_slice(&[0.0, ny, nz]);
        vertices.extend_from_slice(&[x1, y, z]);
        normals.extend_from_slice(&[0.0, ny, nz]);
    }
    for j in 0..sectors {
        let k = base + j * 2;
        indices.extend_from_slice(&[k, k + 1, k + 2, k + 2, k + 1, k + 3]);
    }
}

/// Append a cone along +X from x0 (base, given radius) to x1 (tip).
fn push_cone_x(
    vertices: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<f32>,
    x0: f32,
    x1: f32,
    radius: f32,
    sectors: u32,
) {
    let base = (vertices.len() / 3) as u32;
    for j in 0..=sectors {
        let a = (j as f32) * 2.0 * std::f32::consts::PI / (sectors as f32);
        let (y, z) = (a.cos() * radius, a.sin() * radius);
        vertices.extend_from_slice(&[x0, y, z]);
        normals.extend_from_slice(&[0.3, a.cos(), a.sin()]);
        vertices.extend_from_slice(&[x1, 0.0, 0.0]);
        normals.extend_from_slice(&[0.3, a.cos(), a.sin()]);
    }
    for j in 0..sectors {
        let k = base + j * 2;
        indices.extend_from_slice(&[k, k + 1, k + 2, k + 2, k + 1, k + 3]);
    }
    // base disc
    let disc = (vertices.len() / 3) as u32;
    vertices.extend_from_slice(&[x0, 0.0, 0.0]);
    normals.extend_from_slice(&[-1.0, 0.0, 0.0]);
    for j in 0..=sectors {
        let a = (j as f32) * 2.0 * std::f32::consts::PI / (sectors as f32);
        vertices.extend_from_slice(&[x0, a.cos() * radius, a.sin() * radius]);
        normals.extend_from_slice(&[-1.0, 0.0, 0.0]);
    }
    for j in 0..sectors {
        indices.extend_from_slice(&[disc, disc + 1 + j, disc + 2 + j]);
    }
}

/// Append an axis-aligned box centered at `center` with half-extents `h`.
fn push_box(
    vertices: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<f32>,
    center: [f32; 3],
    h: f32,
) {
    let base = (vertices.len() / 3) as u32;
    for i in 0..8 {
        let sx = if i & 1 == 0 { -1.0 } else { 1.0 };
        let sy = if i & 2 == 0 { -1.0 } else { 1.0 };
        let sz = if i & 4 == 0 { -1.0 } else { 1.0 };
        vertices.extend_from_slice(&[center[0] + sx * h, center[1] + sy * h, center[2] + sz * h]);
        let inv = (sx * sx + sy * sy + sz * sz).sqrt();
        normals.extend_from_slice(&[sx / inv, sy / inv, sz / inv]);
    }
    // 12 triangles over the 8 corners (same topology as CUBE_INDICES)
    let idx: [u32; 36] = [
        2, 3, 7, 7, 6, 2, 0, 4, 5, 5, 1, 0, 4, 0, 2, 2, 6, 4, 1, 5, 7, 7, 3, 1, 0, 1, 3, 3, 2, 0,
        5, 4, 6, 6, 7, 5,
    ];
    indices.extend(idx.iter().map(|i| base + i));
}

/// Translation-gizmo arrow: shaft + cone tip pointing along +X, ~1 unit long.
pub fn arrow_mesh() -> (Vec<f32>, Vec<u32>, Vec<f32>) {
    let mut v = Vec::new();
    let mut i = Vec::new();
    let mut n = Vec::new();
    push_cylinder_x(&mut v, &mut i, &mut n, 0.0, 0.78, 0.028, 14);
    push_cone_x(&mut v, &mut i, &mut n, 0.78, 1.05, 0.085, 14);
    (v, i, n)
}

/// Rotation-gizmo ring: torus of radius `ring_r` in the XY plane (normal +Z).
pub fn torus_mesh(ring_r: f32, tube_r: f32) -> (Vec<f32>, Vec<u32>, Vec<f32>) {
    let (segments, sides) = (48u32, 10u32);
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut i = Vec::new();
    for s in 0..=segments {
        let a = (s as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
        let (ca, sa) = (a.cos(), a.sin());
        for t in 0..=sides {
            let b = (t as f32) * 2.0 * std::f32::consts::PI / (sides as f32);
            let (cb, sb) = (b.cos(), b.sin());
            let r = ring_r + tube_r * cb;
            v.extend_from_slice(&[r * ca, r * sa, tube_r * sb]);
            n.extend_from_slice(&[cb * ca, cb * sa, sb]);
        }
    }
    for s in 0..segments {
        for t in 0..sides {
            let k = s * (sides + 1) + t;
            i.extend_from_slice(&[k, k + sides + 1, k + 1, k + 1, k + sides + 1, k + sides + 2]);
        }
    }
    (v, i, n)
}

/// Scale-gizmo handle: shaft along +X with a small cube at the end.
pub fn scale_handle_mesh() -> (Vec<f32>, Vec<u32>, Vec<f32>) {
    let mut v = Vec::new();
    let mut i = Vec::new();
    let mut n = Vec::new();
    push_cylinder_x(&mut v, &mut i, &mut n, 0.0, 0.82, 0.028, 14);
    push_box(&mut v, &mut i, &mut n, [0.88, 0.0, 0.0], 0.075);
    (v, i, n)
}

/// Uniform-scale center handle: a small cube at the origin.
pub fn center_cube_mesh() -> (Vec<f32>, Vec<u32>, Vec<f32>) {
    let mut v = Vec::new();
    let mut i = Vec::new();
    let mut n = Vec::new();
    push_box(&mut v, &mut i, &mut n, [0.0, 0.0, 0.0], 0.12);
    (v, i, n)
}

pub static CUBE_COLORS: [f32; 24] = [
    1.0, 0.0, 0.0, // Vertex 0
    1.0, 0.0, 0.0, // Vertex 1
    1.0, 0.0, 0.0, // Vertex 2
    1.0, 0.0, 0.0, // Vertex 3
    1.0, 0.0, 0.0, // Vertex 0
    1.0, 0.0, 0.0, // Vertex 1
    1.0, 0.0, 0.0, // Vertex 2
    1.0, 0.0, 0.0,
];
