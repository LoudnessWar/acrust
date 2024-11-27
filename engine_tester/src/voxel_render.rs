// src/voxel_renderer.rs

use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

use crate::octo::OctreeNode;

pub struct VoxelRenderer {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
}

impl VoxelRenderer {
    /// Create a new renderer from precomputed vertex and index data.
    pub fn new(vertices: &[f32], indices: &[i32]) -> Self {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(indices);

        let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const GLvoid).enable();

        VoxelRenderer {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
        }
    }

    pub fn get_block_color(block_type: u8) -> [f32; 3] {
        match block_type {
            0 => [0.0, 0.0, 0.0],     // Air (transparent)
            1 => [0.2, 0.8, 0.2],     // Grass (green)
            2 => [0.5, 0.35, 0.05],   // Dirt (brown)
            3 => [0.5, 0.5, 0.5],     // Stone (gray)
            4 => [0.2, 0.2, 0.2],     // Bedrock (dark gray)
            _ => [1.0, 0.0, 1.0],     // Magenta for unknown
        }
    }

    /// Render the stored geometry.
    pub fn render(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, ptr::null());
        }
    }

    /// Generate vertices and indices for a single cube.
    pub fn generate_cube(x: f32, y: f32, z: f32, color: [f32; 3]) -> (Vec<f32>, Vec<i32>) {
        let vertices = vec![
            // Add positions with colors interleaved
            x - 0.5, y - 0.5, z + 0.5, color[0], color[1], color[2], // Front-bottom-left
            x + 0.5, y - 0.5, z + 0.5, color[0], color[1], color[2], // Front-bottom-right
            x + 0.5, y + 0.5, z + 0.5, color[0], color[1], color[2], // Front-top-right
            x - 0.5, y + 0.5, z + 0.5, color[0], color[1], color[2], // Front-top-left
            x - 0.5, y - 0.5, z - 0.5, color[0], color[1], color[2], // Back-bottom-left
            x + 0.5, y - 0.5, z - 0.5, color[0], color[1], color[2], // Back-bottom-right
            x + 0.5, y + 0.5, z - 0.5, color[0], color[1], color[2], // Back-top-right
            x - 0.5, y + 0.5, z - 0.5, color[0], color[1], color[2], // Back-top-left
        ];

        let indices = vec![
            0, 1, 2, 0, 2, 3, // Front
            4, 5, 6, 4, 6, 7, // Back
            0, 4, 7, 0, 7, 3, // Left
            1, 5, 6, 1, 6, 2, // Right
            3, 2, 6, 3, 6, 7, // Top
            0, 1, 5, 0, 5, 4, // Bottom
        ];

        (vertices, indices)
    }
}
