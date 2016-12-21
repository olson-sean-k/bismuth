#version 150 core

in vec3 a_position;
in vec4 a_color;
out vec4 v_color;

uniform mat4 u_transform;

void main() {
    v_color = a_color;
    gl_Position = u_transform * vec4(a_position, 1.0);
}
