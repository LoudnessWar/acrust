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

    // Equivalent to hierarchical_terrain_generation//note i have no clue what this comment was baout
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

    pub fn generate_multiple_chunks(
        &mut self, 
        center_x: usize, 
        center_y: usize, 
        center_z: usize, 
        x_radius: isize,  // Radius along X axis
        z_radius: isize,  // Radius along Z axis
        chunk_size: usize
    ) -> Vec<(OctreeNode, (f32, f32, f32))> {
        let mut chunks = Vec::new();

        // Iterate through a region along X and Z, keeping Y constant
        for dx in -x_radius..=x_radius {
            for dz in -z_radius..=z_radius {
                // Convert signed offsets to signed chunk coordinates
                let signed_chunk_x = center_x as isize + (dx * chunk_size as isize);
                let signed_chunk_z = center_z as isize + (dz * chunk_size as isize);

                // Safely convert back to usize, handling potential overflow
                let chunk_x = signed_chunk_x.try_into().unwrap_or(0);
                let chunk_z = signed_chunk_z.try_into().unwrap_or(0);

                // Use the constant Y coordinate
                let chunk_y = center_y;

                // Calculate world position for rendering
                let world_x = signed_chunk_x as f32;
                let world_y = center_y as f32;
                let world_z = signed_chunk_z as f32;

                // Generate a seed for this specific chunk
                let chunk_seed = Self::calculate_seed(chunk_x, chunk_y, chunk_z, chunk_size);
                
                // Use the chunk seed to create a deterministic RNG
                let mut chunk_rng = StdRng::seed_from_u64(chunk_seed);

                // Generate the octree for this chunk
                let mut chunk_octree = OctreeNode::new(None, true);
                chunk_octree.lazy_generate(chunk_x, chunk_y, chunk_z, chunk_size, &mut chunk_rng);

                // Add the chunk to the collection
                chunks.push((chunk_octree, (world_x, world_y, world_z)));
            }
        }

        chunks
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