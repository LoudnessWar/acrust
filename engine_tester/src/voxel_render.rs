use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

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

        let (width, height, depth) = chunk.size;

        for z in 0..depth {
            for y in 0..height {
                for x in 0..width {
                    if chunk.get_block(x, y, z) != 0 {
                        // Add cube geometry at position (x, y, z)
                        let cube_vertices = Self::generate_cube_vertices(x as f32, y as f32, z as f32);
                        let cube_indices = Self::generate_cube_indices(index_offset);

                        vertices.extend_from_slice(&cube_vertices);
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
            6 * mem::size_of::<GLfloat>() as GLsizei,
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
            // Front face (Red)
            x - 0.5, y - 0.5, z + 0.5, 1.0, 0.0, 0.0, // Red
            x + 0.5, y - 0.5, z + 0.5, 1.0, 0.0, 0.0, // Red
            x + 0.5, y + 0.5, z + 0.5, 1.0, 0.0, 0.0, // Red
            x - 0.5, y + 0.5, z + 0.5, 0.7, 0.0, 0.0, // Red
            
            // Back face (Green)
            x - 0.5, y - 0.5, z - 0.5, 0.0, 0.7, 0.0, // Green
            x + 0.5, y - 0.5, z - 0.5, 0.0, 1.0, 0.0, // Green
            x + 0.5, y + 0.5, z - 0.5, 0.0, 1.0, 0.0, // Green
            x - 0.5, y + 0.5, z - 0.5, 0.0, 1.0, 0.0, // Green
            
            // Left Face (Blue)
            x - 0.5, y - 0.5, z - 0.5, 0.0, 0.0, 0.3, // Blue
            x - 0.5, y - 0.5, z + 0.5, 0.0, 0.0, 1.0, // Blue
            x - 0.5, y + 0.5, z + 0.5, 0.0, 0.0, 1.0, // Blue
            x - 0.5, y + 0.5, z - 0.5, 0.0, 0.0, 1.0, // Blue
            
            // Right face (Yellow)
            x + 0.5, y - 0.5, z - 0.5, 0.3, 1.0, 0.0, // Yellow
            x + 0.5, y + 0.5, z - 0.5, 1.0, 1.0, 0.0, // Yellow
            x + 0.5, y + 0.5, z + 0.5, 1.0, 0.2, 0.0, // Yellow
            x + 0.5, y - 0.5, z + 0.5, 1.0, 1.0, 0.0, // Yellow
            
            // Top face (Purple)
            x - 0.5, y + 0.5, z - 0.5, 0.5, 0.0, 1.0, // Purple
            x - 0.5, y + 0.5, z + 0.5, 0.5, 0.0, 0.3, // Purple
            x + 0.5, y + 0.5, z + 0.5, 0.3, 0.0, 1.0, // Purple
            x + 0.5, y + 0.5, z - 0.5, 0.5, 0.0, 1.0, // Purple
            
            // Bottom face (Magenta)
            x - 0.5, y - 0.5, z - 0.5, 1.0, 0.0, 1.0, // Magenta
            x + 0.5, y - 0.5, z - 0.5, 1.0, 0.0, 1.0, // Magenta
            x + 0.5, y - 0.5, z + 0.5, 1.0, 1.0, 1.0, // Magenta
            x - 0.5, y - 0.5, z + 0.5, 1.0, 0.0, 1.0, // Magenta
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

    // Render the chunk
    pub fn render(&self) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, ptr::null());
        }
    }
}

// VoxelChunk represents a grid of blocks
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

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: u8) {
        let (width, height, _) = self.size;
        self.blocks[x + y * width + z * width * height] = block_type;
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
        let (width, height, _) = self.size;
        self.blocks[x + y * width + z * width * height]
    }
    
}
