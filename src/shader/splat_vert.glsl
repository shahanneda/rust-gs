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
uniform float scale;
uniform vec3 boxmin;
uniform vec3 boxmax;
uniform mat4 model;

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

vec3 computeCov2D(vec3 mean, float focal_x, float focal_y, float tan_fovx,
                  float tan_fovy, float[6] cov3D, mat4 viewmatrix) {
  vec4 t = viewmatrix * vec4(mean, 1.0);

  float limx = 1.3 * tan_fovx;
  float limy = 1.3 * tan_fovy;
  float txtz = t.x / t.z;
  float tytz = t.y / t.z;
  t.x = min(limx, max(-limx, txtz)) * t.z;
  t.y = min(limy, max(-limy, tytz)) * t.z;

  mat3 J = mat3(focal_x / t.z, 0, -(focal_x * t.x) / (t.z * t.z), 0,
                focal_y / t.z, -(focal_y * t.y) / (t.z * t.z), 0, 0, 0);

  mat3 W = mat3(viewmatrix[0][0], viewmatrix[1][0], viewmatrix[2][0],
                viewmatrix[0][1], viewmatrix[1][1], viewmatrix[2][1],
                viewmatrix[0][2], viewmatrix[1][2], viewmatrix[2][2]);

  mat3 T = W * J;

  // since 3D covariance is a symmetric matrix
  mat3 cov3DMatrix = mat3(cov3D[0], cov3D[1], cov3D[2], cov3D[1], cov3D[3],
                          cov3D[4], cov3D[2], cov3D[4], cov3D[5]);

  mat3 cov = transpose(T) * transpose(cov3DMatrix) * T;

  // im not sure why we need to do this but it helps a lot
  cov[0][0] += .3;
  cov[1][1] += .3;

  // 2d covariance is also symmetric
  return vec3(cov[0][0], cov[0][1], cov[1][1]);
}

float ndc2Pix(float v, float S) { return ((v + 1.) * S - 1.) * .5; }

vec3 get_value_from_texture(vec2 pixel_cord, sampler2D texture) {
  ivec2 pixelCoord = ivec2(pixel_cord.x, pixel_cord.y);
  int mipLevel = 0;
  vec4 pixelValue = texelFetch(texture, pixelCoord, mipLevel);
  return pixelValue.rgb;
}

vec2 convert_splat_index_to_texture_index(uint splat_index) {
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
  vec4 p_hom = projmatrix * model * vec4(p_orig, 1) + model * vec4(0, 0, 0, 0);
  float p_w = 1. / (p_hom.w + 1e-7); // add 1e-7 so we don't divide by zero

  vec3 p_proj = p_hom.xyz * p_w;

  // projecting mean to screenspace
  vec4 p_view = camera * vec4(p_orig, 1);

  // check if the splat is behind the camera
  // key difference is negative vs positive
  if (p_view.z > 0.0) {
    gl_Position = vec4(0, 0, 0, 1);
    return;
  }

  // this 3d covraince is actualy a 3x3 matrix, but we only have 6 here since
  // the matrix is symmetric
  float cov3D[6] = float[6](s_cov3da.x, s_cov3da.y, s_cov3da.z, s_cov3db.x,
                            s_cov3db.y, s_cov3db.z);

  // convert the 3d covaraince to a 2d covariance
  vec3 cov =
      computeCov2D(p_orig, focal_x, focal_y, tan_fovx, tan_fovy, cov3D, camera);

  // if the determinant is 0, that means the spalt is just a single point
  float det = (cov.x * cov.z - cov.y * cov.y);
  if (det == 0.) {
    gl_Position = vec4(0, 0, 0, 1);
    return;
  }

  float det_inv = 1. / det;

  // 2d guassian is: f(x,y) = exp(-0.5 * [x y] * Σ^-1 * [x])
  // inverse of 2d covariance is: Σ^-1 = [a c; c b] / det
  // the conic is just the inverse of the covariance matrix, we do the
  // multiplication
  vec3 conic = vec3(cov.z, -cov.y, cov.x) * det_inv;

  float trace = (cov.x + cov.z); // this is the trace of the 2d matrix
  float half_trace = 0.5 * trace;

  // eigen value = (trace +- sqrt((trace^2 - 4*det)) /2
  // trace/2 +- (sqrt((trace^2 - 4*det))) /2
  // half_trace +- (sqrt((trace^2 - 4*det))) /2
  // half_trace +- (sqrt((4*half_trace^2 - 4*det))) /2
  // half_trace +- (2sqrt((half_trace^2 - det))) /2
  // half_trace +- sqrt((half_trace^2 - det))
  float lambda1 = half_trace + sqrt(max(0.1, half_trace * half_trace - det));
  float lambda2 = half_trace - sqrt(max(0.1, half_trace * half_trace - det));

  // pick the bigger eigen value to be the radius
  float my_radius = ceil(3. * sqrt(max(lambda1, lambda2)));

  // the center of the spalt in pixel coordiantes
  vec2 point_image = vec2(ndc2Pix(p_proj.x, W), ndc2Pix(p_proj.y, H));

  my_radius *= .15 + scale * .85;
  scale_modif = 1. / scale;

  // get a corner id either (-1, -1), (-1, 1), (1, -1), (1, 1)
  vec2 corner = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2) - 1.;

  // move the screen position of this verrtex to one of the corners of the splat
  vec2 screen_pos = point_image + my_radius * corner;

  col = get_value_from_texture(texture_coord, u_color_texture);
  con_o = vec4(conic, s_opacity);
  xy = point_image;
  pixf = screen_pos;
  depth = p_view.z;

  vec2 clip_pos = screen_pos / vec2(W, H) * 2. - 1.;
  gl_Position = vec4(clip_pos, 0, 1);
}
