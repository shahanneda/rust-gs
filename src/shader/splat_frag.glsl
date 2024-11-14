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

out vec4 fragColor;

// https://github.com/graphdeco-inria/diff-gaussian-rasterization/blob/main/cuda_rasterizer/forward.cu#L263
void main() {
  vec2 d = xy - pixf;
  float power =
      -0.5 * (con_o.x * d.x * d.x + con_o.z * d.y * d.y) - con_o.y * d.x * d.y;
  if (power > 0.) {
    discard;
  }
  power *= scale_modif;
  float alpha = min(.99f, con_o.w * exp(power));
  vec3 color = col;

  if (alpha < 1. / 255.) {
    discard;
  }
  // fragColor = vec4(1,0,0,1);
  fragColor = vec4(color * alpha, alpha);
  //   fragColor = vec4(color, 1);
  // fragColor = vec4(color,  1);
  // fragColor = vec4(con_o.x,  0, 0, 1);
  // fragColor = vec4(con_o.x*128.0, 0, 0, 1);
  // fragColor = vec4(0, 0, 0, 1);
  // fragColor = vec4(color*alpha,  1);
  // fragColor = vec4(alpha, 0, 0,  1);
  // fragColor = vec4(color, 0.5);
}