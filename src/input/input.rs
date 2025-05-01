use std::collections::{VecDeque, HashSet};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Key {
    W, A, S, D, Q, Right, Left, Up, Down,
    LShift, Lctrl, Space, Escape, Tab, Mouse1,//what is mouse 1 again??? idk lowkey TODO look into this
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CLICKS {
    Left,
    Right,
    Middle,
}

//you might be wondering... why bother with this junk
//so it looks better for the user
pub fn map_glfw_key(glfw_key: glfw::Key) -> Option<Key> {
    match glfw_key {
        glfw::Key::W => Some(Key::W),
        glfw::Key::A => Some(Key::A),
        glfw::Key::S => Some(Key::S),
        glfw::Key::D => Some(Key::D),
        glfw::Key::Q => Some(Key::Q),
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
    ScrollWheel(f64, f64),
}

pub struct InputSystem {
    event_queue: VecDeque<InputEvent>,
    pressed_keys: HashSet<Key>,
    pressed_mouse_buttons: HashSet<CLICKS>,
    mouse_position: (f64, f64),
    scroll_offset: (f64, f64),
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_position: (0.0, 0.0),
            scroll_offset: (0.0, 0.0),
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
            InputEvent::ScrollWheel(x_offset, y_offset) => {
                self.scroll_offset = (x_offset, y_offset);
            }
            // _ => {} ig just take it out bro
        }
        self.event_queue.push_back(event);
    }

    // pub fn queue_event(&mut self, event: InputEvent) {
    //     if let InputEvent::KeyPressed(ref key) = event {
    //         if self.pressed_keys.insert(key.clone()) {
    //             self.event_queue.push_back(event);
    //         }
    //     } else if let InputEvent::KeyReleased(ref key) = event {
    //         if self.pressed_keys.remove(key) {
    //             self.event_queue.push_back(event);
    //         }
    //     } else {
    //         self.event_queue.push_back(event);
    //     }
    // }

    pub fn get_event_queue(&mut self) -> &mut VecDeque<InputEvent> {
        &mut self.event_queue
    }

    pub fn is_key_pressed(&self, key: &Key) -> bool {//fun fact I could have contain borrow and key not borrow so less lines for user, prolly will not though
        self.pressed_keys.contains(key)
    }

    pub fn is_mouse_button_pressed(&self, button: &CLICKS) -> bool {
        self.pressed_mouse_buttons.contains(button)
    }

    pub fn is_mouse_button_just_pressed(&self, button: &CLICKS) -> bool {
        self.event_queue.iter().any(|event| matches!(event, InputEvent::MouseButtonPressed(b) if b == button))
    }

    pub fn is_mouse_button_released(&self, button: &CLICKS) -> bool {
        self.event_queue.iter().any(|event| matches!(event, InputEvent::MouseButtonReleased(b) if b == button))
    }

    pub fn is_mouse_button_held(&self, button: &CLICKS) -> bool {
        self.pressed_mouse_buttons.contains(button)
    }

    pub fn update_mouse_position(&mut self, new_position: (f64, f64)) -> (f64, f64) {
        let (prev_x, prev_y) = self.mouse_position;
        self.mouse_position = new_position;
        (new_position.0 - prev_x, new_position.1 - prev_y) // Return the delta
    }

    pub fn get_mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    //le scroll wheel stuff
    pub fn get_scroll_offset(&self) -> (f64, f64) {
        self.scroll_offset
    }
    
    pub fn has_scrolled(&self) -> bool {
        self.event_queue.iter().any(|event| matches!(event, InputEvent::ScrollWheel(_, _)))
    }
    
    pub fn get_scroll_y(&self) -> f64 {
        self.scroll_offset.1
    }
    
    pub fn get_scroll_x(&self) -> f64 {
        self.scroll_offset.0
    }
    
    pub fn clear_scroll_offset(&mut self) {
        self.scroll_offset = (0.0, 0.0);
    }
}

//todo add this type of shit back
// if let Some(wheel_delta) = input_system.get_mouse_wheel_delta() {
//     if matches!(camera.mode, CameraMode::ThirdPerson) {
//         camera.adjust_third_person_distance(-wheel_delta * 1.0);
//     }
// }

// fn process_events(window: &mut glfw::Window, 
//     events: &Receiver<(f64, WindowEvent)>,
//     input_system: &mut InputSystem) {
// for (_, event) in glfw::flush_messages(events) {
// match event {
// WindowEvent::Key(key, _, Action::Press, _) => {
//    if let Some(mapped_key) = map_glfw_key(key) {
//        input_system.queue_event(InputEvent::KeyPressed(mapped_key));
//    }
// },
// WindowEvent::Key(key, _, Action::Release, _) => {
//    if let Some(mapped_key) = map_glfw_key(key) {
//        input_system.queue_event(InputEvent::KeyReleased(mapped_key));
//    }
// },
// WindowEvent::MouseButton(button, Action::Press, _) => {
//    if let Some(mapped_button) = map_glfw_mousebutton(button) {
//        input_system.queue_event(InputEvent::MouseButtonPressed(mapped_button));
//    }
// },
// WindowEvent::MouseButton(button, Action::Release, _) => {
//    if let Some(mapped_button) = map_glfw_mousebutton(button) {
//        input_system.queue_event(InputEvent::MouseButtonReleased(mapped_button));
//    }
// },
// WindowEvent::CursorPos(x, y) => {
//    input_system.update_mouse_position((x, y));
// },
// WindowEvent::Scroll(x_offset, y_offset) => {
//    // Handle scroll wheel events
//    input_system.queue_event(InputEvent::ScrollWheel(x_offset, y_offset));
// },
// _ => {},
// }

