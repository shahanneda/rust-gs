#version 300 es
#pragma optimize(off)
#pragma debug(on)

in vec3 v_pos;
in vec3 s_color;
in vec3 s_center;
in vec3 s_cov3da;
in vec3 s_cov3db;

uniform mat4 model;
uniform mat4 camera;
uniform mat4 projection;

out mat4 x;
out vec3 color;

void main() {
  vec3 p_orig = s_center;
  mat4 projmatrix = projection * camera * model;

  vec4 p_hom = projmatrix * vec4(p_orig, 1);
  float p_w = 1. / (p_hom.w + 1e-7); // add 1e-7 so we don't divide by zero

  // Do the projection
  vec3 p_proj = p_hom.xyz * p_w;
  vec4 p_view = camera * vec4(p_orig, 1);
  float cov3D[6] = float[6](s_cov3da.x, s_cov3da.y, s_cov3da.z, s_cov3db.x, s_cov3db.y, s_cov3db.z);
  // vec3 cov = computeCov2D(p_orig, focal_x, focal_y, tan_fovx, tan_fovy, cov3D, viewmatrix);

  // gl_Position is a special variable a vertex shader
  // is responsible for setting
  // color = s_color;
  gl_Position = projection*camera*model*vec4(v_pos + s_center, 1.0);
  color = vec3(cov3D[0], cov3D[4], s_color[1]);
  color = vec3(cov3D[0], cov3D[1] , cov3D[2]);
  color = vec3(cov3D[3], cov3D[4] , cov3D[5]);
  // color = vec3(s_cov3da, s_cov3db)

  // if(gl_VertexID == 0){
  //     color = vec3(1,0,0);
  // }
  // else if(gl_VertexID == 1){
  //     color = vec3(0,1,0);
  // }
  // else if(gl_VertexID == 2){
  //     color = vec3(0,0,1);
  // }
  // else if(gl_VertexID == 3){
  //     color = vec3(1,1,0);
  // }
  // else{
  //     color = vec3(1, 1, 1);
  // }

  // color = vec3(1, 0, 0);
  // gl_PointSize = 1.0;
}