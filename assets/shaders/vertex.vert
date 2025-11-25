#version 140

in vec2 position;
out vec2 f_position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    f_position = position;
}