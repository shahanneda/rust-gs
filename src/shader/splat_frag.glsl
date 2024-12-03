#version 300 es
// Took inspiration from
// https://github.com/graphdeco-inria/diff-gaussian-rasterization/blob/main/cuda_rasterizer/forward.cu
precision mediump float;
// #pragma optimize(off)
// #pragma debug(on)

in vec3 color;
in float scale_out;
in float depth;
in vec4 conic_opacity;
in vec2 center_of_splat;
in vec2 current_vert_screen_pos;

uniform bool do_blending;

out vec4 fragColor;

void main() {
  // distance of this fragment compared to the center of the splat
  vec2 d = center_of_splat - current_vert_screen_pos;

  // this just represents the exponent of a 2d gaussian function f(x,y) =
  // exp(-0.5 * [x y] * Σ^-1 * [x]) inside con_o.xyz we stored the inverse sigma
  // matrix
  float power =
      -0.5 * (conic_opacity.x * d.x * d.x + conic_opacity.z * d.y * d.y) -
      conic_opacity.y * d.x * d.y;
  if (power > 0.) {
    discard;
  }
  power *= scale_out;

  // the conic_opacity.w is the original opacity of the splat
  float alpha = min(.99f, conic_opacity.w * exp(power));
  if (alpha < 1. / 25.) {
    gl_FragDepth = -1000000.0;
    discard;
  }

  gl_FragDepth = -depth / 10.0;
  if (do_blending) {
    fragColor = vec4(color * alpha, alpha);
    // fragColor = vec4(vec3(-depth / 10.0), 1.0);
  } else {
    fragColor = vec4(color, 1);
  }
  // fragColor = vec4(vec3(-depth / 10.0), 1.0);
  // fragColor = vec4(vec3(-(depth / 5.0)), 1.0);
  // fragColor = vec4(color, 1);
  // fragColor = vec4(color,  1);
  // fragColor = vec4(con_o.x,  0, 0, 1);
  // fragColor = vec4(con_o.x*128.0, 0, 0, 1);
  // fragColor = vec4(0, 0, 0, 1);
  // fragColor = vec4(color*alpha,  1);
  // fragColor = vec4(alpha, 0, 0,  1);
  // fragColor = vec4(color, 0.5);
}