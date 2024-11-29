//use acrust::custom_errors::Errors;
#![allow(warnings)]
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::input::input::{InputSystem, InputEvent, Key};
use acrust::graphics::gl_wrapper::*;
use crate::octo::{build_octree, OctreeNode};

use crate::voxel_render::VoxelRenderer;
use crate::chunk_generator::VoxelChunk;
use crate::chunk_manager::ChunkManager;

use crate::player::Player;

//use gl::types::*;
//use std::mem;
use std::time::Instant;
use cgmath::*;
//use std::env;

mod voxel_render;
mod player;
mod chunk_generator;
mod chunk_manager;
mod octo;


fn main() {
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();


    let mut input_system = InputSystem::new();//need to make this so that it is like added on window init or something

    let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    shader_program.bind();

    shader_program.create_uniform("transform");

    shader_program.enable_depth();

    let mut player = Player::new(10.0, 20.0, 10.0 , 1.0);

    let perspective = PerspectiveFov {
        fovy: Rad(1.0), // Field of view (vertical)
        aspect: 1.0,    // Aspect ratio
        near: 0.1,      // Near clipping plane
        far: 100.0,     // Far clipping plane
    };

    let mut camera = Camera::new(perspective);

    //attaching the camera to the player
    camera.attach_to(&player.transform);

    // Initialize Voxel Renderer
    let mut chunk = VoxelChunk::new(64, 64, 64);
    chunk.generate_hierarchical_terrain(42); // 42 is the seed
    let mut chunk_manager = ChunkManager::new();

    let position = (
                    0.0, // Spread chunks along x-axis
                    0.0,             // Same height
                    0.0  // Spread chunks along z-axis
                );

    chunk_manager.add_chunk(chunk, position);

    // Add multiple chunks at different positions
    // for x in 0..3 {
    //     for z in 0..3 {
    //         let mut chunk = VoxelChunk::new(16, 16, 16);
    //         let start = Instant::now();
    //         chunk.generate_wave_function_collapse(1);
    //         println!("Chunk generation took {:?}", start.elapsed());
                        
    //         // Position chunks with some spacing
    //         let position = (
    //             x as f32 * 5.0, // Spread chunks along x-axis
    //             0.0,             // Same height
    //             z as f32 * 5.0  // Spread chunks along z-axis
    //         );
            
    //         chunk_manager.add_chunk(chunk, position);
    //     }
    // }

    // let mut chunk2 = VoxelChunk::new(16, 16, 16);
    // chunk2.generate_wave_function_collapse(1223);
    // chunk_manager.add_chunk(chunk2, (128.0, 0.0, 0.0));

    // let voxel_renderer = VoxelRenderer::from_chunk(&chunk);

    while !window.should_close() {
        // Clear the screen
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let current_mouse_position = window.get_mouse_position();
        let (delta_x, delta_y) = input_system.update_mouse_position(current_mouse_position);

        window.lock_cursor();

        let sensitivity = 0.002;
        camera.rotate(-delta_x as f32 * sensitivity, -delta_y as f32 * sensitivity);


        window.process_input_events(&mut input_system);
        if input_system.is_key_pressed(&Key::W) {
            player.move_forward(camera.get_forward_vector());
        }
        if input_system.is_key_pressed(&Key::S) {
            player.move_backward(camera.get_forward_vector());
        }
        if input_system.is_key_pressed(&Key::A) {
            player.move_left(camera.get_left_vector());
        }
        if input_system.is_key_pressed(&Key::D) {
            player.move_right(camera.get_left_vector());
        }
        if input_system.is_key_pressed(&Key::Space) {
            player.move_up();
        }
        if input_system.is_key_pressed(&Key::LShift) {
            player.move_down();
        }

        chunk_manager.render_all();
        
        camera.update_view();
    
        while let Some(event) = input_system.get_event_queue().pop_front() {
            match event {
                InputEvent::KeyPressed(Key::Space) => {
                    println!("Jump");
                }
                InputEvent::KeyPressed(Key::Lctrl) => {
                    player.speed = 0.3;
                }
                InputEvent::KeyReleased(Key::Lctrl) => {
                    player.speed = 0.1;
                }
                _ => {}
            }
        }

        let transform = camera.get_vp_matrix();
        shader_program.set_matrix4fv_uniform("transform", &transform);
        

        window.update();
    }

}

