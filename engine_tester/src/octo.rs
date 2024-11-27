use crate::chunk_generator::VoxelChunk;

pub struct OctreeNode {
    block_type: Option<u8>,          // `None` for mixed/hollow nodes, `Some(type)` for homogeneous nodes.
    children: Option<[Box<OctreeNode>; 8]>, // Array of 8 child nodes.
}

pub fn build_octree(chunk: &VoxelChunk, x: usize, y: usize, z: usize, size: usize) -> OctreeNode {
    // Base case: Single voxel
    if size == 1 {
        let block_type = chunk.get_block(x, y, z);
        return OctreeNode {
            block_type: if block_type == 0 { None } else { Some(block_type) },
            children: None,
        };
    }

    let half = size / 2;
    let mut children: [Option<Box<OctreeNode>>; 8] = Default::default();
    let mut block_types = Vec::new();
    let mut is_homogeneous = true;

    for i in 0..8 {
        let (dx, dy, dz) = (
            (i & 1) * half,
            ((i >> 1) & 1) * half,
            ((i >> 2) & 1) * half,
        );
        let child = build_octree(&chunk.clone(), x + dx, y + dy, z + dz, half);
        if let Some(block_type) = child.block_type {
            block_types.push(block_type);
        } else {
            is_homogeneous = false;
        }
        children[i] = Some(Box::new(child));
    }

    if is_homogeneous && !block_types.is_empty() && block_types.iter().all(|&t| t == block_types[0]) {
        return OctreeNode {
            block_type: Some(block_types[0]),
            children: None,
        };
    }

    OctreeNode {
        block_type: None,
        children: Some(children.map(|child| child.unwrap())),
    }
}

impl OctreeNode {
    pub fn block_type(&self) -> Option<u8> {
        self.block_type
    }

    pub fn children(&self) -> Option<&[Box<OctreeNode>; 8]> {
        self.children.as_ref()
    }
    
}
