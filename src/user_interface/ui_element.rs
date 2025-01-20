use cgmath::{Vector2, Vector4};

pub struct UIElement {
    position: Vector2<f32>, // 2D position (screen space)
    size: Vector2<f32>,     // Width and height
    color: Vector4<f32>,    // RGBA color
    texture_id: Option<u32>, // Optional texture for the element
}

impl UIElement {
    pub fn new(position: Vector2<f32>, size: Vector2<f32>, color: Vector4<f32>) -> Self {
        Self {
            position,
            size,
            color,
            texture_id: None,
        }
    }

    pub fn set_texture(&mut self, texture_id: u32) {
        self.texture_id = Some(texture_id);
    }

    pub fn set_color(&mut self, color: Vector4<f32>) {
        self.color = color;
    }

    pub fn get_texture_id(&self) -> Option<u32> {
        self.texture_id
    }

    pub fn get_position(&self) -> Vector2<f32>{
        self.position
    }

    pub fn get_size(&self) -> Vector2<f32> {
        self.size
    }
    
}
