// src/chunk_manager.rs

use crate::chunk_generator::VoxelChunk;
use crate::voxel_render::VoxelRenderer;
use crate::octo::{build_octree, OctreeNode};

pub struct ChunkManager {
    chunks: Vec<(VoxelChunk, (f32, f32, f32))>,
    octrees: Vec<(OctreeNode, (f32, f32, f32))>,
    renderer: Option<VoxelRenderer>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            octrees: Vec::new(),
            renderer: None,
        }
    }

    pub fn add_chunk(&mut self, chunk: VoxelChunk, position: (f32, f32, f32)) {
        // Build and store the octree
        let octree = build_octree(&chunk, 0, 0, 0, chunk.get_size().0);
        self.octrees.push((octree, position));

        // Generate renderer data
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;

        for (octree, pos) in &self.octrees {
            self.generate_geometry(octree, pos.0, pos.1, pos.2, &mut vertices, &mut indices, &mut offset);
        }

        // Recreate renderer
        self.renderer = Some(VoxelRenderer::new(&vertices, &indices));
    }

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
        if let Some(block_type) = octree.block_type() {
            if block_type != 0 {
                let color = VoxelRenderer::get_block_color(block_type);
                let (cube_vertices, cube_indices) = VoxelRenderer::generate_cube(x, y, z, color);

                vertices.extend(cube_vertices);
                indices.extend(cube_indices.iter().map(|&i| i + *offset));
                *offset += 24; // 24 vertices per cube
            }
        } else if let Some(children) = octree.children() {
            let size = 1.0; // Adjust as necessary
            for (i, child) in children.iter().enumerate() {
                let (dx, dy, dz) = (
                    (i & 1) as f32 * size,
                    ((i >> 1) & 1) as f32 * size,
                    ((i >> 2) & 1) as f32 * size,
                );
                self.generate_geometry(child, x + dx, y + dy, z + dz, vertices, indices, offset);
            }
        }
    }

    pub fn render_all(&self) {
        if let Some(renderer) = &self.renderer {
            renderer.render();
        }
    }
}
