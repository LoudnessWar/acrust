//use acrust::custom_errors::Errors;
#![allow(warnings)]
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::graphics::skybox::Skybox;
use acrust::input::input::{InputSystem, InputEvent, Key};
use acrust::graphics::gl_wrapper::*;


use crate::octo::OctreeNode;
use crate::voxel_render::VoxelRenderer;
use crate::chunk_generator::*;
use crate::chunk_manager::ChunkManager;
use crate::wave_generator::WaterRender;

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
mod wave_generator;


fn main() {
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();


    let mut input_system = InputSystem::new();//need to make this so that it is like added on window init or something

    let mut shaders_land = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    let mut shaders_water = ShaderProgram::new("shaders/water_vertex_shader.glsl", "shaders/fragment_shader.glsl");

    shaders_land.enable_backface_culling();
    shaders_land.enable_depth();

    let mut material1 = Material::new(shaders_land);
    material1.initialize_uniforms();

    let mut material2 = Material::new(shaders_water);
    material2.initialize_uniforms();

    let mut player = Player::new(0.0, 5.0, 10.0 , 100.0);

    let perspective = PerspectiveFov {
        fovy: Rad(1.0), // Field of view (vertical)
        aspect: 1.0,    // Aspect ratio
        near: 0.1,      // Near clipping plane
        far: 100.0,     // Far clipping plane
    };

    let mut camera = Camera::new(perspective);

    //attaching the camera to the player
    camera.attach_to(&player.transform);

    let mut water = WaterRender::new(20.0, 20.0, 5.0);

    let mut chunk_manager = ChunkManager::new();
    let mut terrain = TerrainGenerator::new(42, 16);

    let chunks = terrain.generate_multiple_chunks(0, 64, 0, 2, 1, 16);

    for (octree, position) in chunks {
        chunk_manager.add_octree(octree, position);
    }


    let octree = terrain.get_root();
    chunk_manager.add_octree(octree.clone(), (0.0, 0.0, 0.0));

    //skybox

        let skybox_faces = [
            "textures/right.jpg",
            "textures/left.jpg",
            "textures/top.png",
            "textures/bottom.png",
            "textures/front.jpg",
            "textures/back.jpg",
        ];

        let skybox = Skybox::new(&skybox_faces);

        let skybox_shader = ShaderProgram::new("shaders/skybox_vertex_shader.glsl", "shaders/skybox_fragment_shader.glsl");
        let mut skybox_material = Material::new(skybox_shader);
        skybox_material.add_uniform("view");
        skybox_material.add_uniform("projection");


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
        if input_system.is_key_pressed(&Key::Left) {
            camera.rotate(10.0 as f32 * sensitivity, 0.0);
        }
        if input_system.is_key_pressed(&Key::Right) {
            camera.rotate(-10.0 as f32 * sensitivity, 0.0);
        }
        if input_system.is_key_pressed(&Key::Up) {
            camera.rotate(0.0, 10.0 as f32 * sensitivity);
        }
        if input_system.is_key_pressed(&Key::Down) {
            camera.rotate(0.0, -10.0 as f32 * sensitivity);
        }
        if input_system.is_key_pressed(&Key::Space) {
            player.move_up();
        }
        if input_system.is_key_pressed(&Key::LShift) {
            player.move_down();
        }

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

        {
            let view_matrix = camera.get_vp_matrix();
            let projection_matrix = camera.get_p_matrix();
    
            skybox_material.apply();
            skybox_material.set_matrix4fv_uniform("view", &view_matrix);
            skybox_material.set_matrix4fv_uniform("projection", &projection_matrix);
    
            skybox.render(skybox_material.borrow_shader(), &view_matrix, &projection_matrix);
        }
    

        material1.apply();
        material1.set_matrix4fv_uniform("transform", &transform);
        chunk_manager.render_all();

        material2.apply();
        material2.set_matrix4fv_uniform("transform", &transform);
        water.render();
        

        window.update();
    }

}

