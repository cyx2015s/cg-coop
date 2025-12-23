use super::aabb::{ AABB, maximum_dim , union_aabb, union_aabb_inplace, union_aabb_point_inplace };
use super::primitive::{Primitive, PrimitiveInfo};
use super::partition::partition_index;
use super::vertex::Vertex;

pub struct Interaction <'a> {
    pub primitive: &'a Primitive,
    pub hitPoint: Vertex,
    pub material: Material,
}

pub struct BVHBuildNode {
    pub bound: AABB,
    pub left: Option<Box<BVHBuildNode>>,
    pub right: Option<Box<BVHBuildNode>>,
    splitAxis: usize,
    startIdx: usize,
    nPrimitives: usize,
}

enum NodeData {
    Interior { left_child: usize, right_child: usize },
    Leaf { start_index: usize, n_primitives: usize },
}

pub struct BVHNode {
    pub box: AABB,

    pub data: NodeData,

}

pub struct BVH {
    pub nodes: Vec<BVHNode>,
    pub orderedPrimitives: Vec<Primitive>,
    pub height: usize,
    pub maxHeight: usize,
}

impl BVH { 
    pub fn new(primitives: Vec<Primitive>) -> Self { 
    }

    fn constructBVH(primitives: Vec<Primitive>) -> Self{
        let mut primInfo: Vec<PrimitiveInfo> = Vec::new();

        for (index, primitive) in primitives.iter().enumerate() {
            let box = match primitive {
                Primitive::Sphere(sphere) => sphere.getAABB(),
                Primitive::Triangle(triangle) => triangle.getAABB(),
            };

            primInfo.push(PrimitiveInfo {
                pid: index,
                box,
            });
        }

        let mut totalNode = 0;
        let mut root = BVH::recursiveBuild(&primitive,&mut primInfo, 0, primitives.len(),&mut totalNode);
    }

    fn recursiveBuild(
        &mut self,
        primitives: &Vec<Primitive>, 
        primInfo: &mut Vec<PrimitiveInfo>,
        start: usize,
        end: usize,
        totalNode: &mut usize,
    )-> (BVHBuildNode, usize) {
        let mut node = BVHBuildNode::new();
        *totalNode += 1;

        let nPrimitives = end - start;
        if nPrimitives == 1 {
            let mut box: AABB = AABB::default();
            for i in start..end {
                union_aabb_inplace(box, &primInfo[i].box);
                self.orderedPrimitives.push(primitives[primInfo[i].pid].clone());
            }
            node.init_leaf(box, start, nPrimitives);
        } 
        else {
            let mut centroidBounds: AABB = AABB::default();
            for i in start..end {
                union_aabb_point_inplace(centroidBounds, &primInfo[i].centroid);
            }

            let axis = maximum_dim(&centroidBounds);

            if centroidBounds.max[axis] == centroidBounds.min[axis] {
                let mut box: AABB = AABB::default();
                let startId = self.orderedPrimitives.len();
                for i in start..end {
                    union_aabb_inplace(box, &primInfo[i].box);
                    self.orderedPrimitives.push(primitives[primInfo[i].pid].clone());
                }
                node.init_leaf(box, startId, nPrimitives);
                return (node, totalNode);
            }

            let pmid: f32 = 0.5 * (centroidBounds.min[axis] + centroidBounds.max[axis]);

            let mid: usize = partition_index(&mut primInfo[start..end], |pi| pi.centroid[axis] < pmid);
            
            if (mid == start || mid == end){
                mid = start + end / 2;
            }

            let (left, right) = (&mut primInfo[start..end]).split_at_mut(mid);


            let (leftChild, totalNode) = self.recursiveBuild(primitives, left, start, mid, &mut totalNode);
            let (rightChild, totalNode) = self.recursiveBuild(primitives, right, mid, right.len(), &mut totalNode);
        
            node.init_interior(Box::new(leftChild), Box::new(rightChild), axis);
        }
        return (node, totalNode);
    }

    pub fn intersect(&self, ray: &mut Ray, isect: &mut Interaction) -> bool {
        let mut hit = false;
        let invDir = Vec3::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        let sign = Vec3::new(if ray.d.x < 0.0 { 1 } else { 0 }, if ray.d.y < 0.0 { 1 } else { 0 }, if ray.d.z < 0.0 { 1 } else { 0 });
        let mut currentNodeIndex:usize = 0;
        let mut toVisitOffset:usize = 0;
        let mut nodesToVisit: [usize; 128] = [0; 128];
        loop {
            let node = &self.nodes[currentNodeIndex];
            if (node.box.intersect_full(&ray, &invDir, &sign)){
                match node.data {
                    NodeData::Leaf { start_index, n_primitives }
                    => {
                        for i in start_index..start_index+n_primitives {
                            let primitive = &self.primitives[i];
                            if (primitive.intersect(&ray)) {
                                hit = true;
                            }
                        }

                        if toVisitOffset == 0 {
                            break;
                        }
                    }
                }
                    NodeData::Interior { left_child, right_child }
                    => {
                        nodesToVisit[toVisitOffset++] = right_child;
                        currentNodeIndex = left_child;
                    }
            }
            else {
                if (toVisitOffset == 0) {
                    break;
                }
                currentNodeIndex = nodesToVisit[--toVisitOffset];
            }
        }
    }
    return hit;
}

impl BVHNode {
    pub fn new_interior(box: AABB, left_child: usize, right_child: usize) -> Self {
        Self {
            box,
            data: NodeData::Interior { left_child, right_child },
        }
    }

    pub fn new_leaf(box: AABB, start_index: usize, n_primitives: usize) -> Self {
        Self {
            box,
            data: NodeData::Leaf { start_index, n_primitives },
        }
    }

}

impl BVHBuildNode {
    pub fn new() -> Self {
        Self {
            aabb: AABB::new(Vec3::ZERO, Vec3::ZERO),
            left: None,
            right: None,
            splitAxis: 0,
            startIdx: 0,
            nPrimitives: 0,
        }
    }

    pub fn init_leaf(&mut self, box: AABB, sid: usize, n: usize) {
        self.bound = box;
        self.startIdx = sid;
        self.nPrimitives = n;
        self.left = None;
        self.right = None;
    }

    pub fn init_interior(&mut self, left: Box<BVHBuildNode>, right: Box<BVHBuildNode>, axis: usize) {
        self.left = Some(left);
        self.right = Some(right);
        self.bound = union_aabb(left.as_ref().unwrap().bound, right.as_ref().unwrap().bound);
        self.splitAxis = axis;
        self.nPrimitives = 0;
    }
}

