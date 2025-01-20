use std::{mem, ptr};

use cgmath::*;
use gl::types::{GLfloat, GLsizei};

use super::ui_element::UIElement;
use crate::graphics::gl_wrapper::*;

pub struct UIManager {//lol this got uuuuuh sanatized since last time i coimmited lol
    elements: Vec<UIElement>,
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    projection: Matrix4<f32>,
}

impl UIManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let vao = Vao::new();
        vao.bind();

        // Vertex buffer object
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();

        // Element buffer object
        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();

        // Define vertex attribute layout (3 floats per vertex for position)
        let stride = 3 * mem::size_of::<GLfloat>() as GLsizei;
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();

        // Create orthographic projection matrix
        let projection = cgmath::ortho(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);

        Self {
            elements: Vec::new(),
            vao,
            vbo,
            ebo,
            projection,
        }
    }

    pub fn add_element(&mut self, element: UIElement) {
        self.elements.push(element);
    }

    pub fn render(&self, shader: &ShaderProgram) {
        shader.bind();
        shader.set_matrix4fv_uniform("projection", &self.projection);

        self.vao.bind();

        for element in &self.elements {
            self.render_ui_element(element, shader);
        }
    }

    pub fn render_ui_element(&self, element: &UIElement, shader: &ShaderProgram) {
        // Define vertices for the quad
        // let vertices = vec![//test lol
        //     20.0, 500.0, 0.0, // Top-left
        //     500.0, 500.0, 0.0, // Top-right
        //     500.0, 100.0, 0.0, // Bottom-right
        //     20.0, 100.0, 0.0, // Bottom-left
        // ];

        let vertices: Vec<f32> = vec![
            element.get_position().x, element.get_position().y + element.get_size().y, 0.0,
            element.get_position().x + element.get_size().x, element.get_position().y + element.get_size().y, 0.0,
            element.get_position().x + element.get_size().x, element.get_position().y, 0.0,
            element.get_position().x, element.get_position().y, 0.0,
        ];

        // Define indices for two triangles
        let indices: Vec<i32> = vec![
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
        ];

        // Upload vertex data
        self.vbo.bind();
        self.vbo.store_f32_data(&vertices);

        // Upload index data
        self.ebo.bind();
        self.ebo.store_i32_data(&indices);

        if let Some(texture_id) = element.get_texture_id() {
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, texture_id);
            }
        }

        // Render the quad
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as i32, // Number of indices
                gl::UNSIGNED_INT,
                ptr::null(),          // Offset in the index buffer
            );
        }
    }
}
