#version 140

in vec3 position;
out vec3 v_dir;

uniform mat4 projection;
uniform mat4 view;

void main() {
    v_dir = position;

    vec4 pos = projection * view * vec4(position, 1.0);
    gl_Position = pos.xyww; // 固定在远平面
}
