use cgmath::Matrix4;
use gl::types::GLfloat;
use std::{mem, ptr};
use super::ui_element::*;

use gl::types::GLsizei;

use super::ui_element::UIElement;
use crate::graphics::gl_wrapper::*;
use crate::input::input::*;


// UIManager class
pub struct UIManager {
    elements: Vec<Box<dyn UIElementTrait>>,
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    projection: Matrix4<f32>,
}

impl UIManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();

        let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();
        VertexAttribute::new(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<GLfloat>()) as *const _,
        )
        .enable();

        let projection = cgmath::ortho(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);

        Self {
            elements: Vec::new(),
            vao,
            vbo,
            ebo,
            projection,
        }
    }

    pub fn add_element(&mut self, element: Box<dyn UIElementTrait>) {
        self.elements.push(element);
    }

    pub fn render(&self, shader: &ShaderProgram) {
        shader.bind();
        shader.set_matrix4fv_uniform("projection", &self.projection);
        self.vao.bind();

        for element in &self.elements {
            self.render_ui_element(element.as_ref(), shader);
        }
    }

    pub fn render_ui_element(&self, element: &dyn UIElementTrait, shader: &ShaderProgram) {
        let vertices: Vec<f32> = vec![//ok this should be saved somewhere and not done every render call...
            element.get_position().x, element.get_position().y + element.get_size().y, 0.0,  0.0, 1.0, // Top-left
            element.get_position().x + element.get_size().x, element.get_position().y + element.get_size().y, 0.0,  1.0, 1.0, // Top-right
            element.get_position().x + element.get_size().x, element.get_position().y, 0.0,  1.0, 0.0, // Bottom-right
            element.get_position().x, element.get_position().y, 0.0,  0.0, 0.0, // Bottom-left
        ];

        let indices: Vec<i32> = vec![//same here
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
            shader.set_uniform1i("useTexture", &1);//i hate this i think
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture_id);
            }
        } else {
            shader.set_uniform1i("useTexture", &0);
            shader.set_uniform4f("color", &element.get_color());
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