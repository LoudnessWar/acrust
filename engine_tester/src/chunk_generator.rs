use rand::{prelude::*, rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};

use crate::octo::{OctreeNode, build_octree};

#[derive(Clone)]
pub struct VoxelChunk {
    size: (usize, usize, usize), // Dimensions of the chunk
    blocks: Vec<u8>,             // Block data
}

impl VoxelChunk {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        let blocks = vec![0; width * height * depth]; // All blocks initially empty
        VoxelChunk {
            size: (width, height, depth),
            blocks,
        }
    }

    // Hierarchical terrain generation method
    pub fn generate_hierarchical_terrain(&mut self, seed: u32) {
        // Create a root octree with hierarchical generation
        let octree = self.generate_terrain_octree(seed);

        // Convert octree to block data
        self.populate_from_octree(&octree);
    }

    // Generate terrain octree
    fn generate_terrain_octree(&self, seed: u32) -> OctreeNode {
        let mut rng = StdRng::seed_from_u64(seed as u64);
        hierarchical_terrain_generation(None, 0, 0, 0, self.size.0, &mut rng)
    }

    // Populate chunk blocks from octree
    fn populate_from_octree(&mut self, octree: &OctreeNode) {
        self.recursive_octree_fill(octree, 0, 0, 0, self.size.0);
    }

    // Recursively fill chunk with octree data
    fn recursive_octree_fill(&mut self, 
                              node: &OctreeNode, 
                              x: usize, 
                              y: usize, 
                              z: usize, 
                              size: usize) {
        if size == 1 {
            // Single voxel
            if let Some(block_type) = node.block_type() {
                self.set_block(x, y, z, block_type);
            }
            return;
        }

        // If node is a leaf
        if node.children().is_none() {
            let block_type = node.block_type().unwrap_or(0);
            for dx in 0..size {
                for dy in 0..size {
                    for dz in 0..size {
                        self.set_block(x + dx, y + dy, z + dz, block_type);
                    }
                }
            }
            return;
        }

        // Recursive descent
        if let Some(children) = node.children() {
            let half = size / 2;
            for (i, child) in children.iter().enumerate() {
                let (dx, dy, dz) = (
                    (i & 1) * half,
                    ((i >> 1) & 1) * half,
                    ((i >> 2) & 1) * half,
                );
                self.recursive_octree_fill(child, x + dx, y + dy, z + dz, half);
            }
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: u8) {
        let (width, height, _) = self.size;
        if x < width && y < height {
            self.blocks[x + y * width + z * width * height] = block_type;
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
        let (width, height, _) = self.size;
        if x < width && y < height {
            self.blocks[x + y * width + z * width * height]
        } else {
            0 // Default to air for out-of-bounds
        }
    }

    pub fn get_size(&self) ->  (usize, usize, usize) {
        self.size
    }
}

// Hierarchical terrain generation function
fn hierarchical_terrain_generation(
    parent_block_type: Option<u8>, 
    x: usize, 
    y: usize, 
    z: usize, 
    size: usize,
    rng: &mut StdRng
) -> OctreeNode {
    // Deterministic seed based on coordinates
    let seed = calculate_seed(x, y, z, size);
    let mut local_rng = StdRng::seed_from_u64(seed);

    // Base case: single voxel
    if size == 1 {
        return OctreeNode {
            block_type: generate_single_block(parent_block_type, &mut local_rng),
            children: None,
            needs_generation: true,
        };
    }

    // Determine terrain type based on parent or generate top-level terrain
    let terrain_type = match parent_block_type {
        None => generate_top_level_terrain(&mut local_rng),
        Some(block_type) => derive_child_terrain(block_type, &mut local_rng)
    };

    // If uniform terrain, create a leaf node
    if local_rng.gen_range(0.0..1.0) > 0.3 {
        return OctreeNode {
            block_type: terrain_type,
            children: None,
            needs_generation: true,
        };
    }

    // Recursive child generation
    let half = size / 2;
    let mut children: [Box<OctreeNode>; 8] = Default::default();

    for i in 0..8 {
        let (dx, dy, dz) = (
            (i & 1) * half,
            ((i >> 1) & 1) * half,
            ((i >> 2) & 1) * half,
        );

        children[i] = Box::new(hierarchical_terrain_generation(
            terrain_type, 
            x + dx, 
            y + dy, 
            z + dz, 
            half,
            &mut local_rng
        ));
    }

    OctreeNode {
        block_type: None, // Mixed node
        children: Some(children),
        needs_generation: true,
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