use std::collections::HashMap;
use rand::Rng;

#[derive(Default, Clone)]
pub struct OctreeNode {
    pub block_type: Option<u8>,
    pub children: Option<[Box<OctreeNode>; 8]>,
    pub needs_generation: bool, // New field to track lazy generation
}


impl OctreeNode {
    pub fn new(block_type: Option<u8>, needs_generation: bool) -> Self {
        Self {
            block_type,
            children: None,
            needs_generation,
        }
    }

    pub fn block_type(&self) -> Option<u8> {
        self.block_type
    }

    pub fn children(&self) -> Option<&[Box<OctreeNode>; 8]> {
        self.children.as_ref()
    }


    //this is called from hiearchical terrain generation on the terrain generation, it is actually in fact used and is the one that we use
    //to generate the single chunk at spawn... also it is used in generate multiple chunks i realized lololololololol
    pub fn lazy_generate(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        size: usize,
        rng: &mut rand::rngs::StdRng,
    ) {
        if !self.needs_generation {
            return; // Skip if already generated
        }

        if size == 1 {
            // Base case: single voxel
            self.block_type = Some(1 + rng.gen_range(0..4)); // Example: Random block type
            self.needs_generation = false;
            return;
        }

        // Subdivide into children
        let half = size / 2;
        let mut children: [Option<Box<OctreeNode>>; 8] = Default::default();

        for i in 0..8 {
            let (dx, dy, dz) = (
                (i & 1) * half,
                ((i >> 1) & 1) * half,
                ((i >> 2) & 1) * half,
            );
            let mut child = OctreeNode::new(None, true);
            child.lazy_generate(x + dx, y + dy, z + dz, half, rng);

            children[i] = Some(Box::new(child));
        }

        self.children = Some(children.map(|child| child.unwrap()));

        // Mark node as no longer needing generation
        self.block_type = None; // Mixed node
        self.needs_generation = false;
    }

    pub fn size(&self) -> f32 {
        // This is a placeholder. You'll need to modify this to track the actual size
        // You might want to add a size field to the OctreeNode struct
        1.0 // Example default size
    }

    pub fn refine_octree(
        &mut self,
        camera_position: (f32, f32, f32),
        camera_distance: f32,
        node_size: f32,
        rng: &mut rand::rngs::StdRng,
    ) {
        if Self::needs_detail(camera_distance, node_size) {
            if let Some(children) = &mut self.children {
                for child in children {
                    child.refine_octree(
                        camera_position,
                        camera_distance - node_size,
                        node_size / 2.0,
                        rng,
                    );
                }
            } else if self.needs_generation {
                self.lazy_generate(0, 0, 0, node_size as usize, rng);
            }
        }
    }

    /// Check if this node needs more detail (e.g., based on camera distance or other criteria)
    pub fn needs_detail(camera_distance: f32, node_size: f32) -> bool {
        camera_distance < node_size * 2.0 // Example threshold
    }
    
}
