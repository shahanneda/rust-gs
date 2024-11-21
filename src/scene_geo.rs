pub static PYRAMID_COLORS: [f32; 54] = [
    // Red
    1.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    // Blue
    0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, // Green
    0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, // Yellow
    1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, // Cyan
    0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, // Magenta
    1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
];

pub static PYRAMID_VERTICES: [f32; 54] = [
    // 0.0, 0.0, 0.0,
    // 0.0, 1.0, 0.0,
    // 0.0, 0.0, 1.0,

    // 2.0, 0.0, 0.0,
    // 2.0, 1.0, 0.0,
    // 2.0, 0.0, 1.0,

    // 0.0, 0.0, 0.0,
    // 0.0, 1.0, 0.0,
    // 0.0, 0.0, -1.0,
    // Front face
    // -1.0, -1.0,  1.0, // Vertex 0
    // 1.0, -1.0,  1.0, // Vertex 1
    // -1.0,  1.0,  1.0, // Vertex 2
    // 1.0,  1.0,  1.0, // Vertex 3
    // // Back face
    // -1.0, -1.0, -1.0, // Vertex 4
    // 1.0, -1.0, -1.0, // Vertex 5
    // -1.0,  1.0, -1.0, // Vertex 6
    // 1.0,  1.0, -1.0, // Vertex 7
    // // Left face
    // -1.0, -1.0,  1.0, // Vertex 8
    // -1.0,  1.0,  1.0, // Vertex 9
    // -1.0, -1.0, -1.0, // Vertex 10
    // -1.0,  1.0, -1.0, // Vertex 11
    // // Right face
    // 1.0, -1.0,  1.0, // Vertex 12
    // 1.0,  1.0,  1.0, // Vertex 13
    // 1.0, -1.0, -1.0, // Vertex 14
    // 1.0,  1.0, -1.0, // Vertex 15
    // // Top face
    // -1.0,  1.0,  1.0, // Vertex 16
    // 1.0,  1.0,  1.0, // Vertex 17
    // -1.0,  1.0, -1.0, // Vertex 18
    // 1.0,  1.0, -1.0, // Vertex 19
    // -1.0,  1.0,  1.5, // Vertex 20
    // 1.0,  1.0,  1.5, // Vertex 21
    // // Bottom face
    // -1.0, -1.0,  1.0, // Vertex 22
    // 1.0, -1.0,  1.0, // Vertex 23
    // -1.0, -1.0, -1.0, // Vertex 24
    // 1.0, -1.0, -1.0, // Vertex 25,

    // 0.0, 3.0, 0.0,
    // 1.0, 3.0, 0.0,
    // 1.0, 3.0, 1.0,

    // 0.0, 3.0, 0.0,
    // 0.0, 3.0, 1.0,
    // 1.0, 3.0, 1.0,
    // -0.5, -0.5, 0.0, // bottom left
    // 0.5, -0.5, 0.0, // bottom right
    // 0.0, 0.5,
    // 0.0, // top middle
    // # Base
    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0,
    // # Side 1
    0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 0.0, 0.0, //
    // # Side 2
    1.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0, 0.0, // # Side 3
    1.0, 1.0, 0.0, 0.5, 0.5, 1.0, 0.0, 1.0, 0.0, // # Side 4
    0.0, 1.0, 0.0, 0.5, 0.5, 1.0, 0.0, 0.0, 0.0,
];

pub static CUBE_VERTICES: [f32; 108] = [
    // Front face
    -1.0, -1.0, 1.0, // bottom left
    1.0, -1.0, 1.0, // bottom right
    1.0, 1.0, 1.0, // top right
    -1.0, -1.0, 1.0, // bottom left
    1.0, 1.0, 1.0, // top right
    -1.0, 1.0, 1.0, // top left
    // Back face
    -1.0, -1.0, -1.0, // bottom left
    -1.0, 1.0, -1.0, // top left
    1.0, 1.0, -1.0, // top right
    -1.0, -1.0, -1.0, // bottom left
    1.0, 1.0, -1.0, // top right
    1.0, -1.0, -1.0, // bottom right
    // Top face
    -1.0, 1.0, -1.0, // back left
    -1.0, 1.0, 1.0, // front left
    1.0, 1.0, 1.0, // front right
    -1.0, 1.0, -1.0, // back left
    1.0, 1.0, 1.0, // front right
    1.0, 1.0, -1.0, // back right
    // Bottom face (fixed)
    -1.0, -1.0, -1.0, // back left
    1.0, -1.0, -1.0, // back right
    1.0, -1.0, 1.0, // front right
    -1.0, -1.0, -1.0, // back left
    1.0, -1.0, 1.0, // front right
    -1.0, -1.0, 1.0, // front left
    // Right face (fixed)
    1.0, -1.0, -1.0, // bottom back
    1.0, 1.0, -1.0, // top back
    1.0, 1.0, 1.0, // top front
    1.0, -1.0, -1.0, // bottom back
    1.0, 1.0, 1.0, // top front
    1.0, -1.0, 1.0, // bottom front
    // Left face (fixed)
    -1.0, -1.0, -1.0, // bottom back
    -1.0, -1.0, 1.0, // bottom front
    -1.0, 1.0, 1.0, // top front
    -1.0, -1.0, -1.0, // bottom back
    -1.0, 1.0, 1.0, // top front
    -1.0, 1.0, -1.0, // top back
];

pub static CUBE_COLORS: [f32; 108] = [
    // Front face (red)
    1.0, 0.0, 0.0, // bottom left
    1.0, 0.0, 0.0, // bottom right
    1.0, 0.0, 0.0, // top right
    1.0, 0.0, 0.0, // bottom left
    1.0, 0.0, 0.0, // top right
    1.0, 0.0, 0.0, // top left
    // Back face (green)
    0.0, 1.0, 0.0, // bottom left
    0.0, 1.0, 0.0, // top left
    0.0, 1.0, 0.0, // top right
    0.0, 1.0, 0.0, // bottom left
    0.0, 1.0, 0.0, // top right
    0.0, 1.0, 0.0, // bottom right
    // Top face (blue)
    0.0, 0.0, 1.0, // back left
    0.0, 0.0, 1.0, // front left
    0.0, 0.0, 1.0, // front right
    0.0, 0.0, 1.0, // back left
    0.0, 0.0, 1.0, // front right
    0.0, 0.0, 1.0, // back right
    // Bottom face (yellow)
    1.0, 1.0, 0.0, // back left
    1.0, 1.0, 0.0, // back right
    1.0, 1.0, 0.0, // front right
    1.0, 1.0, 0.0, // back left
    1.0, 1.0, 0.0, // front right
    1.0, 1.0, 0.0, // front left
    // Right face (magenta)
    1.0, 0.0, 1.0, // bottom back
    1.0, 0.0, 1.0, // top back
    1.0, 0.0, 1.0, // top front
    1.0, 0.0, 1.0, // bottom back
    1.0, 0.0, 1.0, // top front
    1.0, 0.0, 1.0, // bottom front
    // Left face (cyan)
    0.0, 1.0, 1.0, // bottom back
    0.0, 1.0, 1.0, // bottom front
    0.0, 1.0, 1.0, // top front
    0.0, 1.0, 1.0, // bottom back
    0.0, 1.0, 1.0, // top front
    0.0, 1.0, 1.0, // top back
];
