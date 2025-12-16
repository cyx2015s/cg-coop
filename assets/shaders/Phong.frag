#version 140

in vec3 v_position;
in vec3 v_normal;
in vec2 v_tex_coord; 
in vec4 v_frag_pos_light_space;

out vec4 color;

uniform vec3 viewPos;
uniform sampler2D diffuse_tex;
uniform bool has_texture; 

uniform sampler2D shadow_map;

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

// 阴影计算函数
float ShadowCalculation(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir) {
    // 1. 执行透视除法
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    // 2. 变换到 [0,1] 的范围
    projCoords = projCoords * 0.5 + 0.5;

    // 3. 如果超过视锥体远平面，不仅行阴影计算
    if(projCoords.z > 1.0)
        return 0.0;

    // 4. 计算 Bias (偏移量) 防止阴影痤疮
    // 根据表面法线和光线的夹角动态调整 bias
    float bias = max(0.005 * (1.0 - dot(normal, lightDir)), 0.0005);

    // 5. PCF 柔化边缘
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadow_map, 0);
    // 采样周围 3x3 区域
    for(int x = -1; x <= 1; ++x) {
        for(int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadow_map, projCoords.xy + vec2(x, y) * texelSize).r; 
            // 比较深度：如果贴图里的深度 < 当前深度 - bias，说明在阴影里
            shadow += (projCoords.z - bias > pcfDepth ? 1.0 : 0.0);        
        }    
    }
    shadow /= 9.0;
    
    return shadow;
}

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

// 平行光计算，增加 shadow 参数
vec3 calcDirectionalLight(Light l, vec3 normal, float shadow) {
    vec3 lightDir = normalize(-l.direction);
    vec3 diffuse = l.color * max(dot(lightDir, normal), 0.0f) * getDiffuseColor();
    vec3 reflectDir = reflect(-lightDir, normal); 
    
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = l.color * pow(max(dot(reflectDir, viewDir), 0.0f), material.ns) * material.ks; 

    return l.intensity * ((1.0 - shadow) * (diffuse + spec));
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
    float shadow = 0.0;
    
    for (int i = 0; i < num_lights; i++) {
        if (lights[i].light_type == 1) { // Directional
             vec3 lightDir = normalize(-lights[i].direction);
             shadow = ShadowCalculation(v_frag_pos_light_space, normal, lightDir);
             break; 
        }
    }

    for (int i = 0; i < num_lights; i++) {
        switch (lights[i].light_type) {
            case 0: light_color += calcAmbientLight(lights[i]); break;
            case 1: light_color += calcDirectionalLight(lights[i], normal, shadow); break; // 传入 shadow
            case 2: light_color += calPointLight(lights[i], normal); break;
            case 3: light_color += calSpotLight(lights[i], normal); break;
        }
    }
    color = vec4(light_color, 1.0f);
}