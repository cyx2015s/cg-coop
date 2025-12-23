#version 140

out vec4 color;
in vec2 ftex_coords;

uniform sampler2DArray shadow_map;
uniform int layer;

void main()
{
    float depth = texture(shadow_map, vec3(ftex_coords, layer)).r;
    color = vec4(vec3(depth), 1.0);
}