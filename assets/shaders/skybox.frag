#version 140

in vec3 v_dir;
out vec4 color;

uniform sampler2DArray skybox;

vec2 fix_cube_uv(vec2 uv, float texSize) {
    float padding = 0.5 / texSize;
    return uv * (1.0 - 2.0 * padding) + padding;
}

vec2 cube_uv(vec3 d, out int layer) {
    vec3 a = abs(d);

    if (a.x >= a.y && a.x >= a.z) {
        // X major
        if (d.x > 0.0) {
            layer = 0; // +X
            return vec2(-d.z, d.y) / a.x * 0.5 + 0.5;
        } else {
            layer = 1; // -X
            return vec2(d.z, d.y) / a.x * 0.5 + 0.5;
        }
    } else if (a.y >= a.x && a.y >= a.z) {
        // Y major
        if (d.y > 0.0) {
            layer = 2; // +Y
            return vec2(d.x, -d.z) / a.y * 0.5 + 0.5;
        } else {
            layer = 3; // -Y
            return vec2(d.x, d.z) / a.y * 0.5 + 0.5;
        }
    } else {
        // Z major
        if (d.z > 0.0) {
            layer = 4; // +Z
            return vec2(d.x, d.y) / a.z * 0.5 + 0.5;
        } else {
            layer = 5; // -Z
            return vec2(-d.x, d.y) / a.z * 0.5 + 0.5;
        }
    }
}

void main() {
    vec3 dir = normalize(v_dir);
    int layer;
    vec2 uv = cube_uv(dir, layer);
    ivec3 size = textureSize(skybox, 0);
    vec2 uv_fixed = fix_cube_uv(uv, float(size.x));

    color = texture(skybox, vec3(uv_fixed, float(layer)));
}
