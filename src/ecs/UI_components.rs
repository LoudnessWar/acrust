// UI Components - following your existing pattern
use cgmath::{Vector2, Vector4};
use crate::ecs::world::{Component, ComponentStorage};

// UI-specific components
#[derive(Debug, Clone)]
pub struct UITransform {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
    pub local_position: Vector2<f32>, // For hierarchy
}

impl UITransform {
    pub fn new(position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            position,
            size,
            local_position: Vector2::new(0.0, 0.0),
        }
    }
}

// impl Component for UITransform {}

#[derive(Debug, Clone)]
pub struct UIStyle {
    pub color: Vector4<f32>,
    pub texture_id: Option<u32>,
    pub visible: bool,
}

impl UIStyle {
    pub fn new() -> Self {
        Self {
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            texture_id: None,
            visible: true,
        }
    }
    
    pub fn with_color(mut self, color: Vector4<f32>) -> Self {
        self.color = color;
        self
    }
    
    pub fn with_texture(mut self, texture_id: u32) -> Self {
        self.texture_id = Some(texture_id);
        self
    }
}

// impl Component for UIStyle {}

#[derive(Debug, Clone)]
pub struct UIParent {
    pub parent_id: Option<u32>,
}

// impl Component for UIParent {}

#[derive(Debug, Clone)]
pub struct UIChildren {
    pub children: Vec<u32>,
}

impl UIChildren {
    pub fn new() -> Self {
        Self { children: Vec::new() }
    }
    
    pub fn add_child(&mut self, child_id: u32) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }
    
    pub fn remove_child(&mut self, child_id: u32) {
        self.children.retain(|&id| id != child_id);
    }
}

// impl Component for UIChildren {}

#[derive(Debug, Clone)]
pub enum LayoutType {
    None,
    Vertical { spacing: f32 },
    Horizontal { spacing: f32 },
    Grid { cols: u32, spacing: f32 },
}

#[derive(Debug, Clone)]
pub struct UILayout {
    pub layout_type: LayoutType,
    pub padding: Vector4<f32>, // top, right, bottom, left
}

impl UILayout {
    pub fn vertical(spacing: f32) -> Self {
        Self {
            layout_type: LayoutType::Vertical { spacing },
            padding: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
    
    pub fn horizontal(spacing: f32) -> Self {
        Self {
            layout_type: LayoutType::Horizontal { spacing },
            padding: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
    
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Vector4::new(padding, padding, padding, padding);
        self
    }
}

// impl Component for UILayout {}

#[derive(Debug, Clone)]
pub struct UIButton {
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_clicked: bool,
}

impl UIButton {
    pub fn new() -> Self {
        Self {
            is_hovered: false,
            is_pressed: false,
            is_clicked: false,
        }
    }
}

// impl Component for UIButton {}

#[derive(Debug, Clone)]
pub struct UIText {
    pub text: String,
    pub font_size: f32,
}

impl UIText {
    pub fn new(text: String, font_size: f32) -> Self {
        Self { text, font_size }
    }
}

// impl Component for UIText {}

// UI System - following your existing pattern
pub struct UISystem {
    transforms: ComponentStorage<UITransform>,
    pub styles: ComponentStorage<UIStyle>, //todo lol bad fix
    parents: ComponentStorage<UIParent>,
    children: ComponentStorage<UIChildren>,
    layouts: ComponentStorage<UILayout>,
    buttons: ComponentStorage<UIButton>,
    texts: ComponentStorage<UIText>,
    
    // UI-specific state
    layout_dirty: bool,
    hover_state: std::collections::HashMap<u32, bool>,
    
    // OpenGL resources (like your existing UI manager)
    vao: crate::graphics::gl_wrapper::Vao,
    vbo: crate::graphics::gl_wrapper::BufferObject,
    ebo: crate::graphics::gl_wrapper::BufferObject,
    projection: cgmath::Matrix4<f32>,
}

impl UISystem {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        // Set up OpenGL resources (same as your current UIManager)
        let vao = crate::graphics::gl_wrapper::Vao::new();
        vao.bind();

        let vbo = crate::graphics::gl_wrapper::BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();

        let ebo = crate::graphics::gl_wrapper::BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();

        let stride = 5 * std::mem::size_of::<gl::types::GLfloat>() as gl::types::GLsizei;
        crate::graphics::gl_wrapper::VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null()).enable();
        crate::graphics::gl_wrapper::VertexAttribute::new(
            1, 2, gl::FLOAT, gl::FALSE, stride, 
            (3 * std::mem::size_of::<gl::types::GLfloat>()) as *const _
        ).enable();

        let projection = cgmath::ortho(0.0, screen_width, screen_height, 0.0, -1.0, 1.0);

        Self {
            transforms: ComponentStorage::new(),
            styles: ComponentStorage::new(),
            parents: ComponentStorage::new(),
            children: ComponentStorage::new(),
            layouts: ComponentStorage::new(),
            buttons: ComponentStorage::new(),
            texts: ComponentStorage::new(),
            layout_dirty: false,
            hover_state: std::collections::HashMap::new(),
            vao,
            vbo,
            ebo,
            projection,
        }
    }
    
    // Component management - following your existing pattern
    pub fn add_transform(&mut self, entity_id: u32, transform: UITransform) {
        self.transforms.insert(entity_id, transform);
        self.layout_dirty = true;
    }
    
    pub fn add_style(&mut self, entity_id: u32, style: UIStyle) {
        self.styles.insert(entity_id, style);
    }
    
    pub fn add_parent(&mut self, entity_id: u32, parent_id: Option<u32>) {
        self.parents.insert(entity_id, UIParent { parent_id });
        if let Some(pid) = parent_id {
            // Add to parent's children list
            if !self.children.contains(pid) {
                self.children.insert(pid, UIChildren::new());
            }
            self.children.get_mut(pid).unwrap().add_child(entity_id);
        }
        self.layout_dirty = true;
    }
    
    pub fn add_layout(&mut self, entity_id: u32, layout: UILayout) {
        self.layouts.insert(entity_id, layout);
        self.layout_dirty = true;
    }
    
    pub fn add_button(&mut self, entity_id: u32) {
        self.buttons.insert(entity_id, UIButton::new());
    }
    
    pub fn add_text(&mut self, entity_id: u32, text: String, font_size: f32) {
        self.texts.insert(entity_id, UIText::new(text, font_size));
    }
    
    // Safe getters - following your pattern
    pub fn get_transform(&self, entity_id: u32) -> Option<&UITransform> {
        self.transforms.get(entity_id)
    }
    
    pub fn get_transform_mut(&mut self, entity_id: u32) -> Option<&mut UITransform> {
        self.transforms.get_mut(entity_id)
    }
    
    // Layout system - no borrow conflicts like your movement system
    pub fn update_layout(&mut self) {
        if !self.layout_dirty {
            return;
        }
        
        // Find root elements (no parent)
        let root_entities: Vec<u32> = self.transforms
            .iter()
            .filter(|(id, _)| {
                !self.parents.contains(**id) || 
                self.parents.get(**id).unwrap().parent_id.is_none()
            })
            .map(|(id, _)| *id)
            .collect();
        
        // Process each root hierarchy
        for root_id in root_entities {
            self.calculate_hierarchy_positions(root_id, Vector2::new(0.0, 0.0));
        }
        
        self.layout_dirty = false;
    }
    
    fn calculate_hierarchy_positions(&mut self, entity_id: u32, parent_offset: Vector2<f32>) {
        // Get local position
        let local_pos = if let Some(transform) = self.transforms.get(entity_id) {
            transform.local_position
        } else {
            return;
        };
        
        // Calculate absolute position
        let absolute_pos = parent_offset + local_pos;
        
        // Update transform
        if let Some(transform) = self.transforms.get_mut(entity_id) {
            transform.position = absolute_pos;
        }
        
        // Handle layout for children
        if let Some(children_comp) = self.children.get(entity_id) {
            let children = children_comp.children.clone(); // Clone to avoid borrow conflicts
            
            if let Some(layout) = self.layouts.get(entity_id) {//TODO THIS IS BOOTY
                let layout_clone = layout.clone();
                self.apply_layout_to_children(entity_id, &children, &layout_clone, absolute_pos);
            }
            
            // Recursively update children
            for child_id in children {
                self.calculate_hierarchy_positions(child_id, absolute_pos);
            }
        }
    }
    
    fn apply_layout_to_children(&mut self, _container_id: u32, children: &[u32], layout: &UILayout, container_pos: Vector2<f32>) {
        let content_pos = Vector2::new(
            container_pos.x + layout.padding.w, // left padding
            container_pos.y + layout.padding.x, // top padding
        );
        
        match &layout.layout_type {
            LayoutType::Vertical { spacing } => {
                let mut current_y = content_pos.y;
                for &child_id in children {
                    if let Some(child_transform) = self.transforms.get_mut(child_id) {
                        child_transform.local_position = Vector2::new(0.0, current_y - container_pos.y);
                        current_y += child_transform.size.y + spacing;
                    }
                }
            },
            LayoutType::Horizontal { spacing } => {
                let mut current_x = content_pos.x;
                for &child_id in children {
                    if let Some(child_transform) = self.transforms.get_mut(child_id) {
                        child_transform.local_position = Vector2::new(current_x - container_pos.x, 0.0);
                        current_x += child_transform.size.x + spacing;
                    }
                }
            },
            LayoutType::Grid { cols, spacing } => {
                for (index, &child_id) in children.iter().enumerate() {
                    let col = index as u32 % cols;
                    let row = index as u32 / cols;
                    
                    if let Some(child_transform) = self.transforms.get_mut(child_id) {
                        let x = col as f32 * (child_transform.size.x + spacing);
                        let y = row as f32 * (child_transform.size.y + spacing);
                        child_transform.local_position = Vector2::new(x, y);
                    }
                }
            },
            LayoutType::None => {
                // Keep existing local positions
            }
        }
    }
    
    // Input handling - following your existing patterns
    pub fn update_input(&mut self, mouse_pos: (f64, f64), mouse_down: bool, mouse_clicked: bool) {
        let mouse_vec = Vector2::new(mouse_pos.0 as f32, mouse_pos.1 as f32);
        
        // Update button states - no borrow conflicts
        for (entity_id, button) in self.buttons.iter_mut() {
            let is_hovered = if let Some(transform) = self.transforms.get(*entity_id) {
                mouse_vec.x >= transform.position.x &&
                mouse_vec.x <= transform.position.x + transform.size.x &&
                mouse_vec.y >= transform.position.y &&
                mouse_vec.y <= transform.position.y + transform.size.y
            } else {
                false
            };
            
            let was_hovered = self.hover_state.get(entity_id).copied().unwrap_or(false);
            
            button.is_hovered = is_hovered;
            button.is_pressed = is_hovered && mouse_down;
            button.is_clicked = is_hovered && mouse_clicked && was_hovered;
            
            self.hover_state.insert(*entity_id, is_hovered);
        }
    }
    
    // Rendering - similar to your existing render system
    pub fn render(&self, shader: &crate::graphics::gl_wrapper::ShaderProgram) {
        shader.bind();
        shader.set_matrix4fv_uniform("projection", &self.projection);
        self.vao.bind();
        
        // Render all visible UI elements
        for (entity_id, transform) in self.transforms.iter() {
            if let Some(style) = self.styles.get(*entity_id) {
                if !style.visible {
                    continue;
                }
                
                self.render_ui_element(*entity_id, transform, style, shader);
            }
        }
    }
    
    fn render_ui_element(&self, entity_id: u32, transform: &UITransform, style: &UIStyle, shader: &crate::graphics::gl_wrapper::ShaderProgram) {
        let vertices: Vec<f32> = vec![
            transform.position.x, transform.position.y + transform.size.y, 0.0,  0.0, 1.0,
            transform.position.x + transform.size.x, transform.position.y + transform.size.y, 0.0,  1.0, 1.0,
            transform.position.x + transform.size.x, transform.position.y, 0.0,  1.0, 0.0,
            transform.position.x, transform.position.y, 0.0,  0.0, 0.0,
        ];

        let indices: Vec<i32> = vec![0, 1, 2, 0, 2, 3];

        self.vbo.bind();
        self.vbo.store_f32_data(&vertices);
        self.ebo.bind();
        self.ebo.store_i32_data(&indices);

        if let Some(texture_id) = style.texture_id {
            shader.set_uniform1i("useTexture", &1);
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture_id);
            }
        } else {
            shader.set_uniform1i("useTexture", &0);
            shader.set_uniform4f("color", &style.color);
        }

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
    
    // Query methods - like your existing get methods
    pub fn is_button_clicked(&self, entity_id: u32) -> bool {
        self.buttons.get(entity_id).map(|b| b.is_clicked).unwrap_or(false)
    }
    
    pub fn is_button_hovered(&self, entity_id: u32) -> bool {
        self.buttons.get(entity_id).map(|b| b.is_hovered).unwrap_or(false)
    }
}