use super::primitive::*;
use super::material::*;
use super::transform::*;
use cg_coop::shape::Mesh;
pub fn createPrimitiveBuffer(
    spheres: &Vec<Sphere>,
    models: &Vec<Mesh>,
    transforms: &Vec<Transform>,
    sphereMaterials: &Vec<Raytracing_Material>,
    modelMaterials: &Vec<Raytracing_Material>
    bufferWidth: usize,
){
    let mut totalPrimitives: usize = 0;
    let mut totalTriangles: usize = 0;
    let mut totalVertices: usize = 0;

    // 计算图元个数，为BVH做准备
    totalPrimitives += spheres.len();
    for model in models {
        totalTriangles += model.get_indices().len() / 3;
        totalVertices += model.get_vertices().len();
    }
    totalPrimitives += totalTriangles;

    // 初始化图元缓冲、顶点缓冲、三角形缓冲和材质缓冲
    let primitiveBufferSize = roudUp(totalPrimitives, bufferWidth);
    let materialBufferSize = roudUp(sphereMaterials.len() + modelMaterials.len(), bufferWidth);
    let vertexBufferSize = roudUp(totalVertices, bufferWidth);
    let triangleBufferSize = roudUp(totalTriangles, bufferWidth);
    let mut vertices = vec![Vertex::default(); vertexBufferSize];
    let mut triangles = vec![Triangle::default(); triangleBufferSize];
    let mut primitives: Vec<Primitive> = Vec::new();
    let mut materials = vec![Material::default(); materialBufferSize];

    let mut primitiveCnt: usize = 0;
    let mut materialCnt: usize = 0;
    let mut vertexCnt: usize = 0;
    let mut triangleCnt: usize = 0;

    if !spheres.is_empty() { 
        for i, sphere in spheres.iter().enumerate() { 
            primitives.push(
                //todo
            )
        } 

        for i, material in sphereMaterials.iter().enumerate() { 
            materials[materialCnt] = material;
            materialCnt += 1;
        }

        let mut sphereBuffer = vec![Sphere::default(); roundUp(spheres.len(), bufferWidth)];
        for i, sphere in spheres.iter().enumerate() { 
            sphereBuffer[i] = sphere;
        }

        _sphereBuffer = 
    }
}

pub roudUp(val: usize, number: usize) -> usize { 
    (val + number - 1) / number * number
}