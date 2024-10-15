#version 300 es
#pragma optimize(off)
#pragma debug(on)

in vec3 v_pos;
in vec3 v_col;
out vec3 v_color;

uniform mat4 camera;
uniform mat4 projection;


void main() {
	gl_Position = projection * camera * vec4(v_pos,1);
	v_color = v_col;
}