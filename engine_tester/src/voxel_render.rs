use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

pub struct VoxelRenderer {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
}

impl VoxelRenderer {
    // Create a new VoxelRenderer
    pub fn new() -> Self {
        // Cube vertices (coordinates)
        let verticies: [f32; 144] = [
            // Front face
            -0.5, -0.5,  0.5, 1.0, 0.0, 0.0, // Red
             0.5, -0.5,  0.5, 1.0, 0.0, 0.0, // Red
             0.5,  0.5,  0.5, 1.0, 0.0, 0.0, // Red
            -0.5,  0.5,  0.5, 1.0, 0.0, 0.0, // Red
            // Back face
            -0.5, -0.5, -0.5, 0.0, 0.2, 0.0, // Green
            -0.5,  0.5, -0.5, 0.0, 1.0, 0.0, // Green
             0.5,  0.5, -0.5, 0.0, 1.0, 0.0, // Green
             0.5, -0.5, -0.5, 0.0, 1.0, 0.0, // Green
            // // Left face
            -0.5, -0.5, -0.5, 0.0, 0.0, 0.3, // Blue
            -0.5, -0.5,  0.5, 0.0, 0.0, 1.0, // Blue
            -0.5,  0.5,  0.5, 0.0, 0.0, 1.0, // Blue
            -0.5,  0.5, -0.5, 0.0, 0.0, 1.0, // Blue
            // // Right face
             0.5, -0.5, -0.5, 0.3, 1.0, 0.0, // Yellow
             0.5,  0.5, -0.5, 1.0, 1.0, 0.0, // Yellow
             0.5,  0.5,  0.5, 1.0, 1.0, 0.0, // Yellow
             0.5, -0.5,  0.5, 1.0, 1.0, 0.0, // Yellow
            // Top face
            -0.5,  0.5, -0.5, 0.5, 0.0, 1.0, // Purple
            -0.5,  0.5,  0.5, 0.5, 0.0, 1.0, // Purple
             0.5,  0.5,  0.5, 0.5, 0.0, 1.0, // Purple
             0.5,  0.5, -0.5, 0.5, 0.0, 1.0, // Purple
            // Bottom face
            -0.5, -0.5, -0.5, 1.0, 0.0, 1.0, // Magenta
             0.5, -0.5, -0.5, 1.0, 0.0, 1.0, // Magenta
             0.5, -0.5,  0.5, 1.0, 0.0, 1.0, // Magenta
            -0.5, -0.5,  0.5, 1.0, 0.0, 1.0, // Magenta
        ];

        // Indices for the cube faces
        let indices: [i32; 36] = [
            0, 1, 2,  0, 2, 3,  // front
            4, 5, 6,  4, 6, 7,  // back
            8, 9, 10, 8, 10, 11, // left
            12, 13, 14, 14, 15, 12, // right//ook notive how this one id different from here on out... yeah da triangles can be made like this 2 ig
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20  // bottom
        ];

        // Initialize buffers
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&verticies);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(&indices);

        // Set vertex attributes
        let position_attribute = VertexAttribute::new(
            0,  // attribute index
            3,  // size (3 components per vertex)
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        position_attribute.enable();

        let color_attribute = VertexAttribute::new(
            1,  // attribute index for color
            3,  // size (3 components per color)
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<GLfloat>() as GLsizei, // Offset for color data
            (3 * mem::size_of::<GLfloat>()) as *const GLvoid, // Color offset
        );
        color_attribute.enable();


        VoxelRenderer { vao, vbo, ebo }
    }

    // Render the voxel (cube)
    pub fn render(&self) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, ptr::null());
        }
    }
}
