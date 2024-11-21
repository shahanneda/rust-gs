#version 300 es
#pragma optimize(off)
#pragma debug(on)

in vec3 v_pos;
in vec3 v_col;
out vec3 v_color;
out float depth;

uniform mat4 model;
uniform mat4 camera;
uniform mat4 projection;

uniform float W;
uniform float H;

float ndc2Pix(float v, float S) { return ((v + 1.) * S - 1.) * .5; }

void main() {
  //   mat4 scaling_mat = mat4(100.0, 0.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0,
  //   0.0,
  //   100.0, 0.0, 0.0, 0.0, 0.0, 1.0);

  //   gl_Position = projection * camera * vec4(v_pos,
  //   1) * 0.0 +
  //                 projection * scaling_mat *
  //                 vec4(v_pos, 1.0);
  vec4 v_pos_model = model * vec4(v_pos, 1);

  mat4 projmatrix = projection;

  vec4 p_hom = projmatrix * v_pos_model;
  vec4 p_view = camera * v_pos_model;

  float p_w = 1. / (p_hom.w + 1e-7); // add 1e-7 so we don't divide by zero
  vec3 p_proj = p_hom.xyz * p_w;

  // check if the splat is behind the camera
  // key difference is negative vs positive
  if (p_view.z > 0.0) {
    gl_Position = vec4(0, 0, 0, 1);
    return;
  }

  vec2 point_image = vec2(ndc2Pix(p_proj.x, W), ndc2Pix(p_proj.y, H));

  // float scale_modifier = 1.0;
  // my_radius *= .15 + scale_modifier * .85;
  // scale_modif = 1. / scale_modifier;

  // get a corner id either (-1, -1), (-1, 1), (1, -1),
  // (1, 1)
  //   vec2 corner = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2) - 1.;

  // move the screen position of this verrtex to one of
  // the corners of the splat
  vec2 screen_pos = point_image;

  //   col = get_value_from_texture(texture_coord, u_color_texture);
  //   con_o = vec4(conic, s_opacity);
  //   xy = point_image;
  //   pixf = screen_pos;
  depth = p_view.z;
  vec2 clip_pos = screen_pos / vec2(W, H) * 2. - 1.;
  gl_Position = vec4(clip_pos, 0, 1);
  v_color = v_col;
}