//use acrust::custom_errors::Errors;
//#![allow(warnings)]
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::input::input::{InputSystem, InputEvent, Key};
use acrust::graphics::gl_wrapper::*;
use crate::octo::OctreeNode;

use crate::voxel_render::VoxelRenderer;
use crate::chunk_generator::VoxelChunk;
use crate::chunk_manager::ChunkManager;

use crate::player::Player;
use crate::octo::TileRules;

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
    // Initialize the window and OpenGL
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();

    let mut input_system = InputSystem::new();

    let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    shader_program.bind();
    shader_program.create_uniform("transform");
    shader_program.enable_depth();

    let mut player = Player::new(10.0, 20.0, 10.0, 1.0);

    // Initialize camera with perspective settings and attach it to the player
    let perspective = PerspectiveFov {
        fovy: Rad(1.0),  // Field of view (vertical)
        aspect: 1.0,     // Aspect ratio
        near: 0.1,       // Near clipping plane
        far: 100.0,      // Far clipping plane
    };

    let mut camera = Camera::new(perspective);
    camera.attach_to(&player.transform);

    // Define block types and adjacency rules for WFC
    let block_types = vec![0, 1, 2, 3, 4]; // Air, Grass, Dirt, Stone, Bedrock
    let tile_rules = TileRules::new(&block_types);
    let mut chunk_manager = ChunkManager::new(tile_rules);

    println!("before");
    // Initialize the chunk manager with the tile rules
    //let mut chunk_manager = ChunkManager::new(tile_rules);

    chunk_manager.add_chunk(64, (0.0, 0.0, 0.0), 42);

    println!("ater");
    // Generate and add octree-based chunks
    // for x in 0..3 {
    //     for z in 0..3 {
    //         let start = Instant::now();

    //         // Generate octree using WFC and add it to the chunk manager
    //         chunk_manager.add_chunk(
    //             16,                             // Chunk size (16x16x16)
    //             (x as f32 * 16.0, 0.0, z as f32 * 16.0), // Position in world space
    //             1 + x * 3 + z * 7,              // Unique seed for WFC
    //         );

    //         println!("Chunk generation took {:?}", start.elapsed());
    //     }
    // }

    // Main rendering loop
    while !window.should_close() {
        // Clear the screen
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Update mouse position for camera rotation
        let current_mouse_position = window.get_mouse_position();
        let (delta_x, delta_y) = input_system.update_mouse_position(current_mouse_position);
        window.lock_cursor();

        let sensitivity = 0.002;
        camera.rotate(-delta_x as f32 * sensitivity, -delta_y as f32 * sensitivity);

        // Process player movement
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

        // Handle input events like speed changes
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

        // Render chunks using the chunk manager
        chunk_manager.render_all();

        // Update camera view matrix
        camera.update_view();

        // Set the transformation matrix for the shader
        let transform = camera.get_vp_matrix();
        shader_program.set_matrix4fv_uniform("transform", &transform);

        // Update the window
        window.update();
    }
}
