use std::{mem, ptr};

use cgmath::*;
use gl::types::{GLfloat, GLsizei};

use super::ui_element::UIElement;
use crate::graphics::gl_wrapper::*;

pub struct UIManager {
    elements: Vec<UIElement>,
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    projection: Matrix4<f32>,
}

impl UIManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {//just pull from window later maybe erm or maybe not for customizabnle ui
        // let mut vertices = vec![ 
        // 0.5, 0.5, 0.5,
        // 0.5, 0.5, 0.5,
        // 0.5, 0.5, 0.5,
        // 0.5, 0.5, 0.5];
        // let mut indices = vec![0, 1, 2, 0, 2, 3];
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        // vbo.store_f32_data(&vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        // ebo.store_i32_data(&indices);

        let stride = 6 * mem::size_of::<GLfloat>() as GLsizei;
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();

        let projection = cgmath::ortho(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);
        //shader.create_uniform("projection"); I wan this later

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

        // unsafe {
        //     gl::Disable(gl::DEPTH_TEST);
        // }

        // unsafe {
        //     gl::Enable(gl::BLEND);
        //     gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        // }
        
        

        self.vao.bind();
        for element in &self.elements {
            self.render_ui_element(element, shader);
            //print!("bleh");
        }
    }

    pub fn render_ui_element(&self, element: &UIElement, shader: &ShaderProgram) {
        // shader.set_uniform_matrix4("projection", &self.projection);
        // shader.set_uniform_vector4("color", element.color);

        // let vertices = vec![
        //     0.0, element.get_position().x, element.get_position().y,
        //     0.0, element.get_position().x + element.get_size().x, element.get_position().y,
        //     element.get_position().x + element.get_size().x, element.get_position().y + element.get_size().y, 0.0,
        //     element.get_position().x, element.get_position().y + element.get_size().y, 0.0,
        // ];


        let vertices = vec![
            20.0, 70.0, 50.0, // Bottom-left
            50.0, 70.0, 50.0, // Bottom-right
            50.0, 100.0, 50.0, // Top-right
            20.0, 100.0, 50.0, // Top-left
        ];
        

        let indices: Vec<i32> = vec![
            0, 1, 2, // First triangle (Bottom-left → Bottom-right → Top-right)
            0, 2, 3, // Second triangle (Bottom-left → Top-right → Top-left)
        ];

        self.ebo.bind();
        self.ebo.store_i32_data(&indices);

        self.vbo.bind();
        self.vbo.store_f32_data(&vertices);
    
        // if let Some(texture_id) = element.get_texture_id() {
        //     unsafe {
        //         gl::BindTexture(gl::TEXTURE_2D, texture_id);
        //     }
        // }

        self.vao.bind();
        // unsafe {
        //     gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        // }

        //let elem = element.get_texture_id().unwrap();

        unsafe {
            //gl::Disable(gl::CULL_FACE); 

            //gl::BindTexture(gl::TEXTURE_2D, elem);
            gl::DrawArrays(gl::POINTS, 0, vertices.len() as i32 / 3);
            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_INT, ptr::null());

            //gl::DrawArrays(gl::TRIANGLES, 0, 6); // Assuming a quad made of two triangles
        }

    }
}
