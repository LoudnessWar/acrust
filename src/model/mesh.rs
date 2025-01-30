use std::mem;
use std::ptr;

use crate::graphics::gl_wrapper::Vao;
use crate::graphics::gl_wrapper::VertexAttribute;
use crate::graphics::gl_wrapper::BufferObject;

pub struct Mesh {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
}

impl Mesh {
    pub fn new(vertices: &[f32], indices: &[i32]) -> Self {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(indices);//why is this not u 32?... maybe useful in some weird situation

        // Set vertex attributes
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<f32>() as i32, 0 as *const _).enable();
        VertexAttribute::new(1, 3, gl::FLOAT, gl::FALSE, 6 * mem::size_of::<f32>() as i32, (3 * mem::size_of::<f32>()) as *const _).enable();

        Self {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
        }
    }

    pub fn draw(&self) {
        self.vao.bind();
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}