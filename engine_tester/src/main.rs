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

    // UI setup - kept separate from ECS
    let mut ui_manager = UIManager::new(720.0, 720.0);

    let mut ui_element = UIElement::new(1, Vector2::new(50.0, 50.0), Vector2::new(200.0, 100.0));
    ui_element.set_texture(texture_id);

    let mut ui_element2 = UIElement::new(2, Vector2::new(90.0,90.0), Vector2::new(100.0, 100.0));    
    ui_element2.set_color(Vector4::new(1.0, 0.0, 0.0, 1.0));

    let mut ui_draggable = UIDraggable::new(4, Vector2::new(120.0,120.0), Vector2::new(200.0, 200.0));    
    ui_draggable.set_color(Vector4::new(0.0, 1.0, 1.0, 1.0));

    let mut ui_button = Button::new(3, Vector2::new(400.0,90.0), Vector2::new(200.0, 100.0));  
    ui_element2.set_color(Vector4::new(1.0, 1.0, 0.0, 1.0));

    let mut ui_text = UIText::new(
        5,
        Vector2::new(300.0, 300.0),
        Vector2::new(200.0, 50.0),
        "Hello, world! \nmy name is jeff".to_string(),
        24.0,
    );
    ui_manager.add_element(Box::new(ui_text));
    
    ui_manager.add_element(Box::new(ui_element));
    ui_manager.add_element(Box::new(ui_element2));
    ui_manager.add_element(Box::new(ui_button));
    ui_manager.add_element(Box::new(ui_draggable));

    window.lock_cursor();
    let mut sensitivity = 0.002;

    let mut visitor = ClickVisitor::new();
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
    let mut world = World::new();
    
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
        world.update(delta_time);
        
        // This is like 3 funcitons deep at this point world -> render -> fpr -> five different fucntions
        world.render(&mut fpr, &camera, 720, 720, &texture_manager);
        //OOOOKKKKAAAYYYYYYY SOOOOOOOO... IF IT DOESNT HAPPEN AFTER HERE IT DONT HAPPEN LOOKING AT YOU UI NEED TO FIX

        //BRO ITS SO SLOW HELP!!! I think I was lowkey just adding stuff back 

        let view_matrix = skybox.get_skybox_view_matrix(&camera.get_view());
        let projection_matrix = camera.get_p_matrix();
        skybox.render(&mut skybox_material, &texture_manager, &view_matrix, &projection_matrix);

        // UI handling
        if input_system.is_key_pressed(&Key::Tab) {
            ui_manager.update(current_mouse_position);
            ui_manager.render(&ui_shader);

            let mut text_visitor = TextRenderVisitor { text_renderer: &text_renderer };
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::Disable(gl::DEPTH_TEST); // Optional: if text is hidden behind UI
            }
            let mut text_visitor = TextRenderVisitor { text_renderer: &text_renderer };
            ui_manager.visit_all(&mut text_visitor);

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

        // let mut text_visitor = TextRenderVisitor { text_renderer: &text_renderer };
        //     ui_manager.visit_all(&mut text_visitor);

        // OMG HAAIIIII Process UI events the event system to be ony used with the ui because i was lazy HAAIIIIII
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
                    
                    if ui_manager.is_element_hovered(3) {
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

// ClickVisitor implementation remains unchanged
pub struct ClickVisitor {
    pub button_clicked: bool,
}

impl ClickVisitor {
    pub fn new() -> Self {
        Self {
            button_clicked: false,
        }
    }
}

impl UIElementVisitor for ClickVisitor {
    fn visit_button(&mut self, button: &mut Button, is_clicked: bool) {
        if is_clicked {
            self.button_clicked = true;
            println!("Button clicked: ID {}", button.get_id());
            button.set_position(button.get_position() + Vector2::new(10.0, 0.0));
        }
    }

    fn visit_slider(&mut self, slider: &mut Slider) {
        println!("Slider value: {}", slider.get_value());
    }

    fn visit_text(&mut self, text: &mut UIText) {//todo make this do something maybe idk
        println!("Text content: {}", text.get_text());
    }
}