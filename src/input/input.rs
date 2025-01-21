use std::collections::{VecDeque, HashSet};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Key {
    W, A, S, D, Right, Left, Up, Down,
    LShift, Lctrl, Space, Escape, Tab, Mouse1,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CLICKS {
    Left,
    Right,
    Middle,
}

pub fn map_glfw_key(glfw_key: glfw::Key) -> Option<Key> {
    match glfw_key {
        glfw::Key::W => Some(Key::W),
        glfw::Key::A => Some(Key::A),
        glfw::Key::S => Some(Key::S),
        glfw::Key::D => Some(Key::D),
        glfw::Key::Right => Some(Key::Right),
        glfw::Key::Left => Some(Key::Left),
        glfw::Key::Up => Some(Key::Up),
        glfw::Key::Down => Some(Key::Down),
        glfw::Key::LeftShift => Some(Key::LShift),
        glfw::Key::LeftControl => Some(Key::Lctrl),
        glfw::Key::Tab => Some(Key::Tab),
        glfw::Key::Space => Some(Key::Space),
        glfw::Key::Escape => Some(Key::Escape),
        _ => None,
    }
}

pub fn map_glfw_mousebutton(button: glfw::MouseButton) -> Option<CLICKS> {
    match button {
        glfw::MouseButton::Button1 => Some(CLICKS::Left),
        glfw::MouseButton::Button2 => Some(CLICKS::Right),
        glfw::MouseButton::Button3 => Some(CLICKS::Middle),
        _ => None,
    }
}

#[derive(Debug)]
pub enum InputEvent {
    KeyPressed(Key),
    KeyReleased(Key),
    MouseButtonPressed(CLICKS),
    MouseButtonReleased(CLICKS),
}

pub struct InputSystem {
    event_queue: VecDeque<InputEvent>,
    pressed_keys: HashSet<Key>,
    pressed_mouse_buttons: HashSet<CLICKS>,
    mouse_position: (f64, f64),
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_position: (0.0, 0.0),
        }
    }

    pub fn queue_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(ref key) => {
                self.pressed_keys.insert(key.clone());//this is bad
            }
            InputEvent::KeyReleased(ref key) => {
                self.pressed_keys.remove(key);
            }
            InputEvent::MouseButtonPressed(ref button) => {
                self.pressed_mouse_buttons.insert(button.clone());
            }
            InputEvent::MouseButtonReleased(ref button) => {
                self.pressed_mouse_buttons.remove(&button);
            }
            _ => {}
        }
        self.event_queue.push_back(event);
    }

    pub fn get_event_queue(&mut self) -> &mut VecDeque<InputEvent> {
        &mut self.event_queue
    }

    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.contains(key)
    }

    pub fn is_mouse_button_pressed(&self, button: CLICKS) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    pub fn update_mouse_position(&mut self, new_position: (f64, f64)) -> (f64, f64) {
        let (prev_x, prev_y) = self.mouse_position;
        self.mouse_position = new_position;
        (new_position.0 - prev_x, new_position.1 - prev_y) // Return the delta
    }

    pub fn get_mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }
}