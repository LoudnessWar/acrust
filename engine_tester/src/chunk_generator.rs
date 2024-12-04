use rand::{prelude::*, rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};

use crate::octo::OctreeNode;

pub fn generate_terrain_octree(size: usize, seed: u32) -> OctreeNode {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    hierarchical_terrain_generation(None, 0, 0, 0, size, &mut rng, 0)
}

// pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: u8) {
//     let (width, height, _) = self.size;
//     if x < width && y < height {
//         self.blocks[x + y * width + z * width * height] = block_type;
//     }
// }

// pub fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
//     let (width, height, _) = self.size;
//     if x < width && y < height {
//         self.blocks[x + y * width + z * width * height]
//     } else {
//         0 //air
//     }
// }

// pub fn get_size(&self) ->  (usize, usize, usize) {
//     self.size
// }


// Hierarchical terrain generation function
fn hierarchical_terrain_generation(
    parent_block_type: Option<u8>, 
    x: usize, 
    y: usize, 
    z: usize, 
    size: usize,
    rng: &mut StdRng,
    depth: usize,
) -> OctreeNode {
    println!("Generating terrain node at depth {} for region ({}, {}, {}) size {}", depth, x, y, z, size);

    // Base case
    if size == 1 {
        return OctreeNode {
            block_type: Some(1 + rng.gen_range(0..4)), // Randomized block type
            children: None,
            needs_generation: false,
        };
    }

    // Subdivision threshold adjustment
    if rng.gen_range(0.0..1.0) > 0.1 {
        let half = size / 2;
        let mut children: [Box<OctreeNode>; 8] = Default::default();
        for i in 0..8 {
            let (dx, dy, dz) = (
                (i & 1) * half,
                ((i >> 1) & 1) * half,
                ((i >> 2) & 1) * half,
            );
            children[i] = Box::new(hierarchical_terrain_generation(None, x + dx, y + dy, z + dz, half, rng, depth + 1));
        }
        return OctreeNode {
            block_type: None,
            children: Some(children),
            needs_generation: false,
        };
    }

    // Homogeneous node
    OctreeNode {
        block_type: Some(1 + rng.gen_range(0..4)), // Assign a block type
        children: None,
        needs_generation: false,
    }
}

// Seed calculation for deterministic generation
fn calculate_seed(x: usize, y: usize, z: usize, size: usize) -> u64 {
    let mut seed = 0;
    seed ^= x as u64;
    seed = seed.wrapping_mul(0x9E3779B97F4A7C15);
    seed ^= y as u64;
    seed = seed.wrapping_mul(0x9E3779B97F4A7C15);
    seed ^= z as u64;
    seed = seed.wrapping_mul(0x9E3779B97F4A7C15);
    seed ^= size as u64;
    seed
}

// Generate top-level terrain type
fn generate_top_level_terrain(rng: &mut StdRng) -> Option<u8> {
    match rng.gen_range(0..100) {
        0..=40 => Some(1),  // Grass plains
        41..=70 => Some(3), // Stone terrain
        71..=90 => Some(2), // Dirt hills
        _ => Some(4)         // Bedrock regions
    }
}

// Derive child terrain based on parent block type
fn derive_child_terrain(parent_type: u8, rng: &mut StdRng) -> Option<u8> {
    match parent_type {
        0 => Some(1), // Air becomes grass
        1 => match rng.gen_range(0..100) {
            0..=70 => Some(1),  // Mostly grass
            71..=90 => Some(2), // Some dirt
            _ => Some(0)         // Occasional air
        },
        2 => match rng.gen_range(0..100) {
            0..=60 => Some(2),  // Mostly dirt
            61..=85 => Some(3), // Some stone
            _ => Some(1)         // Occasional grass
        },
        3 => match rng.gen_range(0..100) {
            0..=75 => Some(3),  // Mostly stone
            76..=95 => Some(4), // Some bedrock
            _ => Some(2)         // Occasional dirt
        },
        4 => Some(4), // Bedrock stays bedrock
        _ => Some(3)  // Default to stone
    }
}

// Generate a single block based on parent context
fn generate_single_block(parent_type: Option<u8>, rng: &mut StdRng) -> Option<u8> {
    match parent_type {
        None => generate_top_level_terrain(rng),
        Some(block_type) => derive_child_terrain(block_type, rng)
    }
}


    // pub fn lazy_generate(&mut self, chunk: &VoxelChunk, x: usize, y: usize, z: usize, size: usize) {
    //     // Only generate if marked as needing generation
    //     if !self.needs_generation {
    //         return;
    //     }

    //     // Similar to existing build_octree logic, but with wave function collapse
    //     if size == 1 {
    //         let block_type = chunk.get_block(x, y, z);
    //         self.block_type = if block_type == 0 { None } else { Some(block_type) };
    //         self.needs_generation = false;
    //         return;
    //     }

    //     let half = size / 2;
    //     let mut children: [Option<Box<OctreeNode>>; 8] = Default::default();
    //     let mut block_types = Vec::new();
    //     let mut is_homogeneous = true;

    //     for i in 0..8 {
    //         let (dx, dy, dz) = (
    //             (i & 1) * half,
    //             ((i >> 1) & 1) * half,
    //             ((i >> 2) & 1) * half,
    //         );

    //         let mut child = OctreeNode {
    //             block_type: None,
    //             children: None,
    //             needs_generation: true, // Mark child for lazy generation
    //         };

    //         // Only generate if near view or needed
    //         child.lazy_generate(&chunk.clone(), x + dx, y + dy, z + dz, half);

    //         if let Some(block_type) = child.block_type {
    //             block_types.push(block_type);
    //         } else {
    //             is_homogeneous = false;
    //         }
    //         children[i] = Some(Box::new(child));
    //     }

    //     // Optimization for homogeneous regions
    //     if is_homogeneous && !block_types.is_empty() && block_types.iter().all(|&t| t == block_types[0]) {
    //         self.block_type = Some(block_types[0]);
    //         self.children = None;
    //     } else {
    //         self.block_type = None;
    //         self.children = Some(children.map(|child| child.unwrap()));
    //     }

    //     self.needs_generation = true;
    // }

    // pub fn needs_detail(camera_distance: f32, node_size: f32) -> bool {
    //     // Example threshold logic
    //     //camera_distance < node_size * 2.0
    //     true //rn true but later it will be better
    // }