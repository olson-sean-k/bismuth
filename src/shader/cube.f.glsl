#version 150 core

in vec4 v_color;
in vec2 v_uv;

out vec4 f_target0;

uniform sampler2D t_texture;

void main() {
    f_target0 = texture(t_texture, vec2(v_uv.x, -v_uv.y)) * v_color;
}
