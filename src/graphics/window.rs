use glfw::{Action, Context, GlfwReceiver, Key, WindowEvent};
use crate::input::input::{map_glfw_key, map_glfw_mousebutton, InputEvent, InputSystem}; //erm yeah scuffed but I think it will look good when you have to work with it and make sense

use std::time::{Duration, Instant};

//use std::sync::mpsc::Receiver;

// / # Window
// /
// / An abstraction layer for creating a glfw window.
// /
// / ## Example
// / ```
// / let mut window = Window::new(1280, 720, "Window Title");
// / window.init_gl();
// /
// / while !window.should_close() {
// /     window.update;
// / }
// / ```
pub struct Window {
    glfw: glfw::Glfw,
    window_handle: glfw::PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    last_frame_time: Instant,
    target_frame_duration: Duration,
}

impl Window {
    /// Create new window with settings
    pub fn new(width: u32, height: u32, title: &str, target_fps: u8) -> Window {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window!");

        window.set_framebuffer_size_polling(true);
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);//this me thinks is a big slower downers
        window.set_scroll_polling(true);//this as well but it is what it is i mean all games need this so I shouldnt be afreaid of them
        //this one should be toggleable so TODO make the polling of stuff toggleable

        let target_frame_duration = Duration::from_secs_f32(1.0 / target_fps as f32);

        Window {
            glfw,
            window_handle: window,
            events: events,
            last_frame_time: Instant::now(),
            target_frame_duration,
        }
    }

    /// Load gl functions.
    pub fn init_gl(&mut self) {
        self.window_handle.make_current();
        gl::load_with(|s| self.window_handle.get_proc_address(s) as *const _);
    }

    pub fn should_close(&self) -> bool {
        self.window_handle.should_close()
    }

    /// Poll events and swap buffers... i need to fix this all up... update needs to call process input events
    /// TODO make update process input events and merge process_events and process input events or something probably... at least remove
    /// the escape thing from process input events idk if that bi even works lowkey i need to tests
    pub fn update(&mut self) {
        self.process_events();
        self.glfw.poll_events();
        self.frame_buffer();
        self.window_handle.swap_buffers();

        self.last_frame_time = std::time::Instant::now();
    }

    fn frame_buffer(&mut self){
        let elapsed_time = self.last_frame_time.elapsed();
        //println!("Elapsed: {:?}, Target Frame Duration: {:?}", elapsed_time, self.target_frame_duration);
        if elapsed_time < self.target_frame_duration {
            //print!("sleep");
            let sleep_duration = self.target_frame_duration - elapsed_time;
            std::thread::sleep(sleep_duration); // Sleep for the remaining time to reach the target FPS
        }
    }


    
    //TODO: sthis and process_input_events might be redundant to have both!
    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // Make sure the viewport matches the new window dimensions.
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window_handle.set_should_close(true)
                }
                _ => {}
            }
        }
    }

    //get mouse position
    pub fn get_mouse_position(&self) -> (f64, f64) {
        let (x, y) = self.window_handle.get_cursor_pos();//hmmmmmmmmmmmmm
        (x, y)
    }

    //this is like not at all what I was trying to code but imma leave it just in case
    //anyone ever has a desire to do this
    pub fn reset_cursor(&mut self) {
        let (width, height) = self.window_handle.get_framebuffer_size();
        self.window_handle.set_cursor_pos(
            (width / 2) as f64, 
            (height / 2) as f64
        );
    }

    pub fn lock_cursor(&mut self) {
        self.window_handle.set_cursor_mode(glfw::CursorMode::Disabled);
    }

    pub fn unlock_cursor(&mut self) {
        self.window_handle.set_cursor_mode(glfw::CursorMode::Normal);
    }

    //this is to process events
    pub fn process_input_events(&mut self, input_system: &mut InputSystem) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {//TODO take this out later just add it to rest of inputs also where the hell is cursor position like idrecall 
                    self.window_handle.set_should_close(true)
                }
                glfw::WindowEvent::Key(glfw_key, _, action, _) => {
                    if let Some(key) = map_glfw_key(glfw_key) {
                        match action {
                            glfw::Action::Press => input_system.queue_event(InputEvent::KeyPressed(key)),
                            glfw::Action::Release => input_system.queue_event(InputEvent::KeyReleased(key)),
                            _ => {}
                        }
                    }
                }
                glfw::WindowEvent::MouseButton(button, action, _) => {
                    if let Some(button_index) = map_glfw_mousebutton(button) {
                        match action {
                            glfw::Action::Press => input_system.queue_event(InputEvent::MouseButtonPressed(button_index)),
                            glfw::Action::Release => input_system.queue_event(InputEvent::MouseButtonReleased(button_index)),
                            _ => {}
                        }
                    }
                }
                glfw::WindowEvent::Scroll(x_offset, y_offset) => {
                    input_system.queue_event(InputEvent::ScrollWheel(x_offset, y_offset));
                },
                _ => {}
            }
        }
    }
}
