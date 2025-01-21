use cgmath::{Vector2, Vector4};
use super::ui_element::UIElement;
use crate::graphics::window::Window;

// Extend UIElement for buttons
pub struct UIButton {
    element: UIElement,
    on_click: Box<dyn Fn()>,
}

impl UIButton {
    pub fn new(
        position: Vector2<f32>,
        size: Vector2<f32>,
        color: Vector4<f32>,
        on_click: Box<dyn Fn()>,
    ) -> Self {
        Self {
            element: UIElement::new(position, size, color),
            on_click,
        }
    }

    pub fn is_mouse_over(&self, mouse_pos: Vector2<f32>) -> bool {
        let pos = self.element.get_position();
        let size = self.element.get_size();
        mouse_pos.x >= pos.x
            && mouse_pos.x <= pos.x + size.x
            && mouse_pos.y >= pos.y
            && mouse_pos.y <= pos.y + size.y
    }

    pub fn trigger_click(&self) {
        (self.on_click)();
    }

    pub fn get_element(&self) -> &UIElement {
        &self.element
    }
}