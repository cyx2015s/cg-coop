#version 140

in vec3 position;
in vec3 normal;
in vec2 texCoord; 

out vec3 v_position;
out vec3 v_normal;
out vec2 v_tex_coord;
out vec4 v_frag_pos_light_space;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;
uniform mat4 light_space_matrix;

void main() {
    v_position = vec3(model * vec4(position, 1.0));
    v_normal = mat3(transpose(inverse(model))) * normal;
    v_tex_coord = texCoord; 
    
    v_frag_pos_light_space = light_space_matrix * vec4(v_position, 1.0);

    gl_Position = perspective * view * model * vec4(position, 1.0);
}