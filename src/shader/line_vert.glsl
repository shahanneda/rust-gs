#version 300 es
#pragma optimize(off)
#pragma debug(on)

in vec3 v_pos;
in vec3 v_col;
out vec3 v_color;

// uniform mat4 model;
uniform mat4 camera;
uniform mat4 projection;

uniform float W;
uniform float H;

// invert_row

// pub fn invert_row(mat: &mut glm::Mat4, row: usize) {
//     mat[row + 0] = -mat[row + 0];
//     mat[row + 4] = -mat[row + 4];
//     mat[row + 8] = -mat[row + 8];
//     mat[row + 12] = -mat[row + 12];
// }

mat4 invert_row(mat4 mat, int row) {
  mat[row + 0] = -mat[row + 0];
  mat[row + 4] = -mat[row + 4];
  mat[row + 8] = -mat[row + 8];
  mat[row + 12] = -mat[row + 12];
  return mat;
}

void main() {
  mat4 new_camera = invert_row(camera, 1);
  new_camera = invert_row(new_camera, 2);
  new_camera = invert_row(new_camera, 0);

  mat4 new_vpm = projection * camera;
  new_vpm = invert_row(new_vpm, 1);
  new_vpm = invert_row(new_vpm, 0);

  gl_Position = new_vpm * vec4(v_pos.x, v_pos.y, v_pos.z, 1.0);
  v_color = v_col;
}
