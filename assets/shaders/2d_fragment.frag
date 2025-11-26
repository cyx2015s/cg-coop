#version 140
in vec2 f_position;
out vec4 color;

void main() {
    color = vec4(f_position * 0.5 + 0.5, f_position.x * f_position.y + 0.5, 1.0);
}