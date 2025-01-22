use cgmath::Matrix4;
use gl::types::GLfloat;
use std::{mem, ptr};
use super::ui_element::*;

use gl::types::GLsizei;
use cgmath::{Vector2, Vector4};

use crate::graphics::gl_wrapper::*;
use std::collections::VecDeque;

// Define possible UI events
#[derive(Clone, Debug)]
pub enum UIEvent {
    Hover(u32),      // Element ID that's being hovered
    Click(u32),      // Element ID that's been clicked
    MouseEnter(u32), // Element ID mouse entered
    MouseExit(u32),  // Element ID mouse exited
}

pub struct UIManager {
    elements: Vec<Box<dyn UIElementTrait>>,
    event_queue: VecDeque<UIEvent>,
    last_hover_state: Vec<(u32, bool)>, // Tracks previous hover states for MouseEnter/Exit
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
            event_queue: VecDeque::new(),
            last_hover_state: Vec::new(),
            vao,
            vbo,
            ebo,
            projection,
        }
    }

    pub fn add_element(&mut self, element: Box<dyn UIElementTrait>) {
        let id = element.get_id();
        self.last_hover_state.push((id, false));
        self.elements.push(element);
    }

    pub fn get_position(&mut self, id: u32) -> Vector2<f32>{//these are uuuuuuh uuuh maybe I should have used a hash map or something other then this
        let elem = self.elements.iter_mut().find(|element| element.get_id() == id)
        .map(|element| element.as_mut()).unwrap();
        elem.get_position()
    }

    pub fn get_size(&mut self, id: u32) -> Vector2<f32> {
        let elem = self.elements.iter_mut().find(|element| element.get_id() == id)
        .map(|element| element.as_mut()).unwrap();
        elem.get_size()
    }

    pub fn get_color(&mut self, id: u32) -> Vector4<f32> {
        let elem = self.elements.iter_mut().find(|element| element.get_id() == id)
        .map(|element| element.as_mut()).unwrap();
        elem.get_color()
    }
    
    pub fn get_texture_id(&mut self, id: u32) -> Option<u32> {
        let elem = self.elements.iter_mut().find(|element| element.get_id() == id)
        .map(|element| element.as_mut()).unwrap();
        elem.get_texture_id()
    }

    pub fn set_texture(&mut self, id: u32, texture_id: u32) {
        let elem = self.elements.iter_mut().find(|element| element.get_id() == id)
        .map(|element| element.as_mut()).unwrap();
        elem.set_texture(texture_id);
    }

    pub fn set_color(&mut self, id: u32, color: Vector4<f32>) {
        let elem = self.elements.iter_mut().find(|element| element.get_id() == id)
        .map(|element| element.as_mut()).unwrap();
        elem.set_color(color);
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

    pub fn update(&mut self, mouse_pos: (f64, f64)) {
        // Clear previous events
        self.event_queue.clear();

        // Check each element for interactions
        for (index, element) in self.elements.iter().enumerate() {
            let id = element.get_id();
            let is_currently_hovered = element.is_hovered(mouse_pos);
            let was_hovered = self.last_hover_state[index].1;

            // Generate hover event
            if is_currently_hovered {
                self.event_queue.push_back(UIEvent::Hover(id));
            }

            // Generate MouseEnter/Exit events
            if is_currently_hovered && !was_hovered {
                self.event_queue.push_back(UIEvent::MouseEnter(id));
            } else if !is_currently_hovered && was_hovered {
                self.event_queue.push_back(UIEvent::MouseExit(id));
            }

            // Update hover state
            self.last_hover_state[index].1 = is_currently_hovered;
        }
    }

    pub fn poll_event(&mut self) -> Option<UIEvent> {
        self.event_queue.pop_front()
    }

    // Add a method to check if there are any events of a specific type for an element
    pub fn has_event_for_element(&self, id: u32, event_type: fn(&UIEvent) -> bool) -> bool {
        self.event_queue.iter().any(|event| {
            match event {
                UIEvent::Hover(elem_id) |
                UIEvent::Click(elem_id) |
                UIEvent::MouseEnter(elem_id) |
                UIEvent::MouseExit(elem_id) => *elem_id == id && event_type(event)
            }
        })
    }

    pub fn is_element_hovered(&self, id: u32) -> bool {
        self.has_event_for_element(id, |event| matches!(event, UIEvent::Hover(_)))
    }
}