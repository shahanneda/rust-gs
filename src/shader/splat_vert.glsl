#version 300 es
#pragma optimize(off)
#pragma debug(on)

in vec3 v_pos;
in uint s_index;
uniform mat4 camera;
uniform mat4 projection;

uniform float W;
uniform float H;
uniform float focal_x;
uniform float focal_y;
uniform float tan_fovx;
uniform float tan_fovy;
uniform vec3 boxmin;
uniform vec3 boxmax;

out vec3 col;
out float depth;
out float scale_modif;
out vec4 con_o;
out vec2 xy;
out vec2 pixf;

uniform sampler2D u_color_texture;
uniform sampler2D u_position_texture;
uniform sampler2D u_cov3da_texture;
uniform sampler2D u_cov3db_texture;
uniform sampler2D u_opacity_texture;

const uint texture_width = 2000u;



vec3 computeCov2D(vec3 mean, float focal_x, float focal_y, float tan_fovx, float tan_fovy, float[6] cov3D, mat4 viewmatrix) {
    vec4 t = viewmatrix * vec4(mean, 1.0);

    float limx = 1.3 * tan_fovx;
    float limy = 1.3 * tan_fovy;
    float txtz = t.x / t.z;
    float tytz = t.y / t.z;
    t.x = min(limx, max(-limx, txtz)) * t.z;
    t.y = min(limy, max(-limy, tytz)) * t.z;

    mat3 J = mat3(
        focal_x / t.z, 0, -(focal_x * t.x) / (t.z * t.z),
        0, focal_y / t.z, -(focal_y * t.y) / (t.z * t.z),
        0, 0, 0
    );

    mat3 W =  mat3(
        viewmatrix[0][0], viewmatrix[1][0], viewmatrix[2][0],
        viewmatrix[0][1], viewmatrix[1][1], viewmatrix[2][1],
        viewmatrix[0][2], viewmatrix[1][2], viewmatrix[2][2]
    );

    mat3 T = W * J;

    mat3 Vrk = mat3(
        cov3D[0], cov3D[1], cov3D[2],
        cov3D[1], cov3D[3], cov3D[4],
        cov3D[2], cov3D[4], cov3D[5]
    );

    mat3 cov = transpose(T) * transpose(Vrk) * T;

    cov[0][0] += .3;
    cov[1][1] += .3;
    return vec3(cov[0][0], cov[0][1], cov[1][1]);
}

float ndc2Pix(float v, float S) {
    return ((v + 1.) * S - 1.) * .5;
}

vec3 get_value_from_texture(vec2 pixel_cord, sampler2D texture){
    ivec2 pixelCoord = ivec2(pixel_cord.x, pixel_cord.y);
    int mipLevel = 0;
    vec4 pixelValue = texelFetch(texture, pixelCoord, mipLevel);
    return pixelValue.rgb;
}

vec2 convert_splat_index_to_texture_index(uint splat_index){
    return vec2(splat_index % texture_width, splat_index / texture_width);
}

void main() {
    vec2 texture_coord = convert_splat_index_to_texture_index(s_index);

  vec3 s_cov3da = get_value_from_texture(texture_coord, u_cov3da_texture);
  vec3 s_cov3db = get_value_from_texture(texture_coord, u_cov3db_texture);
  float s_opacity = get_value_from_texture(texture_coord, u_opacity_texture).x;
  vec3 s_color = get_value_from_texture(texture_coord, u_color_texture);
  vec3 s_center = get_value_from_texture(texture_coord, u_position_texture);

//   vec3 p_orig = vec3(s_center.x, -s_center.y, s_center.z);
  vec3 p_orig = vec3(s_center.x, s_center.y, s_center.z);

  mat4 projmatrix = projection;
  vec4 p_hom = projmatrix * vec4(p_orig, 1);
  float p_w = 1. / (p_hom.w + 1e-7); // add 1e-7 so we don't divide by zero

  vec3 p_proj = p_hom.xyz * p_w;
  vec4 p_view = camera * vec4(p_orig, 1);

  // check if the splat is behind the camera
  // key difference is negative vs positive 
  if (p_view.z > 0.0) {
      gl_Position = vec4(0, 0, 0, 1);
      return;
  }

  

  float cov3D[6] = float[6](s_cov3da.x, s_cov3da.y, s_cov3da.z, s_cov3db.x, s_cov3db.y, s_cov3db.z);
  vec3 cov = computeCov2D(p_orig, focal_x, focal_y, tan_fovx, tan_fovy, cov3D, camera);

  float det = (cov.x * cov.z - cov.y * cov.y);
  if (det == 0.) {
      gl_Position = vec4(0, 0, 0, 1);
      return;
  }
  float det_inv = 1. / det;
  vec3 conic = vec3(cov.z, -cov.y, cov.x) * det_inv;

  float mid = 0.5 * (cov.x + cov.z);
  float lambda1 = mid + sqrt(max(0.1, mid * mid - det));
  float lambda2 = mid - sqrt(max(0.1, mid * mid - det));
  float my_radius = ceil(3. * sqrt(max(lambda1, lambda2)));
  vec2 point_image = vec2(ndc2Pix(p_proj.x, W), ndc2Pix(p_proj.y, H));

  float scale_modifier = 1.0;
  my_radius *= .15 + scale_modifier * .85;
  scale_modif = 1. / scale_modifier;

    vec2 corner = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2) - 1.;
    vec2 screen_pos = point_image + my_radius * corner;

    // if (corner == vec2(-1, -1)) {
    //     col = vec3(1,0,0);
    // }
    // else if (corner == vec2(1, -1)) {
    //     col = vec3(0,1,0);
    // }
    // else if (corner == vec2(1, 1)) {
    //     col = vec3(0,0,1);
    // }
    // else {
    //     col = vec3(1,1,0);
    // }
//   if(gl_VertexID == 0){
//       col = vec3(1,0,0);
//       corner = vec2(-1, -1);
//   }
//   else if(gl_VertexID == 1){
//       col = vec3(0,1,0);
//       corner = vec2(1, -1);
//   }
//   else if(gl_VertexID == 2){
//       col = vec3(0,0,1);
//       corner = vec2(1, 1);
//   }
//   else if(gl_VertexID == 3){
//       col = vec3(1,1,0);
//       corner = vec2(-1, 1);
//   }
//   else{
//       col = vec3(1, 1, 1);
//   }

  col = get_value_from_texture(texture_coord, u_color_texture);
  con_o = vec4(conic, s_opacity);
  xy = point_image;
  pixf = screen_pos;
  depth = p_view.z;

  vec2 clip_pos = screen_pos / vec2(W, H) * 2. - 1.;
  gl_Position = vec4(clip_pos, 0, 1);
}
