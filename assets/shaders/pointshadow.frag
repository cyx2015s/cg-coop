#version 140

in vec3 vWorldPos;

uniform vec3 lightPos;
uniform float range;

void main(){
    float dist = length(vWorldPos - lightPos);
    gl_FragDepth = dist / range;
}