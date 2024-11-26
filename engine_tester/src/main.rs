//use acrust::custom_errors::Errors;
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::input::input::{InputSystem, InputEvent, Key};
use acrust::graphics::gl_wrapper::*;

use crate::voxel_render::VoxelRenderer;
use crate::voxel_render::VoxelChunk;

use crate::player::Player;

//use gl::types::*;
//use std::mem;
//use std::ptr;
use cgmath::*;
//use std::env;

mod voxel_render;
mod player;

fn main() {
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();


    let mut input_system = InputSystem::new();//need to make this so that it is like added on window init or something

    let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    shader_program.bind();

    shader_program.create_uniform("transform");

    shader_program.enable_depth();//BRQO I had this commented out and I could not for the life of me figure out why depth buffering was not working 😭😭😭

    //camera innit herm yeah might do this diff idk
    let mut player = Player::new();
    let perspective = PerspectiveFov {
        fovy: Rad(1.0), // Field of view (vertical)
        aspect: 1.0,    // Aspect ratio
        near: 0.1,      // Near clipping plane
        far: 100.0,     // Far clipping plane
    };

    let mut camera = Camera::new(perspective);

    camera.attach_to(&player.transform);


    // Initialize Voxel Renderer
    let mut chunk = VoxelChunk::new(16, 16, 16);
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                chunk.set_block(x, y, z, 1); // Fill the chunk with solid blocks
            }
        }
    }

    let voxel_renderer = VoxelRenderer::from_chunk(&chunk);
    

    let mut camera_speed = 0.1;

    while !window.should_close() {
        // Clear the screen
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        voxel_renderer.render();

        //mouse stuff maybe make a delta function somewhere to make this less messy
        let current_mouse_position = window.get_mouse_position();
        let (delta_x, delta_y) = input_system.update_mouse_position(current_mouse_position);

        //window.reset_cursor();

        window.lock_cursor();

        let sensitivity = 0.002;
        camera.rotate(-delta_x as f32 * sensitivity, -delta_y as f32 * sensitivity);


        window.process_input_events(&mut input_system);
        if input_system.is_key_pressed(&Key::W) {
            player.move_forward(camera.transform.rotation);
        }
        if input_system.is_key_pressed(&Key::S) {
            player.move_backward(camera.transform.rotation);
        }
        if input_system.is_key_pressed(&Key::A) {
            player.move_left(camera.transform.rotation);
        }
        if input_system.is_key_pressed(&Key::D) {
            player.move_right(camera.transform.rotation);
        }
        
        camera.update_view();
        
    
        while let Some(event) = input_system.get_event_queue().pop_front() {
            match event {
                InputEvent::KeyPressed(Key::Space) => {
                    println!("Jump");
                }
                InputEvent::KeyPressed(Key::LShift) => {
                    camera_speed = 0.3;
                }
                InputEvent::KeyReleased(Key::LShift) => {
                    camera_speed = 0.1;
                }
                _ => {}
            }
        }

        let transform = camera.get_vp_matrix();
        shader_program.set_matrix4fv_uniform("transform", &transform);
        

        window.update();
    }

}

