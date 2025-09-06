use cgmath::{Vector2, Vector4};

use crate::input::input::{InputSystem, CLICKS};

//lol these should return like references bruh
pub trait UIElementTrait {
    fn is_hovered(&self, mouse_pos: (f64, f64)) -> bool;
    fn get_id(&self) -> u32;
    fn get_position(&self) -> Vector2<f32>;
    fn get_size(&self) -> Vector2<f32>;
    fn get_texture_id(&self) -> Option<u32>;
    fn get_color(&self) -> Vector4<f32>;
    fn set_texture(&mut self, texture_id: u32);
    fn set_color(&mut self, color: Vector4<f32>); 
    fn set_position(&mut self, position: Vector2<f32>);
    fn set_size(&mut self, size: Vector2<f32>);
    fn accept(&mut self, visitor: &mut dyn UIElementVisitor);
    fn is_draggable(&self) -> bool {//guys I need to do this more like... wtf am i stupid
        false
    }
}

// impl UIElementTrait {
//     pub fn is_draggable(&self) -> bool {
//         false
//     }
// }
// Define the Visitor trait
pub trait UIElementVisitor {
    fn visit_button(&mut self, button: &mut Button, is_clicked: bool);
    fn visit_slider(&mut self, slider: &mut Slider);
    fn visit_text(&mut self, text: &mut UIText);
}

// A basic UIElement struct for shared properties
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
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            texture_id: None,//I really hate how texture is handled rn
            //it check for presence in the vertex shaderr
            //completely overrides color which sucks befcause like
            //maybe I just want it to change over color
            //a lot of things just dont have texture so its useless there
            //again it checks for texture using if in the vertex shader and just over rides color
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

    fn get_id(&self) -> u32 {//lol these need to be refrences later
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
    fn set_position(&mut self, position: Vector2<f32>) {
        self.position = position;
    }

    fn set_size(&mut self, size: Vector2<f32>) {
        self.size = size;
    }

    fn accept(&mut self, _visitor: &mut dyn UIElementVisitor) {
        // Base UIElements don't need special visitor logic
    }
}

pub struct UIDraggable {
    base: UIElement,
    is_pressed: bool,
}

impl UIDraggable {
    pub fn new(id: u32, position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            base: UIElement::new(id, position, size),
            is_pressed: false,//what this does nothing lol how should i go :idkemoji: ok so do I change this with a) the event que or b) my own is clicked class...idkdk
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed.clone()
    }
}

impl UIElementTrait for UIDraggable {
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
    fn set_position(&mut self, position: Vector2<f32>) {
        print!("bleh");
        self.base.set_position(position);
    }

    fn set_size(&mut self, size: Vector2<f32>) {
        self.base.set_size(size);
    }

    fn accept(&mut self, visitor: &mut dyn UIElementVisitor) {
        print!("accpeted");
        //visitor.visit_button(self, true);//this should be changed to a draggable visitor
    }

    fn is_draggable(&self) -> bool {
        true
    }
}

pub struct Button {
    base: UIElement,
    is_pressed: bool,
}

impl Button {
    pub fn new(id: u32, position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self {
            base: UIElement::new(id, position, size),
            is_pressed: false,//what this does nothing lol how should i go :idkemoji: ok so do I change this with a) the event que or b) my own is clicked class...idkdk
        }
    }

    pub fn is_clicked(&mut self, input_system: &InputSystem) -> bool {
        if input_system.is_mouse_button_pressed(&CLICKS::Left) && self.base.is_hovered(input_system.get_mouse_position()) {
            if !self.is_pressed {
                self.is_pressed = true;
                return true;
            }
        } else {
            self.is_pressed = false;
        }
        false
    }

    // pub fn is_clicked(&self) -> bool {
    //     self.is_pressed
    // }
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
    fn set_position(&mut self, position: Vector2<f32>) {
        self.base.set_position(position);
    }

    fn set_size(&mut self, size: Vector2<f32>) {
        self.base.set_size(size);
    }

    fn accept(&mut self, visitor: &mut dyn UIElementVisitor) {
        visitor.visit_button(self, true);
    }
}

// Define a Slider struct as an example of another element
pub struct Slider {
    base: UIElement,
    value: f32,
    min: f32,
    max: f32,
}

impl Slider {
    pub fn new(id: u32, position: Vector2<f32>, size: Vector2<f32>, min: f32, max: f32) -> Self {
        Self {
            base: UIElement::new(id, position, size),
            value: min,
            min,
            max,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }
}

impl UIElementTrait for Slider {
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
    fn set_position(&mut self, position: Vector2<f32>) {
        self.base.set_position(position);
    }

    fn set_size(&mut self, size: Vector2<f32>) {
        self.base.set_size(size);
    }

    fn accept(&mut self, visitor: &mut dyn UIElementVisitor) {
        visitor.visit_slider(self);
    }
}

pub struct Slot {
    position: Vector2<f32>,
    size: Vector2<f32>,
}

impl Slot {
    pub fn new(position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self { position, size }
    }
    
    pub fn is_within(&self, pos: Vector2<f32>) -> bool {
        pos.x >= self.position.x && pos.x <= self.position.x + self.size.x &&
        pos.y >= self.position.y && pos.y <= self.position.y + self.size.y
    }
    
    pub fn get_position(&self) -> Vector2<f32> {
        self.position
    }
}

pub struct UIText {
    base: UIElement,
    text: String,
    font_size: f32,
}

impl UIText {
    pub fn new(id: u32, position: Vector2<f32>, size: Vector2<f32>, text: String, font_size: f32) -> Self {
        Self {
            base: UIElement::new(id, position, size),
            text,
            font_size,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }

    pub fn get_font_size(&self) -> f32 {
        self.font_size
    }
}

impl UIElementTrait for UIText {
    fn is_hovered(&self, mouse_pos: (f64, f64)) -> bool {
        self.base.is_hovered(mouse_pos)
    }
    fn get_id(&self) -> u32 { self.base.get_id() }
    fn get_position(&self) -> Vector2<f32> { self.base.get_position() }
    fn get_size(&self) -> Vector2<f32> { self.base.get_size() }
    fn get_color(&self) -> Vector4<f32> { self.base.get_color() }
    fn get_texture_id(&self) -> Option<u32> { None }
    fn set_texture(&mut self, _texture_id: u32) {}
    fn set_color(&mut self, color: Vector4<f32>) { self.base.set_color(color); }
    fn set_position(&mut self, position: Vector2<f32>) { self.base.set_position(position); }
    fn set_size(&mut self, size: Vector2<f32>) { self.base.set_size(size); }
    fn accept(&mut self, visitor: &mut dyn UIElementVisitor) {
        visitor.visit_text(self);
    }
}
