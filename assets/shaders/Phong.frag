#version 140

in vec3 v_position;
in vec3 v_normal;

out vec4 color;

uniform vec3 viewPos;
uniform mat4 ambient_light;
uniform mat4 directional_light;
uniform mat4 point_light;
uniform mat4 spot_light;
uniform mat4 material;

vec3 calcAmbientLight() {
    return ambient_light[2].xyz * ambient_light[0][3] * material[0].xyz;
}

vec3 calcDirectionalLight(vec3 normal) {
    vec3 lightDir = normalize(-directional_light[1].xyz);
    vec3 diffuse = directional_light[2].xyz * max(dot(lightDir, normal), 0.0f) * material[1].xyz;
    vec3 reflectDir = reflect(lightDir, normal);
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = directional_light[2].xyz * pow(max(dot(reflectDir, viewDir), 0.0f), material[3][3]) * material[2].xyz;
    return directional_light[0][3] * (diffuse + spec);
}

vec3 calPointLight(vec3 normal) {
    vec3 lightDir = normalize(point_light[0].xyz - v_position);
    vec3 diffuse = point_light[2].xyz * max(dot(lightDir, normal), 0.0f) * material[1].xyz;
    float distance = length(v_position - point_light[0].xyz);
    float attenuation = 1.0f / (point_light[3][0] + point_light[3][1] * distance + point_light[3][2] * distance * distance);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = point_light[2].xyz * pow(max(dot(reflectDir, viewDir), 0.0f), material[3][3]) * material[2].xyz;
    return point_light[0][3] * (diffuse + spec) * attenuation;
}

vec3 calSpotLight(vec3 normal) { 
    vec3 lightDir = normalize(spot_light[0].xyz - v_position);
    float theta = acos(-dot(lightDir, normalize(spot_light[1].xyz)));
    if (theta > spot_light[1][3]) {
        return vec3(0.0f);
    }
    vec3 diffuse = spot_light[2].xyz * max(dot(lightDir, normal), 0.0f) * material[1].xyz;
    float distance = length(v_position - spot_light[0].xyz);
    float attenuation = 1.0f / (spot_light[3][0] + spot_light[3][1] * distance + spot_light[3][2] * distance * distance);
    vec3 reflectDir = reflect(-lightDir, normal);
    vec3 viewDir = normalize(viewPos - v_position);
    vec3 spec = spot_light[2].xyz * pow(max(dot(reflectDir, viewDir), 0.0f), material[3][3]) * material[2].xyz;
    return spot_light[0][3] * (diffuse + spec) * attenuation;
}

void main() {
    vec3 normal = normalize(v_normal);
    vec3 light_color = vec3(0.0f);
    light_color += calcAmbientLight();
    light_color += calcDirectionalLight(normal);
    light_color += calPointLight(normal);
    light_color += calSpotLight(normal);
    color = vec4(light_color, 1.0f);
  
}