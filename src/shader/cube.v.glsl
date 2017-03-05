#version 150 core

in vec3 a_position;
in vec2 a_uv;
in vec4 a_color;

out vec4 v_color;
out vec2 v_uv;

uniform transform {
    mat4 u_camera;
    mat4 u_model;
};

void main() {
    v_color = a_color;
    v_uv = a_uv;
    gl_Position = u_camera * u_model * vec4(a_position, 1.0);
}
