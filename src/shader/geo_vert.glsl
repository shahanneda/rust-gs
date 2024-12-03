#version 300 es
// #pragma optimize(off)
// #pragma debug(on)

in vec3 v_pos;
in vec3 v_col;
in vec3 v_norm;
out vec3 v_color;
out vec3 v_normal;
out float depth;
out vec3 v_pos_out;

uniform mat4 model;
uniform mat4 camera;
uniform mat4 projection;

float ndc2Pix(float v, float S) { return ((v + 1.) * S - 1.) * .5; }

void main() {
  vec4 pos = projection * model * vec4(v_pos, 1.0);
  v_color = v_col;
  v_normal = normalize(mat3(transpose(inverse(model))) * v_norm);
  v_pos_out = vec3(model * vec4(v_pos, 1));
  vec4 p_view = camera * model * vec4(v_pos, 1);
  depth = p_view.z;
  gl_Position = vec4(pos.x, pos.y, pos.z, pos.w);
}