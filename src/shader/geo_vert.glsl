#version 300 es
#pragma optimize(off)
#pragma debug(on)

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

uniform float W;
uniform float H;

float ndc2Pix(float v, float S) { return ((v + 1.) * S - 1.) * .5; }

void main() {
  vec4 pos = projection * model * vec4(v_pos, 1.0) +
             W * H * 0.0 * camera * model * vec4(0., 0., 0., 0.);
  v_color = v_col;
  v_normal = normalize(mat3(transpose(inverse(model))) * v_norm);
  v_pos_out = vec3(model * vec4(v_pos, 1));
  vec4 p_view = camera * model * vec4(v_pos, 1);
  depth = p_view.z;
  gl_Position = vec4(pos.x, pos.y, pos.z, pos.w);
  // //   mat4 scaling_mat = mat4(100.0, 0.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0,
  // //   0.0,
  // //   0.0,
  // //   100.0, 0.0, 0.0, 0.0, 0.0, 1.0);

  // //   gl_Position = projection * camera * vec4(v_pos,
  // //   1) * 0.0 +
  // //                 projection * scaling_mat *
  // //                 vec4(v_pos, 1.0);
  // vec4 v_pos_model = model * vec4(v_pos, 1);

  // mat4 projmatrix = projection;

  // vec4 p_hom = projmatrix * v_pos_model;
  // vec4 p_view = camera * v_pos_model;

  // // gl_Position = vec4(0, 0, 0, 1);
  // // return
  // // if (p_hom.w > 1e-10) {
  // //   gl_Position = vec4(0, 0, 0, 1);
  // //   return;
  // // }

  // float p_w = 1. / (p_hom.w + 1e-7); // add 1e-7 so we don't divide by zero
  // vec3 p_proj = p_hom.xyz * p_w;

  // if (p_view.z >= -1e-7) {
  //   depth = p_view.z;
  //   gl_Position = vec4(-1, -1, 0, 1);
  //   v_color = vec3(0.0, 1.0, 0.0);
  //   return;
  // }

  // vec2 point_image = vec2(ndc2Pix(p_proj.x, W), ndc2Pix(p_proj.y, H));

  // // float scale_modifier = 1.0;
  // // my_radius *= .15 + scale_modifier * .85;
  // // scale_modif = 1. / scale_modifier;

  // // get a corner id either (-1, -1), (-1, 1), (1, -1),
  // // (1, 1)
  // //   vec2 corner = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2) - 1.;

  // // move the screen position of this verrtex to one of
  // // the corners of the splat
  // vec2 screen_pos = point_image;

  // // if (p_hom.w > 1e-10) {
  // //   gl_Position = vec4(0, 0, 0, 1);
  // //   reurn;
  // // }

  // //   col = get_value_from_texture(texture_coord, u_color_texture);
  // //   con_o = vec4(conic, s_opacity);
  // //   xy = point_image;
  // //   pixf = screen_pos;
  // depth = p_view.z;
  // vec2 clip_pos = screen_pos / vec2(W, H) * 2. - 1.;
  // gl_Position = vec4(clip_pos, 0, 1);
  // v_color = v_col;
}