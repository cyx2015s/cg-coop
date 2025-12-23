#version 140

in vec3 position;
out vec4 u_color;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;
uniform int selected_idx;

void main() {
    if(gl_VertexID == selected_idx){
        u_color = vec4(1.0, 1.0, 0.0, 1.0);
    } else {
        u_color = vec4(0.0, 0.0, 1.0, 1.0);
    }
    gl_Position = projection * view * model * vec4(position, 1.0);
}
