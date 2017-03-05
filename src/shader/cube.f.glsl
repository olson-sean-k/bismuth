#version 150 core

in vec4 v_color;
in vec2 v_uv;

out vec4 f_target0;

void main() {
    f_target0 = v_color;
}
