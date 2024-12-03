#version 300 es
precision mediump float;
#pragma optimize(off)
#pragma debug(on)

out vec4 fragColor;
in float depth;
in vec3 v_color;

void main() {
  fragColor = vec4(v_color.x, v_color.y, v_color.z, 1.0);
}
