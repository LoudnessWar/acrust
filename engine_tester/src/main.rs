//use acrust::custom_errors::Errors;
#![allow(warnings)]
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::graphics::texture_manager::TextureManager;
use acrust::graphics::skybox::Skybox;
use acrust::input::input::{InputSystem, InputEvent, Key, CLICKS};
use acrust::graphics::gl_wrapper::*;
use acrust::user_interface::ui_element::UIElement;
use acrust::user_interface::ui_manager::UIManager;
use acrust::user_interface::ui_element::UIElementTrait;
use acrust::user_interface::ui_manager::UIEvent;
use acrust::user_interface::ui_element::Button;
use acrust::user_interface::ui_element::Slider;
use acrust::user_interface::ui_element::UIElementVisitor;
use acrust::graphics::materials::Material;


use crate::octo::OctreeNode;
use crate::voxel_render::VoxelRenderer;
use crate::chunk_generator::*;
use crate::chunk_manager::ChunkManager;
// use crate::wave_generator::WaterRender;
use cgmath::Vector3;
use acrust::model::cube::Cube;

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



//:big todo give everything a model matrix rn things are just at origin with verticies deciding their location, and not like a proper model scheme, everyobject should from now on
//get a transformation matrix(not everyhing only the things that are using the transormation matrix so after this push, literally only the voxels), for its like where it is relative to the camera, but they should also all get a model matrix
//
fn main() {
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();
    println!("after window init");

    let mut input_system = InputSystem::new();


    let mut shader_manager = ShaderManager::new();
    shader_manager.load_shader("land", "shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    shader_manager.load_shader("water", "shaders/water_vertex_shader.glsl", "shaders/water_fragment_shader.glsl");
    

    // let mut shaders_land = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    // let mut shaders_water = ShaderProgram::new("shaders/water_vertex_shader.glsl", "shaders/water_fragment_shader.glsl");
    let mut ui_shader = ShaderProgram::new("shaders/ui_vertex.glsl", "shaders/ui_fragment.glsl");
    ui_shader.create_uniform("projection");
    ui_shader.create_uniform("color");
    ui_shader.create_uniform("useTexture");

    shader_manager.enable_backface_culling("land");//have this on by default prolly and make it so you have to turn them off if you wsnat them off I think
    shader_manager.enable_depth("land");
    let mut material1 = Material::new("land");
    //material1.initialize_uniforms();

    //let mut water = WaterRender::new(20.0, 20.0, 5.0, shader_manager.);
    //water.set_position(water.get_position() - Vector3::new(10.0, 0.0, 10.0));

    let mut player = Player::new(0.0, 5.0, 10.0 , 100.0);

    let perspective = PerspectiveFov {
        fovy: Rad(1.0), // Field of view (vertical)
        aspect: 1.0,    // Aspect ratio
        near: 0.1,      // Near clipping plane
        far: 1000.0,     // Far clipping plane
    };

    let mut camera = Camera::new(perspective);
    camera.attach_to(&player.transform);


    let mut chunk_manager = ChunkManager::new();
    let mut terrain = TerrainGenerator::new(42, 16);

    let chunks = terrain.generate_multiple_chunks(0, 32, 0, 2, 1, 16);

    for (octree, position) in chunks {
        chunk_manager.add_octree(octree, position);
    }

    let octree = terrain.get_root();
    chunk_manager.add_octree(octree.clone(), (0.0, 0.0, 0.0));


    //cube

    // let mut cube = Cube::new(1, Vector3::new(0.0, 0.0, 0.0), 1.0, 1.0, 1.0, &material1);

    // cube.translate(Vector3::new(1.0, 0.0, 0.0));


    // let skybox_faces = [
    //     "textures/right.jpg",
    //     "textures/left.jpg",
    //     "textures/top.png",
    //     "textures/bottom.png",
    //     "textures/front.jpg",
    //     "textures/back.jpg",
    // ];

    // let skybox = Skybox::new(&skybox_faces);

    // let skybox_shader = ShaderProgram::new("shaders/skybox_vertex_shader.glsl", "shaders/skybox_fragment_shader.glsl");
    // let mut skybox_material = Material::new(skybox_shader);
    // skybox_material.init_uniform("view");
    // skybox_material.init_uniform("projection");
    // skybox_material.init_uniform("skybox");
    
    let mut texture_manager = TextureManager::new();

    let texture_id = texture_manager.load_texture("textures/right.jpg")
                .expect("Failed to load texture");

    let mut ui_manager = UIManager::new(720.0, 720.0);
    let mut ui_element = UIElement::new(1, Vector2::new(50.0, 50.0), Vector2::new(200.0, 100.0));
    ui_element.set_texture(texture_id);
    let mut ui_element2 = UIElement::new(2, Vector2::new(90.0,90.0), Vector2::new(100.0, 100.0));    
    ui_element2.set_color(Vector4::new(1.0, 0.0, 0.0, 1.0));
    let mut ui_button = Button::new(3, Vector2::new(400.0,90.0), Vector2::new(200.0, 100.0)); //button is a bad fucking name 
    ui_element2.set_color(Vector4::new(1.0, 1.0, 0.0, 1.0));
    ui_manager.add_element(Box::new(ui_element));
    ui_manager.add_element(Box::new(ui_element2));
    ui_manager.add_element(Box::new(ui_button));

    let current_mouse_position = window.get_mouse_position();//not really needed i think
    window.lock_cursor();
    let mut sensitivity = 0.002;

    let mut visitor = ClickVisitor::new();
    let mut time = 0.0;




    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let current_mouse_position = window.get_mouse_position();
        let (delta_x, delta_y) = input_system.update_mouse_position(current_mouse_position);

        camera.rotate(-delta_x as f32 * sensitivity, -delta_y as f32 * sensitivity);

        //uuuh problem I geniuenly forget how this imput system works cry emoji
        //I fink if I rememba correctry its like uuuhhhh process input events calls input:;events and adds them to que directly
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
        if input_system.is_key_pressed(&Key::Tab) {
            ui_manager.update(current_mouse_position);
            ui_manager.render(&ui_shader);
        }

        camera.update_view();
    
        while let Some(event) = input_system.get_event_queue().pop_front() {
            match event {
                InputEvent::KeyPressed(Key::Lctrl) => {
                    player.speed = 0.3;
                }
                InputEvent::KeyReleased(Key::Lctrl) => {
                    player.speed = 0.1;
                }
                InputEvent::KeyPressed(Key::Tab) => {//Ok need a function to lock other inputs from coming in so like you dont interact with outside world when ja
                    window.unlock_cursor();
                    sensitivity = 0.000;
                }
                InputEvent::KeyReleased(Key::Tab) => {
                    window.lock_cursor();
                    sensitivity = 0.002;
                }
                InputEvent::MouseButtonPressed(CLICKS::Left) => {
                    println!("pewpew");
                    if (ui_manager.is_element_hovered(3)){//somthing here to pattern match instead of this
                        ui_manager.visit_element(3, &mut visitor);
                    }
                }
                InputEvent::MouseButtonPressed(CLICKS::Right) => {
                    println!("clear");
                }
                _ => {}
            }
        }

        while let Some(event) = ui_manager.poll_event() {
            match event {
                UIEvent::Hover(id) => {},
                UIEvent::Click(id) => {},
                UIEvent::MouseEnter(id) => {
                    println!("Mouse entered element {}", id);
                },
                UIEvent::MouseExit(id) => {
                    println!("Mouse exited element {}", id);
                },
                _ => {}
            }
        }

        let transform = camera.get_vp_matrix();
    
        material1.apply_no_model(&shader_manager, &texture_manager);
        material1.set_matrix4_property(&mut shader_manager, "transform", transform.clone());
        chunk_manager.render_all();

        //water.render(time, &camera);

        //cube.render(material1.borrow_shader(), &camera.get_vp_matrix());

        // {
        //     let view_matrix = skybox.get_skybox_view_matrix(&camera.get_view());
        //     let projection_matrix = camera.get_p_matrix();
        
        //     skybox_material.apply();
        //     skybox.render(skybox_material.borrow_shader(), &view_matrix, &projection_matrix);
        // }
        

        window.update();
        time += 0.1;
    }

}


//THIS IS JUST GOING DOWN HERE BC IM LAZY, A BASIC ALIEN LIKE THIS SHOULD JUST AUTOMATICALLY BE IMPLIMENTED INTO THE UI
pub struct ClickVisitor {
    pub button_clicked: bool,
    //pub input_system: &InputSystem,//eeeeehhhhhhh I think there can be better solutions
}
//ill just talk about it here
//we have this que and this visit system ok whatever its not the end of the world
//its clunky I want is clicked to be in the thing not a class you have to write yourself
//yk i want it built into button

impl ClickVisitor {
    pub fn new() -> Self {
        Self {
            button_clicked: false,
            //input_system: false,
        }
    }
}

impl UIElementVisitor for ClickVisitor {
    fn visit_button(&mut self, button: &mut Button, is_clicked: bool) {
        if is_clicked{
            self.button_clicked = true;
            println!("Button clicked: ID {}", button.get_id());
            button.set_position(button.get_position() + Vector2::new(10.0, 0.0));
        }
    }

    fn visit_slider(&mut self, slider: &mut Slider) {
        println!("Slider value: {}", slider.get_value());
    }
}

