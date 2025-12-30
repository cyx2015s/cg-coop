#version 140

in vec3 position;

uniform mat4 model;
uniform mat4 lightVP;

out vec3 vWorldPos;

void main() {
    vec4 worldPos = model * vec4(position, 1.0);
    vWorldPos = worldPos.xyz;
    gl_Position = lightVP * worldPos;
}