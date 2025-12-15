#version 140

in vec3 v_position;
in vec3 v_normal;
in vec2 v_tex_coord; 

out vec4 color;

uniform vec3 viewPos;
uniform sampler2D diffuse_tex;
uniform bool has_texture; 

struct Light {
    vec3 color;
    float intensity;
    vec3 position;
    float angle;
    vec3 direction;
    float range;
    vec3 kfactor;
    int light_type;
};

struct Material {
    vec3 ka;
    float _pad1;
    vec3 kd;
    float _pad2;
    vec3 ks;
    float ns;
};

layout(std140) uniform Light_Block {
    Light lights[32];
    int num_lights;
};
layout(std140) uniform Material_Block {
    Material material;
};

vec3 getDiffuseColor() {
    if (has_texture) {
        return texture(diffuse_tex, v_tex_coord).rgb * material.kd; 
    } else {
        return material.kd;
    }
}

vec3 calcAmbientLight(Light l) { 
    vec3 ambientColor = material.ka;
    
    if (has_texture) {
        ambientColor *= texture(diffuse_tex, v_tex_coord).rgb;
    }
    
    return l.color * l.intensity * ambientColor;
}

vec3 calcDirectionalLight(Light l, vec3 normal) {
    vec3 lightDir = normalize(-l.direction);
    vec3 diffuse = l.color * max(dot(lightDir, normal), 0.0f) * getDiffuseColor();
    vec3 reflectDir = reflect(lightDir, normal);
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = l.color * pow(max(dot(reflectDir, viewDir), 0.0f), material.ns) * material.ks;
    return l.intensity * (diffuse + spec);
}

vec3 calPointLight(Light l, vec3 normal) {
    vec3 lightDir = normalize(l.position - v_position);
    vec3 diffuse = l.color * max(dot(lightDir, normal), 0.0f) * getDiffuseColor();
    float distance = length(v_position - l.position);
    float attenuation = 1.0f / (l.kfactor[0] + l.kfactor[1] * distance + l.kfactor[2] * distance * distance);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = l.color * pow(max(dot(reflectDir, viewDir), 0.0f), material.ns) * material.ks;
    return l.intensity * (diffuse + spec) * attenuation;
}

vec3 calSpotLight(Light l, vec3 normal) { 
    vec3 lightDir = normalize(l.position - v_position);
    float theta = acos(-dot(lightDir, normalize(l.direction)));
    if (theta > l.angle) {
        return vec3(0.0f);
    }
    vec3 diffuse = l.color * max(dot(lightDir, normal), 0.0f) * getDiffuseColor();
    float distance = length(v_position - l.position);
    float attenuation = 1.0f / (l.kfactor[0] + l.kfactor[1] * distance + l.kfactor[2] * distance * distance);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = l.color * pow(max(dot(reflectDir, viewDir), 0.0f), material.ns) * material.ks;
    return l.intensity * (diffuse + spec) * attenuation;
}

void main() {
    vec3 normal = normalize(v_normal);
    vec3 light_color = vec3(0.0f);
    for (int i = 0; i < num_lights; i++) {
        switch (lights[i].light_type) {
            case 0: light_color += calcAmbientLight(lights[i]); break;
            case 1: light_color += calcDirectionalLight(lights[i], normal); break;
            case 2: light_color += calPointLight(lights[i], normal); break;
            case 3: light_color += calSpotLight(lights[i], normal); break;
        }
    }
    color = vec4(light_color, 1.0f);
}