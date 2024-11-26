use std::collections::{VecDeque, HashSet};

//can maybe make this all less juncky with like a struct that has everything buy idrc ig
#[derive(Debug, PartialEq, Eq, Hash, Clone)]//erm this feels like its way to much stuff idk man
pub enum Key {
    W,
    A,
    S,
    D,
    Right,
    Left,
    Up,
    Down,
    LShift,
    Space,
    Escape,
    // TODO: need to make this erm like editable idk what the correct word is... not editable from here thouhg ytkyk 
}

#[derive(Debug)]
pub enum InputEvent {
    KeyPressed(Key),
    KeyReleased(Key),
    //MouseMoved(f64, f64), remove all the juck TODO
    MouseButtonPressed(u8),//erm u8 girl 
    MouseButtonReleased(u8),
    // MouseButtonPressed(glfw::MouseButton),//might use this so I left it you can see below where it can be changed 
    // MouseButtonReleased(glfw::MouseButton),
}

//this is to make it a bit more abstract and to keep glfw out of da mains,erm might be ass but who cares ig rn
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
        glfw::Key::Space => Some(Key::Space),
        glfw::Key::Escape => Some(Key::Escape),
        _ => None,
    }
}


//could also pass like a camera delta but idk just seems like its doing too much
pub struct InputSystem {
    event_queue: VecDeque<InputEvent>,
    pressed_keys: HashSet<Key>,
    mouse_position: (f64, f64),
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            pressed_keys: HashSet::new(),
            mouse_position: (0.0, 0.0),
        }
    }

    pub fn queue_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(ref key) => {
                self.pressed_keys.insert(key.clone());
            }
            InputEvent::KeyReleased(ref key) => {
                self.pressed_keys.remove(key);
            }
            _ => {}
        }
        self.event_queue.push_back(event);
    }
    
    pub fn get_event_queue(&mut self) -> &mut VecDeque<InputEvent> {
        &mut self.event_queue
    }

    //a check for like holding
    pub fn is_key_pressed(&self, key: &Key) -> bool {
        self.pressed_keys.contains(key)
    }

    //ok so idk if I like how the mouse is done I might just keep it completely seperate from the rest of the inputs the mouse polling and do it differently
    //I think the way it could be done is just to use the window get mouse position and use its delta to right away
    //without going through the input system change the camera direction
    //I am doing this now thought bc I think I would eventually just end up writting some mouse wrapper anyway
    //and so If I do that later I will at least have this now to like work with
    //also idk 
    pub fn update_mouse_position(&mut self, new_position: (f64, f64)) -> (f64, f64) {
        let (prev_x, prev_y) = self.mouse_position;
        self.mouse_position = new_position;
        (new_position.0 - prev_x, new_position.1 - prev_y) // Return the delta
    }
}





//there is a better way to do this ngl ngl
//     pub fn process_events(&mut self, glfw_events: &mut glfw::FlushedMessages<(f64, glfw::WindowEvent)>) {//erm like idk why this just miraculously worked
//         for (_, event) in glfw_events{
//             match event {
//                 glfw::WindowEvent::Key(glfw_key, _, action, _) => {
//                     if let Some(key) = map_glfw_key(glfw_key) {
//                         match action {//wow it can activate on press or release LIT!!!
//                             glfw::Action::Press => self.event_queue.push(InputEvent::KeyPressed(key)),
//                             glfw::Action::Release => self.event_queue.push(InputEvent::KeyReleased(key)),
//                             _ => {}
//                         }
//                     }
//                 }//lol this is also in window soz
//                 glfw::WindowEvent::CursorPos(x, y) => {
//                     self.event_queue.push(InputEvent::MouseMoved(x, y))
//                 }//cool
//                 glfw::WindowEvent::MouseButton(button, action, _) => {
//                     let button_index = button as u8; // customize this as needed ie might just use glfw::mouse or whatever
//                     match action {
//                         glfw::Action::Press => {
//                             self.event_queue.push(InputEvent::MouseButtonPressed(button_index))
//                         }
//                         glfw::Action::Release => {
//                             self.event_queue.push(InputEvent::MouseButtonReleased(button_index))
//                         }
//                         _ => {}
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }

//     pub fn get_event_queue(&mut self) -> &mut EventQueue {
//         &mut self.event_queue
//     }
// }

//can just use input que no need for antoher wrapper 
// pub struct EventQueue {
//     events: VecDeque<InputEvent>,
// }

// impl EventQueue {
//     pub fn new() -> Self {
//         EventQueue {
//             events: VecDeque::new(),
//         }
//     }

//     pub fn push(&mut self, event: InputEvent) {
//         self.events.push_back(event);
//     }

//     pub fn pop(&mut self) -> Option<InputEvent> {
//         self.events.pop_front()
//     }
// }