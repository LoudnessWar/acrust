// UI Components there wes never the word your in this file never has been no one has ever said your not a word not a word
use cgmath::{Vector2, Vector3, Vector4};
use crate::ecs::world::{Component, ComponentStorage};
use crate::user_interface::text_render::{self, TextRenderer};
use crate::input::input::{InputSystem, InputEvent, Key, CLICKS};

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
    pub background_color: Vector4<f32>,  // Renamed from 'color'
    pub text_color: Vector4<f32>,        // New field for text color
    pub texture_id: Option<u32>,
    pub visible: bool,
    pub render_background: bool,         // New field to control background rendering
}

impl UIStyle {
    pub fn new() -> Self {
        Self {
            background_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            text_color: Vector4::new(0.0, 0.0, 0.0, 1.0),  // Default to black text
            texture_id: None,
            visible: true,
            render_background: true,  // Default to rendering background
        }
    }
    
    pub fn with_color(mut self, color: Vector4<f32>) -> Self {//this naming is a bit bad but i think its more intuitive if that makes sense instead of making it know that there is a sepertaion between text and background color like they will figure it out once and then not have to think about it everytime they create a ui element with
        //a background color
        self.background_color = color;
        self
    }
    
    pub fn with_text_color(mut self, color: Vector4<f32>) -> Self {
        self.text_color = color;
        self
    }
    
    pub fn with_texture(mut self, texture_id: u32) -> Self {
        self.texture_id = Some(texture_id);
        self
    }
    
    pub fn text_only(mut self) -> Self {
        self.render_background = false;
        self
    }
    
    // Convenience method for buttons will probably make more of these later
    pub fn button_style(background: Vector4<f32>, text: Vector4<f32>) -> Self {
        Self::new()
            .with_color(background)
            .with_text_color(text)
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

    pub fn grid(cols: u32, spacing: f32) -> Self {
        Self {
            layout_type: LayoutType::Grid { cols, spacing },
            padding: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
    
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Vector4::new(padding, padding, padding, padding);
        self
    }
}

#[derive(Debug, Clone)]
pub struct UIZIndex {
    pub z_index: i32, // Higher values render on top
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

#[derive(Debug, Clone)]
pub struct UITextInput {
    pub text: String,
    pub cursor_position: usize,
    pub is_focused: bool,
    pub placeholder: String,
    pub max_length: Option<usize>,
    pub cursor_visible: bool,
    pub cursor_timer: f32,  // For blinking cursor
}

impl UITextInput {
    pub fn new(placeholder: String) -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            is_focused: false,
            placeholder,
            max_length: None,
            cursor_visible: true,
            cursor_timer: 0.0,
        }
    }
    
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
    
    pub fn insert_char(&mut self, c: char) {
        if let Some(max_len) = self.max_length {
            if self.text.len() >= max_len {
                return;
            }
        }
        
        self.text.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.reset_cursor_blink();
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
        }
        self.reset_cursor_blink();
    }
    
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
        self.reset_cursor_blink();
    }
    
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
        self.reset_cursor_blink();
    }
    
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
        self.reset_cursor_blink();
    }
    
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
        self.reset_cursor_blink();
    }
    
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
        self.reset_cursor_blink();
    }
    
    pub fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_timer = 0.0;
    }
    
    pub fn update_cursor_blink(&mut self, delta_time: f32) {
        self.cursor_timer += delta_time;
        if self.cursor_timer >= 0.5 {  // Blink every 500ms
            self.cursor_visible = !self.cursor_visible;
            self.cursor_timer = 0.0;
        }
    }
    
    pub fn get_display_text(&self) -> &str {
        if self.text.is_empty() && !self.is_focused {
            &self.placeholder
        } else {
            &self.text
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

// UI System
pub struct UISystem {
    transforms: ComponentStorage<UITransform>,
    pub styles: ComponentStorage<UIStyle>, //todo lol bad fix
    parents: ComponentStorage<UIParent>,
    children: ComponentStorage<UIChildren>,
    pub layouts: ComponentStorage<UILayout>,
    buttons: ComponentStorage<UIButton>,
    pub texts: ComponentStorage<UIText>,
    z_indices: ComponentStorage<UIZIndex>,
    
    // UI-specific state
    pub layout_dirty: bool,
    hover_state: std::collections::HashMap<u32, bool>,
    
    // OpenGL resources
    vao: crate::graphics::gl_wrapper::Vao,
    vbo: crate::graphics::gl_wrapper::BufferObject,
    ebo: crate::graphics::gl_wrapper::BufferObject,
    projection: cgmath::Matrix4<f32>,

    pub text_renderer: TextRenderer,
}

impl UISystem {
    pub fn new(screen_width: f32, screen_height: f32, text_renderer: TextRenderer) -> Self {
        // Set up OpenGL resources
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
            z_indices: ComponentStorage::new(),
            layout_dirty: false,
            hover_state: std::collections::HashMap::new(),
            vao,
            vbo,
            ebo,
            projection,
            text_renderer,
        }
    }
    
    // Component management
    pub fn add_transform(&mut self, entity_id: u32, transform: UITransform) {
        self.transforms.insert(entity_id, transform);
        self.layout_dirty = true;
    }

    pub fn add_z_index(&mut self, entity_id: u32, z_index: i32) {
        self.z_indices.insert(entity_id, UIZIndex { z_index });
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
    
    pub fn get_transform(&self, entity_id: u32) -> Option<&UITransform> {
        self.transforms.get(entity_id)
    }
    
    pub fn get_transform_mut(&mut self, entity_id: u32) -> Option<&mut UITransform> {
        self.transforms.get_mut(entity_id)
    }

    pub fn update_text(&mut self, entity_id: u32, new_text: String) {
        if let Some(text_component) = self.texts.get_mut(entity_id) {
            text_component.text = new_text;
        }
    }
    
    pub fn get_text_dimensions(&self, entity_id: u32) -> Option<(f32, f32)> {
        if let Some(text_component) = self.texts.get(entity_id) {
            let scale = text_component.font_size / 24.0;
            let (width, height) = self.text_renderer.measure_text(&text_component.text, scale);
            Some((width, height))
        } else {
            None
        }
    }
    
    pub fn auto_size_text_elements(&mut self) {
        let text_entities: Vec<u32> = self.texts.iter().map(|(id, _)| *id).collect();
        
        for entity_id in text_entities {
            if let Some((width, height)) = self.get_text_dimensions(entity_id) {
                if let Some(transform) = self.transforms.get_mut(entity_id) {
                    transform.size = Vector2::new(width, height);
                }
            }
        }
        
        self.layout_dirty = true;
    }

    
    // Layout system
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
            // Use the root's position as the starting offset
            let root_pos = self.transforms.get(root_id).map(|t| t.position).unwrap_or(Vector2::new(0.0, 0.0));
            self.calculate_hierarchy_positions(root_id, root_pos);
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
    
    // Input handling
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
    
    // ok so basically this is not fun at all, also just real quick i got a new computer and now i have like the co piolot shit on my vs code
    //and its like fine or whatever it gives useful stuff like 1/10 tries but it is so terrible when you are writing commetents
    //like it had no idea that here i was going to complain about the fact the the text and UI elements use a different shader and so i need to render the UI elements and then the text and make sure that they are index properly so that this
    //works because random junk like changes the hash so that they are rendered out of order and so you cannot see the text, especially when they are both 0. TBH  TBH TBH it should be deterministic it should be
    //todo maybe make it deteministic
    //but honestly it doesnt really matter if you want it to be a certain way use the z index
    //like i remember using godot that like the z sorting in that is ass imo because it is based off the parent child relationship and so it made procedurally generetad
    //sprites really hard to get the z plane on properly so all i really want is for the text of a button to be one above the actual button element. This can create issues if you are like
    //putting a thing exactly one above a button but in that case 1. why are you placing an element on top of a button, 2. it doesnt really matter all that much like just move everything else up one layer on the z index then it will no longer be a problem.
    //BITCH

    //fuch this has nothing to do with the borrow checker i know your stalking me copilot because I have complained about the borrow checker in so many comments before but the comments are my space like stay out!!!
    pub fn render(&self, shader: &crate::graphics::gl_wrapper::ShaderProgram) {
        let mut render_list: Vec<(u32, i32)> = Vec::new();
        
        for (entity_id, transform) in self.transforms.iter() {
            if let Some(style) = self.styles.get(*entity_id) {
                if style.visible {
                    let z_index = self.z_indices.get(*entity_id)
                        .map(|z| z.z_index)
                        .unwrap_or(0);
                    
                    render_list.push((*entity_id, z_index));
                }
            }
        }

        render_list.sort_by_key(|&(_, z)| z);

        shader.bind();
        shader.set_matrix4fv_uniform("projection", &self.projection);
        self.vao.bind();
        
        // First pass: Render all background elements (only if render_background is true)
        for (entity_id, _) in &render_list {
            let transform = self.transforms.get(*entity_id).unwrap();
            let style = self.styles.get(*entity_id).unwrap();
            
            // Only render background if the flag is set
            if style.render_background {
                self.render_ui_element(*entity_id, transform, style, shader);
            }
        }

        // Second pass: Render all text elements on top
        for (entity_id, _) in &render_list {
            if self.texts.contains(*entity_id) {
                self.render_text_element(*entity_id);
            }
        }
    }

    // Updated render_ui_element to use background_color
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
            shader.set_uniform4f("color", &style.background_color); // Use background_color instead of color
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

    // Updated render_text_element to use text_color
    fn render_text_element(&self, entity_id: u32) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::DEPTH_TEST);
        }
        
        if let Some(text_component) = self.texts.get(entity_id) {
            if let Some(transform) = self.transforms.get(entity_id) {
                if let Some(style) = self.styles.get(entity_id) {
                    if !style.visible {
                        return;
                    }
                    
                    // Use text_color instead of background color
                    let text_color = Vector3::new(style.text_color.x, style.text_color.y, style.text_color.z);
                    let scale = text_component.font_size / 24.0;
                    
                    self.text_renderer.render_text(
                        &text_component.text,
                        transform.position.x,
                        transform.position.y,
                        scale,
                        text_color,
                        &self.projection,
                    );
                }
            }
        }
        
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }
    }
    
    // fn render_ui_element(&self, entity_id: u32, transform: &UITransform, style: &UIStyle, shader: &crate::graphics::gl_wrapper::ShaderProgram) { //todo lol this needs to be checked todo
    //     let vertices: Vec<f32> = vec![
    //         transform.position.x, transform.position.y + transform.size.y, 0.0,  0.0, 1.0,
    //         transform.position.x + transform.size.x, transform.position.y + transform.size.y, 0.0,  1.0, 1.0,
    //         transform.position.x + transform.size.x, transform.position.y, 0.0,  1.0, 0.0,
    //         transform.position.x, transform.position.y, 0.0,  0.0, 0.0,
    //     ];

    //     let indices: Vec<i32> = vec![0, 1, 2, 0, 2, 3];

    //     self.vbo.bind();
    //     self.vbo.store_f32_data(&vertices);
    //     self.ebo.bind();
    //     self.ebo.store_i32_data(&indices);

    //     if let Some(texture_id) = style.texture_id {
    //         shader.set_uniform1i("useTexture", &1);
    //         unsafe {
    //             gl::ActiveTexture(gl::TEXTURE0);
    //             gl::BindTexture(gl::TEXTURE_2D, texture_id);
    //         }
    //     } else {
    //         shader.set_uniform1i("useTexture", &0);
    //         shader.set_uniform4f("color", &style.color);
    //         //println!("Rendering UI Element {} with color {:?}", entity_id, style.color);
    //     }

    //     unsafe {
    //         gl::DrawElements(
    //             gl::TRIANGLES,
    //             indices.len() as i32,
    //             gl::UNSIGNED_INT,
    //             std::ptr::null(),
    //         );
    //     }
    // }

    // fn render_text_elements(&self) {
    //     unsafe {
    //         gl::Enable(gl::BLEND);
    //         gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    //         gl::Disable(gl::DEPTH_TEST);
    //     }
        
    //     for (entity_id, text_component) in self.texts.iter() {
    //         if let Some(transform) = self.transforms.get(*entity_id) {
    //             if let Some(style) = self.styles.get(*entity_id) {
    //                 if !style.visible {
    //                     continue;
    //                 }
                    
    //                 // Calculate text color from style
    //                 let text_color = Vector3::new(style.color.x, style.color.y, style.color.z);
                    
    //                 // Calculate scale based on font size and transform size
    //                 let scale = text_component.font_size / 24.0; // Assuming 24.0 is base font size
                    
    //                 self.text_renderer.render_text(
    //                     &text_component.text,
    //                     transform.position.x,
    //                     transform.position.y,
    //                     scale,
    //                     text_color,
    //                     &self.projection,
    //                 );
    //             }
    //         }
    //     }
        
    //     unsafe {
    //         gl::Enable(gl::DEPTH_TEST);
    //         gl::Disable(gl::BLEND);
    //     }
    // }
    
    // Query methods
    pub fn is_button_clicked(&self, entity_id: u32) -> bool {
        self.buttons.get(entity_id).map(|b| b.is_clicked).unwrap_or(false)
    }
    
    pub fn is_button_hovered(&self, entity_id: u32) -> bool {
        self.buttons.get(entity_id).map(|b| b.is_hovered).unwrap_or(false)
    }
}