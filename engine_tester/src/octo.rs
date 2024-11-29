use crate::chunk_generator::VoxelChunk;

#[derive(Default)]
pub struct OctreeNode {
    pub block_type: Option<u8>,
    pub children: Option<[Box<OctreeNode>; 8]>,
    pub needs_generation: bool, // New field to track lazy generation
}

pub fn build_octree(chunk: &VoxelChunk, x: usize, y: usize, z: usize, size: usize) -> OctreeNode {
    // Base case: Single voxel
    if size == 1 {
        let block_type = chunk.get_block(x, y, z);
        return OctreeNode {
            block_type: if block_type == 0 { None } else { Some(block_type) },
            children: None,
            needs_generation: true,
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
            needs_generation: false,
        };
    }

    OctreeNode {
        block_type: None,
        children: Some(children.map(|child| child.unwrap())),
        needs_generation: true,
    }
}

impl OctreeNode {
    pub fn block_type(&self) -> Option<u8> {
        self.block_type
    }

    pub fn children(&self) -> Option<&[Box<OctreeNode>; 8]> {
        self.children.as_ref()
    }
    
    pub fn lazy_generate(&mut self, chunk: &VoxelChunk, x: usize, y: usize, z: usize, size: usize) {
        // Only generate if marked as needing generation
        if !self.needs_generation {
            return;
        }

        // Similar to existing build_octree logic, but with wave function collapse
        if size == 1 {
            let block_type = chunk.get_block(x, y, z);
            self.block_type = if block_type == 0 { None } else { Some(block_type) };
            self.needs_generation = false;
            return;
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

            let mut child = OctreeNode {
                block_type: None,
                children: None,
                needs_generation: true, // Mark child for lazy generation
            };

            // Only generate if near view or needed
            child.lazy_generate(&chunk.clone(), x + dx, y + dy, z + dz, half);

            if let Some(block_type) = child.block_type {
                block_types.push(block_type);
            } else {
                is_homogeneous = false;
            }
            children[i] = Some(Box::new(child));
        }

        // Optimization for homogeneous regions
        if is_homogeneous && !block_types.is_empty() && block_types.iter().all(|&t| t == block_types[0]) {
            self.block_type = Some(block_types[0]);
            self.children = None;
        } else {
            self.block_type = None;
            self.children = Some(children.map(|child| child.unwrap()));
        }

        self.needs_generation = false;
    }

    pub fn needs_detail(camera_distance: f32, node_size: f32) -> bool {
        // Example threshold logic
        //camera_distance < node_size * 2.0
        true //rn true but later it will be better
    }
}
