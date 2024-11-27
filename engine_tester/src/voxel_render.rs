use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

use rand::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::chunk_generator::VoxelChunk;

// VoxelRenderer for rendering voxels or chunks
pub struct VoxelRenderer {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
}

impl VoxelRenderer {
    // Create a VoxelRenderer from a chunk
    pub fn from_chunk(chunk: &VoxelChunk) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;
        //let mut block_colors = Vec::new();

        let (width, height, depth) = chunk.get_size();

        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    let block_type = chunk.get_block(x, y, z);
                    if block_type != 0 {
                        // Add cube geometry at position (x, y, z)
                        let mut cube_vertices = Self::generate_cube_vertices(x as f32, y as f32, z as f32);
                        let cube_indices = Self::generate_cube_indices(index_offset);

                        // Get color for this block type
                        let block_color = Self::get_block_color(block_type);

                        // Extend vertices with color for each vertex
                        let mut colored_vertices = Vec::new();
                        for vertex in cube_vertices.chunks(3) {
                            colored_vertices.extend_from_slice(vertex);
                            colored_vertices.extend_from_slice(&block_color);
                        }
                        
                        vertices.extend_from_slice(&colored_vertices);
                        indices.extend_from_slice(&cube_indices);

                        index_offset += 24; // 24 vertices per cube
                    }
                }
            }
        }

        let index_count = indices.len() as i32;

        // Initialize buffers
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(&indices);

        // Set vertex attributes
        let position_attribute = VertexAttribute::new(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,//6 if with color 3 without so 3 now
            ptr::null(),
        );
        position_attribute.enable();

        let color_attribute = VertexAttribute::new(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            (3 * mem::size_of::<GLfloat>()) as *const GLvoid,
        );
        color_attribute.enable();

        VoxelRenderer { vao, vbo, ebo, index_count }
    }

    // Generate cube vertices at a specific position
    fn generate_cube_vertices(x: f32, y: f32, z: f32) -> Vec<f32> {
        vec![
            // Front face
            x - 0.5, y - 0.5, z + 0.5,
            x + 0.5, y - 0.5, z + 0.5,
            x + 0.5, y + 0.5, z + 0.5,
            x - 0.5, y + 0.5, z + 0.5,
            
            // Back face
            x - 0.5, y - 0.5, z - 0.5,
            x + 0.5, y - 0.5, z - 0.5,
            x + 0.5, y + 0.5, z - 0.5,
            x - 0.5, y + 0.5, z - 0.5,
            
            // Left Face
            x - 0.5, y - 0.5, z - 0.5,
            x - 0.5, y - 0.5, z + 0.5,
            x - 0.5, y + 0.5, z + 0.5,
            x - 0.5, y + 0.5, z - 0.5,
            
            // Right face
            x + 0.5, y - 0.5, z - 0.5,
            x + 0.5, y + 0.5, z - 0.5,
            x + 0.5, y + 0.5, z + 0.5,
            x + 0.5, y - 0.5, z + 0.5,
            
            // Top face
            x - 0.5, y + 0.5, z - 0.5,
            x - 0.5, y + 0.5, z + 0.5,
            x + 0.5, y + 0.5, z + 0.5,
            x + 0.5, y + 0.5, z - 0.5,
            
            // Bottom face
            x - 0.5, y - 0.5, z - 0.5,
            x + 0.5, y - 0.5, z - 0.5,
            x + 0.5, y - 0.5, z + 0.5,
            x - 0.5, y - 0.5, z + 0.5,
        ]
    }

    // Generate cube indices
    fn generate_cube_indices(offset: i32) -> Vec<i32> {
        vec![
            0 + offset, 1 + offset, 2 + offset, 0 + offset, 2 + offset, 3 + offset, // Front
            4 + offset, 5 + offset, 6 + offset, 4 + offset, 6 + offset, 7 + offset, // Back
            8 + offset, 9 + offset, 10 + offset, 8 + offset, 10 + offset, 11 + offset, // left
            12 + offset, 13 + offset, 14 + offset, 14 + offset, 15 + offset, 12 + offset, // right//ook notive how this one id different from here on out... yeah da triangles can be made like this 2 ig
            16 + offset, 17 + offset, 18 + offset, 18 + offset, 19 + offset, 16 + offset, // top
            20 + offset, 21 + offset, 22 + offset, 22 + offset, 23 + offset, 20 + offset // bottom
        ]
    }

    fn get_block_color(block_type: u8) -> [f32; 3] {
        match block_type {
            0 => [0.0, 0.0, 0.0],     // Air (transparent)
            1 => [0.2, 0.8, 0.2],     // Grass (green)
            2 => [0.5, 0.35, 0.05],   // Dirt (brown)
            3 => [0.5, 0.5, 0.5],     // Stone (gray)
            4 => [0.2, 0.2, 0.2],     // Bedrock (dark gray)
            _ => [1.0, 0.0, 1.0],     // Magenta for unknown
        }
    }

    // Render the chunk
    pub fn render(&self) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

