use rand::{prelude::*, rngs::StdRng, SeedableRng};
use std::collections::{HashMap, HashSet};

use crate::octo::OctreeNode;
pub struct TerrainGenerator {
    rng: StdRng,
    root: OctreeNode,
}

impl TerrainGenerator {
    pub fn new(seed: u32, size: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed as u64);
        let root = Self::hierarchical_terrain_generation(0, 0, 0, size, &mut rng);
        
        Self {
            rng,
            root,
        }
    }

    // Equivalent to the previous generate_terrain_octree function
    pub fn get_root(&self) -> &OctreeNode {
        &self.root
    }

    // Equivalent to hierarchical_terrain_generation
    fn hierarchical_terrain_generation(
        x: usize,
        y: usize,
        z: usize,
        size: usize,
        rng: &mut StdRng,
    ) -> OctreeNode {
        let mut root = OctreeNode::new(None, true);
        root.lazy_generate(x, y, z, size, rng);
        root
    }

    // New method to refine the octree, using the stored RNG
    pub fn refine_octree(&mut self, camera_position: (f32, f32, f32), camera_distance: f32) {
        self.root.refine_octree(camera_position, camera_distance, self.root.size(), &mut self.rng);
    }

    // Seed calculation method
    pub fn calculate_seed(x: usize, y: usize, z: usize, size: usize) -> u64 {
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
    fn generate_top_level_terrain(&mut self) -> Option<u8> {
        match self.rng.gen_range(0..100) {
            0..=40 => Some(1),  // Grass plains
            41..=70 => Some(3), // Stone terrain
            71..=90 => Some(2), // Dirt hills
            _ => Some(4)         // Bedrock regions
        }
    }

    // Derive child terrain based on parent block type
    fn derive_child_terrain(&mut self, parent_type: u8) -> Option<u8> {
        match parent_type {
            0 => Some(1), // Air becomes grass
            1 => match self.rng.gen_range(0..100) {
                0..=70 => Some(1),  // Mostly grass
                71..=90 => Some(2), // Some dirt
                _ => Some(0)         // Occasional air
            },
            2 => match self.rng.gen_range(0..100) {
                0..=60 => Some(2),  // Mostly dirt
                61..=85 => Some(3), // Some stone
                _ => Some(1)         // Occasional grass
            },
            3 => match self.rng.gen_range(0..100) {
                0..=75 => Some(3),  // Mostly stone
                76..=95 => Some(4), // Some bedrock
                _ => Some(2)         // Occasional dirt
            },
            4 => Some(4), // Bedrock stays bedrock
            _ => Some(3)  // Default to stone
        }
    }

    // Generate a single block based on parent context
    fn generate_single_block(&mut self, parent_type: Option<u8>) -> Option<u8> {
        match parent_type {
            None => self.generate_top_level_terrain(),
            Some(block_type) => self.derive_child_terrain(block_type)
        }
    }
}