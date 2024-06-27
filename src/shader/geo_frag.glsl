#version 300 es
precision mediump float;
#pragma optimize(off)
#pragma debug(on)

out vec4 fragColor;
in vec3 v_color;

void main() {
//     vec2 position = gl_FragCoord.xy;
//     vec3 color = vec3(
//         mod(position.x, 256.0) / 255.0, 
//         mod(position.y, 256.0) / 255.0, 
//         mod((position.x + position.y), 256.0) / 255.0
//     );
    fragColor = vec4(v_color, 1);
}