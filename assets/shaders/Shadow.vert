#version 140

in vec3 position;

uniform mat4 model;
uniform mat4 light_space_matrix; // 灯光的 投影矩阵 * 视图矩阵

void main() {
    gl_Position = light_space_matrix * model * vec4(position, 1.0);
}