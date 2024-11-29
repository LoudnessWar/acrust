// src/chunk_manager.rs

use crate::voxel_render::VoxelRenderer;
use crate::octo::{OctreeNode, TileRules, generate_wave_function_collapse_as_octree};

pub struct ChunkManager {
    octrees: Vec<(OctreeNode, (f32, f32, f32))>, // Store octrees with positions
    renderer: Option<VoxelRenderer>,            // Renderer for octrees
    tile_rules: TileRules,                      // Rules for block adjacency
}

impl ChunkManager {
    /// Create a new chunk manager with the given tile rules
    pub fn new(tile_rules: TileRules) -> Self {
        Self {
            octrees: Vec::new(),
            renderer: None,
            tile_rules,
        }
    }

    /// Generate and add a new octree-based chunk using Wave Function Collapse
    pub fn add_chunk(&mut self, size: usize, position: (f32, f32, f32), seed: u32) {
        // Generate the octree using WFC
        let octree = generate_wave_function_collapse_as_octree(size, &[0, 1, 2, 3, 4], &self.tile_rules, seed);
        println!("Generated octree");
    
        // Store the generated octree with its position
        self.octrees.push((octree, position));
    
        // Regenerate geometry for the renderer
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;
    
        for (octree, pos) in &self.octrees {
            println!("Octree: made");
            self.generate_geometry(octree, pos.0, pos.1, pos.2, &mut vertices, &mut indices, &mut offset);
        }
    
        println!("Vertices generated: {}", vertices.len());
        println!("Indices generated: {}", indices.len());
    
        // Recreate the renderer only if we have geometry
        if !vertices.is_empty() {
            self.renderer = Some(VoxelRenderer::new(&vertices, &indices));
        } else {
            println!("No geometry generated!");
        }
    }

    /// Recursively generate geometry from an octree
    fn generate_geometry(
        &self,
        octree: &OctreeNode,
        x: f32,
        y: f32,
        z: f32,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<i32>,
        offset: &mut i32,
    ) {
        // Always generate geometry for non-zero block types
        let block_types = octree.block_type();
        println!("Block type: {}", block_types[0]);
        if !block_types.is_empty() && block_types[0] != 0 {
            println!("Block type: {}", block_types[0]);
            let color = VoxelRenderer::get_block_color(block_types[0]);
            let (cube_vertices, cube_indices) = VoxelRenderer::generate_cube(x, y, z, color);
    
            vertices.extend(cube_vertices);
            indices.extend(cube_indices.iter().map(|&i| i + *offset));
            *offset += 24; // 24 vertices per cube
        }
    
        // Recursively process children
        if let Some(children) = &octree.children() {
            let half_size = 1.0; // Adjust based on node size
            for (i, child) in children.iter().enumerate() {
                let (dx, dy, dz) = (
                    (i & 1) as f32 * half_size,
                    ((i >> 1) & 1) as f32 * half_size,
                    ((i >> 2) & 1) as f32 * half_size,
                );
                self.generate_geometry(child, x + dx, y + dy, z + dz, vertices, indices, offset);
            }
        }
    }

    /// Render all octree chunks
    pub fn render_all(&self) {
        if let Some(renderer) = &self.renderer {
            renderer.render();
        }
    }
}
