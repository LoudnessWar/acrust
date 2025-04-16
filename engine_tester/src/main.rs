//use acrust::custom_errors::Errors;
#![allow(warnings)]
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::graphics::texture_manager::TextureManager;
use acrust::graphics::skybox::Skybox;
use acrust::graphics::gl_wrapper::*;
use acrust::graphics::materials::Material;
use acrust::graphics::materials::MaterialManager;

use acrust::input::input::{InputSystem, InputEvent, Key, CLICKS};

use acrust::user_interface::ui_element::UIElement;
use acrust::user_interface::ui_manager::UIManager;
use acrust::user_interface::ui_element::UIElementTrait;
use acrust::user_interface::ui_manager::UIEvent;
use acrust::user_interface::ui_element::Button;
use acrust::user_interface::ui_element::Slider;
use acrust::user_interface::ui_element::UIElementVisitor;
use acrust::user_interface::ui_element::*;
use acrust::user_interface::ui_manager::DragState;


use acrust::model::objload::Model;
use acrust::model::transform::WorldCoords;
use acrust::model::objload::GeneralModel;
use acrust::model::cube::Cube;
use acrust::model::objload::load_obj;

use acrust::graphics::gl_wrapper;//going to remove later


use acrust::sound::sound::*;
use std::{sync::mpsc, thread, time::Duration};



use crate::octo::OctreeNode;
use crate::voxel_render::VoxelRenderer;
use crate::chunk_generator::*;
use crate::chunk_manager::ChunkManager;
use crate::wave_generator::WaterRender;
use crate::midi::MidiHandler;
use crate::player::Player;

use cgmath::Vector3;
use std::time::Instant;
use cgmath::*;


mod voxel_render;
mod player;
mod chunk_generator;
mod chunk_manager;
mod octo;
mod wave_generator;
mod midi;

fn main() {
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();

    let mut input_system = InputSystem::new();

    let mut shader_manager = ShaderManager::new();
    shader_manager.load_shader("Basic", "shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    shader_manager.load_shader("generic", "shaders/generic_vertex.glsl", "shaders/generic_fragment.glsl");

    //shader_manager.init_forward_plus_light_debug();

    shader_manager.init_forward_plus();

    let depth_shader = shader_manager.get_shader("depth").unwrap();
    let light_shader = shader_manager.get_shader("light").unwrap();

    let mut ui_shader = ShaderProgram::new("shaders/ui_vertex.glsl", "shaders/ui_fragment.glsl");
    ui_shader.create_uniform("projection");
    ui_shader.create_uniform("color");
    ui_shader.create_uniform("useTexture");

    //ShaderManager::enable_backface_culling();
    ShaderManager::enable_depth();

    let mut light_manager = LightManager::new();

    light_manager.lights.push(Light {
        position: [0.0, -20.0, 0.0],
        radius: 10.0,
        color: [1.0, 1.0, 1.0],
        intensity: 100.0
    });

    light_manager.lights.push(Light {
        position: [0.0, 15.0, -15.0],
        radius: 30.0,
        color: [0.2, 1.0, 1.0],
        intensity: 100.0
    });

    //let mut debug_comp_shader = LightManager::create_debug_display_shader();

    //light_manager.initialize_gpu_culling(720, 720, &shader_manager);

    let mat_man = MaterialManager::new();//ok I am going to give a like reasoning here as to why this isn't like a global variable
    //or something and there are so many hoops jumped through with this and MaterialManager... ok the simple reason is
    //its safer and uuh down the line will work out better because we arn't using this for read only. IE we are editing stuff even in the
    //functions that dont we are just getting around decalairing it as such because of unsafe. SO basically. this is safer.

    let material = mat_man.load_material("mat1", &shader_manager, "Basic");
    let material = mat_man.load_material("mat2", &shader_manager, "generic");
    
    mat_man.init_uniform("mat2", "model");
    mat_man.init_uniform("mat2", "view");
    mat_man.init_uniform("mat2", "projection");

    mat_man.init_uniform("mat1", "transform");

    let mut player = Player::new(0.0, 0.0, -10.0 , 100.0);

    let perspective = PerspectiveFov {
        fovy: Rad(1.0),
        aspect: 1.0,
        near: 1.0,
        far: 1000.0,
    };

    let mut camera = Camera::new(perspective);

    // let mut camera = Camera::new_reversed_z(1.0, 1.0, 4.0, 1000.0);

    camera.attach_to(&player.transform);

    let mut cube = Cube::new(5.0, Vector3::new(0.0, 0.0, 0.0), 1.0, mat_man.get_mat("mat1"));
    let mut cube2 = Cube::new(5.0, Vector3::new(15.0, 15.0, 15.0), 1.0, mat_man.get_mat("mat1"));

    let mut model = GeneralModel::new(load_obj("models/teddy.obj"), WorldCoords::new(0.0, 10.0, 100.0, 1.0), mat_man.get_mat("mat2"));
    mat_man.update_uniform("mat2", "lightDir", UniformValue::Vector3(vec3(0.0, 10.0, 0.0)));
    mat_man.update_uniform("mat2", "lightColor", UniformValue::Vector3(vec3(0.0, 1.0, 1.0)));
    mat_man.update_uniform("mat2", "objectColor", UniformValue::Vector3(vec3(1.0, 1.0, 1.0)));

    let mut texture_manager = TextureManager::new();

    let texture_id = texture_manager.load_texture("textures/right.jpg")
                .expect("Failed to load texture");

    let mut ui_manager = UIManager::new(720.0, 720.0);

    let mut ui_element = UIElement::new(1, Vector2::new(50.0, 50.0), Vector2::new(200.0, 100.0));
    ui_element.set_texture(texture_id);

    let mut ui_element2 = UIElement::new(2, Vector2::new(90.0,90.0), Vector2::new(100.0, 100.0));    
    ui_element2.set_color(Vector4::new(1.0, 0.0, 0.0, 1.0));

    let mut ui_draggable = UIDraggable::new(4, Vector2::new(120.0,120.0), Vector2::new(200.0, 200.0));    
    ui_draggable.set_color(Vector4::new(0.0, 1.0, 1.0, 1.0));

    let mut ui_button = Button::new(3, Vector2::new(400.0,90.0), Vector2::new(200.0, 100.0)); //button is a bad fucking name 
    ui_element2.set_color(Vector4::new(1.0, 1.0, 0.0, 1.0));
    
    ui_manager.add_element(Box::new(ui_element));
    ui_manager.add_element(Box::new(ui_element2));
    ui_manager.add_element(Box::new(ui_button));
    ui_manager.add_element(Box::new(ui_draggable));

    window.lock_cursor();
    let mut sensitivity = 0.002;

    let mut visitor = ClickVisitor::new();
    let mut time = 0.0;

    let mut ds = DragState::new();

    let mut fpr = ForwardPlusRenderer::new(&shader_manager);

    fpr.add_light(
        [0.0, -20.0, 0.0],  // position
        10.0,             // radius
        [1.0, 1.0, 1.0],  // color (white)
        100.0               // intensity
    );

    fpr.add_light(
        [0.0, 15.0, 0.0],
        20.0, // Giant radius
        [0.2, 0.3, 1.0],
        100.0
    );

    fpr.initialize_light_culling(720, 720, &shader_manager);

    let models: Vec<Model> = vec![Model::new(load_obj("models/teddy.obj"), WorldCoords::new(0.0, 0.0, 0.0, 0.0), mat_man.get_mat("mat2"))];

    
    while !window.should_close() {
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            let ctx_err = gl::GetError();
            if ctx_err != gl::NO_ERROR {
                panic!("GL context error before while loop or from previous call: 0x{:X}", ctx_err);
            }
        }

        let current_mouse_position = window.get_mouse_position();
        let (delta_x, delta_y) = input_system.update_mouse_position(current_mouse_position);

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
        if input_system.is_key_pressed(&Key::Tab) {
            ui_manager.update(current_mouse_position);
            ui_manager.render(&ui_shader);
            if input_system.is_mouse_button_just_pressed(&CLICKS::Left) {
                ui_manager.start_drag(current_mouse_position);
            }
            if input_system.is_mouse_button_held(&CLICKS::Left) {
                ui_manager.update_dragging(current_mouse_position);
            }
            if input_system.is_mouse_button_released(&CLICKS::Left) {
                ui_manager.end_drag();
            }
        }
    
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
                    ds.end_drag();
                }
                InputEvent::MouseButtonReleased(CLICKS::Left) => {
                    println!("clear");
                    ds.end_drag();
                }
                InputEvent::MouseButtonPressed(CLICKS::Right) => {
                    println!("clear");
                }
                _ => {}
            }
        }

        camera.update_view();

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

        model.set_uniforms(&texture_manager, &camera);

        // unsafe {
        //     gl::Enable(gl::DEPTH_TEST);
        //     gl::DepthFunc(gl::GEQUAL);  // For reverse Z
        //     gl::ClearDepth(0.0);         // Clear to far plane (0.0 in reverse Z)
        //     gl::ClipControl(gl::LOWER_LEFT, gl::ZERO_TO_ONE);//this might muck up some other thigns
        // }

        fpr.render(
            &models,
            &camera,
            720,
            720,
            &texture_manager
        );

        // fpr.render_debug(
        //     &models,
        //     &camera,
        //     720,
        //     720,
        //     &texture_manager,
        //     &mut debug_comp_shader
        // );



        // mat_man.update_uniform("mat1", "transform", &transform);
        // cube.render(&texture_manager);
        // cube2.render(&texture_manager);
        //model.simple_render(&texture_manager, &camera);

        window.update();//frame_buffer here
        time += 0.1;
        //panic!("end");
    }
}



//:big todo give everything a model matrix rn things are just at origin with verticies deciding their location, and not like a proper model scheme, everyobject should from now on
//get a transformation matrix(not everyhing only the things that are using the transormation matrix so after this push, literally only the voxels), for its like where it is relative to the camera, but they should also all get a model matrix

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

