#version 300 es
precision mediump float;
#pragma optimize(off)
#pragma debug(on)

in vec3 col;
in float scale_modif;
in float depth;
in vec4 con_o;
in vec2 xy;
in vec2 pixf;

uniform bool do_blending;

out vec4 fragColor;

// https://github.com/graphdeco-inria/diff-gaussian-rasterization/blob/main/cuda_rasterizer/forward.cu#L263
void main() {

  // distance of this fragment compared to the centerd of the splat
  vec2 d = xy - pixf;

  // this just represents the exponent of a 2d gaussian function f(x,y) =
  // exp(-0.5 * [x y] * Σ^-1 * [x]) inside con_o.xyz we stored the inverse sigma
  // matrix
  float power =
      -0.5 * (con_o.x * d.x * d.x + con_o.z * d.y * d.y) - con_o.y * d.x * d.y;

  if (power > 0.) {
    discard;
  }

  power *= scale_modif;
  // the con_o.w is the oirigal opacity of the splat
  float alpha = min(.99f, con_o.w * exp(power));
  vec3 color = col;

  if (alpha < 1. / 25.) {
    gl_FragDepth = -1000000.0;
    discard;
  }

  gl_FragDepth = -depth / 3.0;
  // fragColor = vec4(1,0,0,1);
  if (do_blending) {
    fragColor = vec4(color * alpha, alpha);
  } else {
    fragColor = vec4(color, 1);
  }
  // fragColor = vec4(vec3(-depth), 1.0);
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