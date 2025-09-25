#![allow(warnings)]
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::graphics::texture_manager::TextureManager;
use acrust::graphics::skybox::Skybox;
use acrust::graphics::gl_wrapper::*;
use acrust::graphics::materials::Material;
use acrust::graphics::materials::MaterialManager;

use acrust::input::input::{InputSystem, InputEvent, Key, CLICKS};

use acrust::ecs::UI_components::UILayout;
use acrust::ecs::UI_components::UITransform;
use acrust::ecs::UI_components::UIStyle;


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
use acrust::model::objload::ModelTrait;
use acrust::model::transform::WorldCoords;
//use acrust::model::objload::GeneralModel;

use acrust::graphics::camera::CameraMode;//this can be not in here if I do it correct

use acrust::user_interface::text_render::TextRenderer;
use acrust::user_interface::visitors::TextRenderVisitor;

use acrust::model::cube::Cube;
use acrust::model::triangle::Triangle;
use acrust::model::objload::load_obj;
use acrust::model::objload::load_obj_new_normals;

use acrust::ecs::player::Player;
use acrust::ecs::world::{World, Entity};
use acrust::ecs::components::{Renderable, Velocity};

use acrust::graphics::gl_wrapper;

use acrust::sound::sound::*;
use std::{sync::mpsc, thread, time::Duration};

use crate::octo::OctreeNode;
use crate::voxel_render::VoxelRenderer;
use crate::chunk_generator::*;
use crate::chunk_manager::ChunkManager;
use crate::wave_generator::WaterRender;
use crate::midi::MidiHandler;

use cgmath::Vector3;
use std::time::Instant;
use cgmath::*;

mod voxel_render;
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

    shader_manager.init_forward_plus();

    let depth_shader = shader_manager.get_shader("depth").unwrap();
    let light_shader = shader_manager.get_shader("light").unwrap();

    let mut ui_shader = ShaderProgram::new("shaders/ui_vertex.glsl", "shaders/ui_fragment.glsl");
    ui_shader.create_uniform("projection");
    ui_shader.create_uniform("color");
    ui_shader.create_uniform("useTexture");

    ShaderManager::enable_backface_culling();
    ShaderManager::enable_depth();

    let mat_man = MaterialManager::new();
    let material = mat_man.load_material("mat1", &shader_manager, "Basic");
    let material = mat_man.load_material("mat2", &shader_manager, "generic");
    
    mat_man.init_uniform("mat2", "model");
    mat_man.init_uniform("mat2", "view");
    mat_man.init_uniform("mat2", "projection");
    mat_man.init_uniform("mat1", "transform");

    // I should probably add player to the ecs
    let mut player = Player::new(0.0, 0.0, -10.0, 100.0);

    let perspective = PerspectiveFov {
        fovy: Rad(1.0),
        aspect: 1.0,
        near: 1.0,
        far: 1000.0,
    };

    let mut camera = Camera::new(perspective);
    camera.attach_to(&player.transform, Vector3::new(0.0, 5.0, 10.0));
    camera.tp();
    camera.update_view();//not really needed here but good to have
    //camera.attach_to(&player.transform, Vector3::new(10.0, 0.0, 0.0));//this system is scuffed camera and player should be in ecs to avoid attach detech but for now eeh. TODO

    // Initialize materials
    mat_man.update_uniform("mat2", "lightDir", UniformValue::Vector3(vec3(0.0, 10.0, 0.0)));
    mat_man.update_uniform("mat2", "lightColor", UniformValue::Vector3(vec3(0.0, 1.0, 1.0)));
    mat_man.update_uniform("mat2", "objectColor", UniformValue::Vector3(vec3(1.0, 1.0, 1.0)));

    let mut texture_manager = TextureManager::new();
    let texture_id = texture_manager.load_texture("textures/right.jpg")
                .expect("Failed to load texture");

    
    let mut text_shader = ShaderProgram::new("shaders/text.vert", "shaders/text.frag");
    text_shader.create_uniform("projection");
    text_shader.create_uniform("textColor");
    let mut text_renderer = TextRenderer::new(text_shader);
    text_renderer.load_font("fonts/Roboto.ttf", 24.0);


    window.lock_cursor();
    let mut sensitivity = 0.002;

    //let mut visitor = ClickVisitor::new();
    let mut time = 0.0;

    let mut ds = DragState::new();

    // Initialize ForwardPlusRenderer
    let mut fpr = ForwardPlusRenderer::new(&shader_manager);

    fpr.add_light(
        [0.0, -1.0, 20.0],  // position
        50.0,             // radius
        [1.0, 1.0, 1.0],  // color (white)
        10.1               // intensity
    );

    fpr.add_light(
        [0.0, 20.0, 5.0],
        30.0, 
        [0.2, 0.3, 1.0],
        10.1
    );

    fpr.initialize_light_culling(720, 720, &shader_manager);

    // Initialize the ECS World
    let mut world = World::new_with_ui(720.0, 720.0, text_renderer);

    let (main_menu_id, ui_element1_id, ui_element2_id, ui_button_id, ui_text_id) = setup_ui_system(&mut world, texture_id);
    
    // Create entities in the ECS
    let teddy_entity = world.create_entity("Teddy");
    let teddy_model = Model::new(
        load_obj_new_normals("models/teddy.obj"), 
        WorldCoords::new(0.0, 0.0, 0.0, 0.0), 
        mat_man.get_mat("mat2")
    );
    
    // Add components to entities
    world.movement.add_coords(teddy_entity.id, WorldCoords::new(0.0, 0.0, 0.0, 0.0));
    world.movement.add_velocity(teddy_entity.id, Velocity {
        direction: Vector3::new(0.0, 0.0, 0.0),
        speed: 0.0
    });
    world.render.add_renderable(teddy_entity.id, Renderable {
        model: Box::new(teddy_model)
    });
    
    // Create a triangle entity
    let triangle_entity = world.create_entity("Triangle");
    let triangle_model = Triangle::new(
        2.0, 4.0, 
        Vector3::new(0.0, 0.0, 0.0), 
        0.0, 
        mat_man.get_mat("mat2")
    );
    
    world.movement.add_coords(triangle_entity.id, WorldCoords::new(10.0, 20.0, 20.0, 0.0));
    println!("triangle coords: {:#?}", world.movement.get_coords(triangle_entity.id).unwrap().get_position());
    world.movement.add_velocity(triangle_entity.id, Velocity {
        direction: Vector3::new(0.0, 0.0, 0.0),
        speed: 0.0
    });
    world.render.add_renderable(triangle_entity.id, Renderable {
        model: Box::new(triangle_model)
    });
    
    // AYYY fuck this guy
    let player_entity = world.spawn_player("MainPlayer", 0.0, 0.0, -10.0, 0.0);

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
    let mut skybox_material = Material::new_unlocked(skybox_shader);
    skybox_material.init_uniform("view");
    skybox_material.init_uniform("projection");
    
    // Time tracking for delta time calculation
    let mut last_frame_time = Instant::now();

    let mut show_ui = false;

    while !window.should_close() {
        // Calculate delta time
        let current_time = Instant::now();
        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = current_time;
        
        unsafe {
            gl::ClearColor(0.5, 0.3, 0.6, 1.0);
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

        // THIS IS GENIENLY SUPER SCUFFED TODO FIX THIS HOW THIS IS DONE
        // if input_system.is_key_pressed(&Key::W) {
        //     player.move_forward(camera.get_forward_vector());
        //     // Also update the ECS player position for synchronization if needed
        //     //this like also needs to be like a function of something like... omglob
        //     //man I love the way if let be though like feel funcitonal yk
        //     if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
        //         coords.position = *player.get_position();
        //     }
        // }
        // if input_system.is_key_pressed(&Key::S) {
        //     player.move_backward(camera.get_forward_vector());
        //     if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
        //         coords.position = *player.get_position();
        //     }
        // }
        // if input_system.is_key_pressed(&Key::A) {
        //     player.move_left(camera.get_left_vector());
        //     if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
        //         coords.position = *player.get_position();
        //     }
        // }
        // if input_system.is_key_pressed(&Key::D) {
        //     player.move_right(camera.get_left_vector());
        //     if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
        //         coords.position = *player.get_position();
        //     }
        // }

        if input_system.is_key_pressed(&Key::W) {
            player.move_forward_with_camera(&camera);//eehhhhhhhhhhhh meehehheheheh hidk if I like this yo TODOs
            if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
                coords.position = *player.get_position();
            }
        }
        if input_system.is_key_pressed(&Key::S) {
            player.move_backward_with_camera(&camera);
            if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
                coords.position = *player.get_position();
            }
        }
        if input_system.is_key_pressed(&Key::A) {
            player.move_left_with_camera(&camera);
            if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
                coords.position = *player.get_position();
            }
        }
        if input_system.is_key_pressed(&Key::D) {
            player.move_right_with_camera(&camera);
            if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
                coords.position = *player.get_position();
            }
        }
        
        if input_system.is_key_pressed(&Key::Q) {
            // println!("Time: {}", time);
            // if time % 2.0 == 1.0 { lol floating point precision error anyway
            //     camera.cycle_mode();
            // }
        }
        if input_system.is_key_pressed(&Key::Space) {
            player.move_up();
            if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
                coords.position = *player.get_position();
            }
        }
        if input_system.is_key_pressed(&Key::LShift) {
            player.move_down();
            if let Some(coords) = world.movement.get_coords_mut(player_entity.id) {
                coords.position = *player.get_position();
            }
        }
        
        // Handle camera rotation keys
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

        if input_system.has_scrolled() {
            let (x_offset, y_offset) = input_system.get_scroll_offset();
            println!("Scroll offsets: X={}, Y={}", x_offset, y_offset);
            if matches!(camera.mode, CameraMode::ThirdPerson) {
                camera.adjust_third_person_distance(-y_offset as f32);
            }
        }
    


        camera.update_view();

        // ecs update who gaf
        //world.update(delta_time);
        let mouse_down = input_system.is_mouse_button_held(&CLICKS::Left);
        let mouse_clicked = input_system.is_mouse_button_just_pressed(&CLICKS::Left);

        world.update_ui(delta_time, current_mouse_position, mouse_down, mouse_clicked);
        
        // This is like 3 funcitons deep at this point world -> render -> fpr -> five different fucntions
        world.render(&mut fpr, &camera, 720, 720, &texture_manager);
        //OOOOKKKKAAAYYYYYYY SOOOOOOOO... IF IT DOESNT HAPPEN AFTER HERE IT DONT HAPPEN LOOKING AT YOU UI NEED TO FIX

        //BRO ITS SO SLOW HELP!!! I think I was lowkey just adding stuff back 

        let view_matrix = skybox.get_skybox_view_matrix(&camera.get_view());
        let projection_matrix = camera.get_p_matrix();
        skybox.render(&mut skybox_material, &texture_manager, &view_matrix, &projection_matrix);


        if show_ui {
            // Enable blending for UI
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::Disable(gl::DEPTH_TEST);
            }
            
            // Render UI
            world.ui.render(&ui_shader);
            
            // Handle UI interactions
            if world.is_ui_button_clicked(ui_button_id) {
                println!("ECS UI Button clicked!");
                // Move the button as an example
                world.update_ui_element_position(ui_button_id, Vector2::new(250.0, 200.0));
            }
            
            if world.is_ui_button_hovered(ui_button_id) {
                // Change button color on hover
                world.update_ui_element_color(ui_button_id, Vector4::new(0.8, 0.8, 1.0, 1.0));
            } else {
                // Reset button color
                world.update_ui_element_color(ui_button_id, Vector4::new(0.7, 0.7, 0.7, 1.0));
            }
            
            // Re-enable depth testing for 3D rendering
            unsafe {
                gl::Enable(gl::DEPTH_TEST);
                gl::Disable(gl::BLEND);
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
                InputEvent::KeyPressed(Key::Tab) => {
                    show_ui = !show_ui; // Toggle UI visibility
                    if show_ui {
                        window.unlock_cursor();
                        sensitivity = 0.0;
                    } else {
                        window.lock_cursor();
                        sensitivity = 0.002;
                    }
                }
                InputEvent::KeyPressed(Key::Q) => {
                    camera.cycle_mode();
                }
                InputEvent::MouseButtonPressed(CLICKS::Left) => {
                    if show_ui {
                        // UI input is handled in world.update()
                        println!("UI active - mouse input handled by UI system");
                    } else {
                        println!("pewpew: {:#?}", player.get_position());
                    }
                }
                _ => {}
            }
        }

        // This here really demonstates the issue with this because like this should not go after the rendering
        while let Some(event) = input_system.get_event_queue().pop_front() {
            match event {
                InputEvent::KeyPressed(Key::Lctrl) => {
                    player.speed = 0.3;
                }
                InputEvent::KeyReleased(Key::Lctrl) => {
                    player.speed = 0.1;
                }
                InputEvent::KeyPressed(Key::Tab) => {
                    window.unlock_cursor();
                    sensitivity = 0.000;
                }
                InputEvent::KeyReleased(Key::Tab) => {
                    window.lock_cursor();
                    sensitivity = 0.002;
                }
                InputEvent::KeyPressed(Key::Q) => {
                    camera.cycle_mode();
                }
                InputEvent::KeyReleased(Key::Q) => {

                }
                InputEvent::MouseButtonPressed(CLICKS::Left) => {
                    println!("pewpew: {:#?}", player.get_position());
                    
                    // if ui_manager.is_element_hovered(3) {
                    //     ui_manager.visit_element(3, &mut visitor);
                    // }
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
        
        // Add a test motion to the teddy bear
        // if let Some(coords) = world.movement.get_coords_mut(teddy_entity.id) {
        //     coords.position.y = 5.0 * f32::sin(time);
        // }

        if let Some(coords) = world.movement.get_coords_mut(triangle_entity.id) {
            coords.position = *player.get_position();
        }

        window.update();
        time += 0.1;
    }
}

fn setup_ui_system(world: &mut World, texture_id: u32) -> (u32, u32, u32, u32, u32) {
    // Create main menu container
    let main_menu = world.create_ui_container(
        "main_menu",
        Vector2::new(0.0, 0.0),
        Vector2::new(720.0, 720.0),
        UILayout::grid(2, 110.0).with_padding(40.0),
    );

    let title_text = world.create_ui_text(
        "title",
        Vector2::new(0.0, 0.0), // Will be positioned by container
        "Game Menu".to_string(),
        32.0,
    );
    world.add_ui_child(main_menu.id, title_text.id);
    
    // Create UI elements as children of the container
    let ui_element1 = world.create_entity("ui_element1");
    world.ui.add_transform(ui_element1.id, UITransform::new(Vector2::new(100.0, 300.0), Vector2::new(200.0, 100.0)));
    world.ui.add_style(ui_element1.id, UIStyle::new().with_texture(texture_id));
    //world.add_ui_child(main_menu.id, ui_element1.id);
    
    let ui_element2 = world.create_entity("ui_element2");
    world.ui.add_transform(ui_element2.id, UITransform::new(Vector2::new(0.0, 0.0), Vector2::new(100.0, 100.0)));
    world.ui.add_style(ui_element2.id, UIStyle::new().with_color(Vector4::new(1.0, 0.0, 0.0, 1.0)));
    world.add_ui_child(main_menu.id, ui_element2.id);
    
    let ui_button = world.create_ui_button(
        "ui_button",
        Vector2::new(0.0, 0.0),
        Vector2::new(200.0, 160.0),
        "Click Me!".to_string(),
    );
    world.add_ui_child(main_menu.id, ui_button.id);
    

    let info_text = world.create_ui_text(
        "info_text",
        Vector2::new(0.0, 0.0),
        "Press Tab to toggle UI\nUse WASD to move".to_string(),
        14.0,
    );
    world.add_ui_child(main_menu.id, info_text.id);
    
    // Return IDs for later reference
    (main_menu.id, ui_element1.id, ui_element2.id, ui_button.id, info_text.id)
}