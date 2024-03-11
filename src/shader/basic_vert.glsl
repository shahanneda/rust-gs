#version 300 es

in vec3 coordinates;

uniform mat4 model;
uniform mat4 camera;
uniform mat4 projection;

out mat4 x;

void main() {
  // gl_Position is a special variable a vertex shader
  // is responsible for setting
  gl_Position = projection*camera*model*vec4(coordinates, 1.0);
}