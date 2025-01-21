use cgmath::{Vector2, Vector4};
use crate::input::input::*;

pub trait UIElementTrait {
    //fn render(&self, shader: &ShaderProgram);
    fn is_hovered(&self, mouse_pos: (f64, f64)) -> bool;
    fn get_id(&self) -> u32;
    fn get_position(&self) -> Vector2<f32>;
    fn get_size(&self) -> Vector2<f32>;
    fn get_texture_id(&self) -> Option<u32>;
    fn get_color(&self) -> Vector4<f32>;
    fn set_texture(&mut self, texture_id: u32);
    fn set_color(&mut self, color: Vector4<f32>); 
}

pub struct UIElement {
    id: u32,
    position: Vector2<f32>,
    size: Vector2<f32>,
    color: Vector4<f32>,
    texture_id: Option<u32>,
}

impl UIElement {
    pub fn new(id: u32, position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            id,
            position,
            size,
            color: Vector4::new(1.0,1.0,1.0,1.0),
            texture_id: None,
        }
    }
}

impl UIElementTrait for UIElement {
    fn is_hovered(&self, mouse_pos: (f64, f64)) -> bool {
        let (mouse_x, mouse_y) = mouse_pos;
        mouse_x as f32 >= self.position.x
            && mouse_x as f32 <= self.position.x + self.size.x
            && mouse_y as f32 >= self.position.y
            && mouse_y as f32 <= self.position.y + self.size.y
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_position(&self) -> Vector2<f32>{
        self.position
    }

    fn get_size(&self) -> Vector2<f32> {
        self.size
    }

    fn get_color(&self) -> Vector4<f32> {
        self.color
    }
    
    fn get_texture_id(&self) -> Option<u32> {
        self.texture_id
    }

    fn set_texture(&mut self, texture_id: u32) {
        self.texture_id = Some(texture_id);
    }

    fn set_color(&mut self, color: Vector4<f32>) {
        self.color = color;
    }
}

// Button struct inheriting from UIElement
pub struct Button {
    base: UIElement,
    is_pressed: bool,
}

impl Button {
    pub fn new(id: u32, position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            base: UIElement::new(id, position, size),
            is_pressed: false,
        }
    }

    pub fn is_clicked(&mut self, input_system: &InputSystem) -> bool {
        if input_system.is_mouse_button_pressed(CLICKS::Left) && self.base.is_hovered(input_system.get_mouse_position()) {
            if !self.is_pressed {
                self.is_pressed = true;
                return true;
            }
        } else {
            self.is_pressed = false;
        }
        false
    }
}

impl UIElementTrait for Button {
    fn is_hovered(&self, mouse_pos: (f64, f64)) -> bool {
        self.base.is_hovered(mouse_pos)
    }

    fn get_id(&self) -> u32 {
        self.base.get_id()
    }

    fn get_position(&self) -> Vector2<f32> {
        self.base.get_position()
    }

    fn get_size(&self) -> Vector2<f32> {
        self.base.get_size()
    }

    fn get_color(&self) -> Vector4<f32> {
        self.base.get_color()
    }

    fn get_texture_id(&self) -> Option<u32> {
        self.base.get_texture_id()
    }

    fn set_texture(&mut self, texture_id: u32) {
        self.base.set_texture(texture_id);
    }

    fn set_color(&mut self, color: Vector4<f32>) {
        self.base.set_color(color);
    }
}
