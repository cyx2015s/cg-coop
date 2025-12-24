#version 140

uniform usampler2DArray rng_state;
uniform int rng_index;

// Scene Config
uniform sampler2D bvh;

in vec2 ftex_coords;
out vec4 color;

const float INFINITY = 10e10f;
const float FloatOneMinusEpsilon = 0.99999994f;


const int maxTraceDepth = 16;

uint rngState;
void rngInit();

float rngGetRandom1D();

layout(std140) uniform CameraBlock {
    mat4 camera_to_world;
    mat4 raster_to_camera;
};

struct Vertex {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
};

struct Material {
    int type;
    float fuzz;
    float ior;
    vec3 albedo;
};

struct Ray {
    vec3 o;
    vec3 dir;
    float tMax;
};

struct Primitive {
    int shape_type;
    int shape_idx;
    int material_idx;
};

struct Interaction {
    Primitive primitive;
    Vertex hit_point;
    Material material;
};

struct AABB {
    vec3 p_min;
    vec3 p_max;
};

struct BVHNode {
    AABB box;
    int node_type;
    int first_val;
    int second_val;
};

Ray generateRay(vec2 u);
vec4 trace(inout Ray ray);

bool intersect(inout Ray ray, inout Interaction isect);

vec2 getSampleIdx(sampler2D data, int idx);
void getVec3FromTexture(sampler2D data, vec2 tex_coord, out vec3 v);

void getBVHNodeData(sampler2D data, int idx, out BVHNode node);

void main() { 
    rngInit();
    Ray ray = generateRay(vec2(rngGetRandom1D(), rngGetRandom1D()));

}

void rngInit() {
    rngState = texture(rng_state, vec3(ftex_coords, rng_index)).r;
}

float rngGetRandom1D() {
    rngState ^= (rngState << 13);
    rngState ^= (rngState >> 17);
    rngState ^= (rngState << 5);
    return min(FloatOneMinusEpsilon, float(rngState) * (1.0 / 4294967296.0));
}

Ray generateRay(vec2 u) {
    vec4 pixelPos = vec4(gl_FragCoord.x - 0.5 + u.x, gl_FragCoord.y - 0.5 + u.y, 0.0, 1.0);
    Ray ray;
    ray.o = vec3(camera_to_world * vec4(0.0, 0.0, 0.0, 1.0));
    vec3 localRayDir = (raster_to_camera * pixelPos).xyz - vec3(0.0, 0.0, 0.0);
    ray.dir = normalize(vec3(camera_to_world * vec4(localRayDir.xyz, 0.0)));
    ray.tMax = INFINITY;
    return ray;
}

vec4 trace(inout Ray ray) {
    vec3 radiance = vec3(0.0);
    vec3 throughput = vec3(1.0);

    for (int depth = 0; depth < maxTraceDepth; depth++) {
        Interaction isect;
        if (!intersect(ray, isect)){

        }
    }
}

bool intersect(inout Ray ray, inout Interaction isect) {
    int stack[256];
    int stackPtr = 0;
    stack[stackPtr++] = 0;

    bool foundHit = false;
    vec3 invDir = 1.0 / ray.dir;

    while (stackPtr > 0) {
        int nodeIdx = stack[--stackPtr];
        BVHNode node;
        getBVHNodeData(bvh, nodeIdx, node);
    }
}

vec2 getSampleIdx(sampler2D data, int idx) {
    ivec2 texSize = textureSize(data, 0);
    int x = idx % texSize.x;
    int y = idx / texSize.x;
    return vec2((float(x) + 0.5) / float(texSize.x), (float(y) + 0.5) / float(texSize.y));
}

void getVec3FromTexture(sampler2D data, vec2 tex_coord, out vec3 v) {
    v = texture(data, tex_coord).rgb;
}

void getBVHNodeData(sampler2D data, int idx, out BVHNode node) { 
    vec3 v[3];
    int vid = idx * 3;
    for (int i = 0; i < 3; ++i) {
        getVec3FromTexture(data, getSampleIdx(data, vid + i), v[i])
    }
}