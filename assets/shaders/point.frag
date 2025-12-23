#version 140

out vec4 color;

void main() {
    vec2 coord = gl_PointCoord * 2.0 - 1.0;
    float distance = length(coord);
    
    // 圆形：丢弃边缘像素
    if (distance > 1.0) {
        discard;
    }
    
    // 可选：柔化边缘（抗锯齿）
    float alpha = 1.0 - smoothstep(0.8, 1.0, distance);
    color = vec4(1.0, 0.0, 0.0, 1.0);
}