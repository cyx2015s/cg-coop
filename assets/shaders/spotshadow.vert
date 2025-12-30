#version 140

in vec3 position;

out vec4 fPosition;

uniform mat4 model;
uniform mat4 lightSpaceMatrix;

void main() {
    fPosition = model * vec4(position, 1.0);
    gl_Position = lightSpaceMatrix * fPosition;
}