use cgmath::{Matrix4, Vector2};
use gl::types::GLfloat;
use std::{mem, ptr};
use super::ui_element::*;
use gl::types::GLsizei;
use crate::{graphics::gl_wrapper::*, user_interface::ui_manager};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum UIEvent {
    Hover(u32),
    Click(u32),
    MouseEnter(u32),
    MouseExit(u32),
    DragStart(u32),
    Dragging(u32, Vector2<f32>),
    DragEnd(u32),
}

pub struct UIManager {
    elements: Vec<Box<dyn UIElementTrait>>,
    event_queue: VecDeque<UIEvent>,
    last_hover_state: Vec<(u32, bool)>,
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    projection: Matrix4<f32>,
    vertex_cache: Vec<(u32, Vec<f32>)>,
    index_cache: Vec<(u32, Vec<i32>)>,
    drag_state: DragState,
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
            vertex_cache: Vec::new(),
            index_cache: Vec::new(),
            drag_state: DragState::new(),//hmm so dragging is not always needed so this might be extra
        }
    }
    
    pub fn add_element(&mut self, element: Box<dyn UIElementTrait>) {
        let id = element.get_id();
        self.elements.push(element);
        self.last_hover_state.push((id, false));
    }
    
    pub fn update(&mut self, mouse_pos: (f64, f64)) {
        self.event_queue.clear();
        let mouse_vec = Vector2::new(mouse_pos.0 as f32, mouse_pos.1 as f32);
        
        for (index, element) in self.elements.iter().enumerate() {
            let id = element.get_id();
            let is_hovered = element.is_hovered(mouse_pos);
            let was_hovered = self.last_hover_state[index].1;
            
            if is_hovered { self.event_queue.push_back(UIEvent::Hover(id)); }
            if is_hovered && !was_hovered { self.event_queue.push_back(UIEvent::MouseEnter(id)); }
            if !is_hovered && was_hovered { self.event_queue.push_back(UIEvent::MouseExit(id)); }
            
            self.last_hover_state[index].1 = is_hovered;
        }
    }
    
    //broooo I hate this function I want drag to like not be in here but I get it like this we should have good dragging
    // pub fn update_drag(&mut self, drag_state: &mut DragState, mouse_pos: (f64, f64)) {
    //     let mouse_vec = Vector2::new(mouse_pos.0 as f32, mouse_pos.1 as f32);
    
    //     // If dragging, update position
    //     if let Some(dragged_id) = drag_state.get_dragging_id() {
    //         if let Some(element) = self.elements.iter_mut().find(|e| e.get_id() == dragged_id) {
    //             element.set_position(mouse_vec - drag_state.offset);
    //             self.event_queue.push_back(UIEvent::Dragging(dragged_id, element.get_position()));
    //         }
    //         return; // Prevent unnecessary checks
    //     }
    
    //     // Check for a draggable element and start dragging
    //     for element in &mut self.elements {
    //         if element.is_hovered(mouse_pos) && element.is_draggable() {
    //             drag_state.start_drag(element.get_id(), mouse_vec - element.get_position());
    //             self.event_queue.push_back(UIEvent::DragStart(element.get_id()));
    //             break;
    //         }
    //     }
    // }

    pub fn start_drag(&mut self, mouse_pos: (f64, f64)) {
        for element in &mut self.elements {
            if element.is_hovered(mouse_pos) && element.is_draggable() {
                let offset = Vector2::new(mouse_pos.0 as f32, mouse_pos.1 as f32) - element.get_position();
                self.drag_state.start_drag(element.get_id(), offset);
                self.event_queue.push_back(UIEvent::DragStart(element.get_id()));
                break;
            }
        }
    }
    
    pub fn update_dragging(&mut self, mouse_pos: (f64, f64)) {
        if let Some(id) = self.drag_state.get_dragging_id() {
            if let Some(element) = self.elements.iter_mut().find(|e| e.get_id() == id) {
                element.set_position(Vector2::new(mouse_pos.0 as f32, mouse_pos.1 as f32) - self.drag_state.offset);
                self.event_queue.push_back(UIEvent::Dragging(id, element.get_position()));
            }
        }
    }

    pub fn end_drag(&mut self) {
        if let Some(id) = self.drag_state.get_dragging_id() {
            self.event_queue.push_back(UIEvent::DragEnd(id));
        }
        self.drag_state.end_drag();
    }
    
    
    pub fn has_event_for_element(&self, id: u32, event_type: fn(&UIEvent) -> bool) -> bool {
        self.event_queue.iter().any(|event| matches!(event, UIEvent::Hover(e) | UIEvent::Click(e) | UIEvent::MouseEnter(e) | UIEvent::MouseExit(e) | UIEvent::DragStart(e) | UIEvent::Dragging(e, _) | UIEvent::DragEnd(e) if *e == id && event_type(event)))
    }
    
    pub fn cache_vertices(&mut self) {
        self.vertex_cache.clear();
        self.index_cache.clear();
        for element in &self.elements {
            let id = element.get_id();
            let vertices = vec![
                element.get_position().x, element.get_position().y + element.get_size().y, 0.0,  0.0, 1.0,
                element.get_position().x + element.get_size().x, element.get_position().y + element.get_size().y, 0.0,  1.0, 1.0,
                element.get_position().x + element.get_size().x, element.get_position().y, 0.0,  1.0, 0.0,
                element.get_position().x, element.get_position().y, 0.0,  0.0, 0.0,
            ];
            let indices = vec![0, 1, 2, 0, 2, 3];
            self.vertex_cache.push((id, vertices));
            self.index_cache.push((id, indices));
        }
    }

    pub fn visit_element(&mut self, id: u32, visitor: &mut dyn UIElementVisitor) {
        if let Some(element) = self.elements.iter_mut().find(|e| e.get_id() == id) {
            element.accept(visitor);
        }
    }

    pub fn visit_all(&mut self, visitor: &mut dyn UIElementVisitor) {
        for element in self.elements.iter_mut() {
            element.accept(visitor);
        }
    }

    pub fn render(&self, shader: &ShaderProgram) {
        shader.bind();//do i just do in init? no bc in mat is norm done?
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

    pub fn poll_event(&mut self) -> Option<UIEvent> {
        self.event_queue.pop_front()
    }

    pub fn is_element_hovered(&self, id: u32) -> bool {
        self.has_event_for_element(id, |event| matches!(event, UIEvent::Hover(_)))
    }

    pub fn get_elements(&self) -> &Vec<Box<dyn UIElementTrait>> {
        &self.elements
    }

    pub fn get_projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    // pub fn handle_event(&mut self, event: &UIEvent) {
    //     match event {
    //         UIEvent::Click(id) => {
    //             if let Some(element) = self.elements.iter_mut().find(|e| e.get_id() == *id) {
    //                 element.accept(&mut ClickVisitor::new());
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}

pub struct DragState {
    dragging_id: Option<u32>,
    offset: Vector2<f32>,
}

impl DragState {
    pub fn new() -> Self {
        Self {
            dragging_id: None,
            offset: Vector2::new(0.0, 0.0),
        }
    }

    pub fn start_drag(&mut self, id: u32, offset: Vector2<f32>) {
        self.dragging_id = Some(id);
        self.offset = offset;
    }

    pub fn end_drag(&mut self) {
        self.dragging_id = None;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging_id.is_some()
    }

    pub fn get_dragging_id(&self) -> Option<u32> {
        self.dragging_id
    }
}