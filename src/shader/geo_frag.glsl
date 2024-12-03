#version 300 es
precision mediump float;
// #pragma optimize(off)
// #pragma debug(on)

out vec4 fragColor;
in float depth;
in vec3 v_pos_out;
in vec3 v_color;
in vec3 v_normal;

uniform vec3 light_pos;
uniform bool is_picking;
uniform vec3 picking_color;
uniform bool shadows;

void main() {
  //     vec2 position = gl_FragCoord.xy;
  //     vec3 color = vec3(
  //         mod(position.x, 256.0) / 255.0,
  //         mod(position.y, 256.0) / 255.0,
  //         mod((position.x + position.y), 256.0) / 255.0
  //     );
  // if (depth >= 0.0) {
  //   gl_FragDepth = -depth / 10.0;
  //   discard;
  // }
  // fragColor = vec4(vec3(-(depth / 10.0)), 1.0);
  float currentDepth = gl_FragCoord.z;

  vec3 normal = normalize(v_normal);
  vec3 lightDir = normalize(light_pos - v_pos_out);
  float diff = max(dot(normal, lightDir), 0.4);
  gl_FragDepth = -depth / 10.0;

  if (shadows) {
    fragColor = vec4(diff * v_color, 1.0);
  } else {
    fragColor = vec4(v_color, 1.0);
  }

  if (is_picking) {
    fragColor = vec4(picking_color, 1.0);
  }
  // fragColor = vec4(vec3(-depth / 10.0), 1.0);
  // fragColor = vec4(v_color.x, v_color.y, v_color.z, 1.0);

  // fragColor = vec4(vec3(currentDepth), 1.0);
  //   fragColor = vec4(1.0, 0.0, 0.0, 1.0);
  //   vec3 frag = vec4(1.0, 0.0, 0.0);
}