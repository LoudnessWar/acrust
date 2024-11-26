//use acrust::custom_errors::Errors;
use acrust::graphics::window::Window;
use acrust::graphics::camera::Camera;
use acrust::input::input::{InputSystem, InputEvent, Key};
use acrust::graphics::gl_wrapper::*;

//use gl::types::*;
//use std::mem;
//use std::ptr;
use cgmath::*;
//use std::env;

mod voxel_render;

fn main() {
    let mut window = Window::new(720, 720, "CUBE!", 60);
    window.init_gl();


    let mut input_system = InputSystem::new();//need to make this so that it is like added on window init or something

    let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
    shader_program.bind();

    shader_program.create_uniform("transform");

    shader_program.enable_depth();//BRQO I had this commented out and I could not for the life of me figure out why depth buffering was not working 😭😭😭

    //camera innit herm yeah might do this diff idk
    let perspective = PerspectiveFov {
        fovy: Rad(1.0), // Field of view (vertical)
        aspect: 1.0,    // Aspect ratio
        near: 0.1,      // Near clipping plane
        far: 100.0,     // Far clipping plane
    };

    let mut camera = Camera::new(perspective);

    // Initialize Voxel Renderer
    let mut voxel_renderer = voxel_render::VoxelRenderer::new();

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
        camera.rotation.y -= (-delta_x as f32) * sensitivity; // Invert X to match common FPS controls
        camera.rotation.x -= (-delta_y as f32) * sensitivity;

        //TODO: tihs is a little broken rn
        camera.rotation.x = camera.rotation.x.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);


        window.process_input_events(&mut input_system);
        if input_system.is_key_pressed(&Key::W) {
            camera.move_forward(0.1);
        }
        if input_system.is_key_pressed(&Key::S) {
            camera.move_backward(0.1);
        }
        if input_system.is_key_pressed(&Key::A) {
            camera.move_left(0.1);
        }
        if input_system.is_key_pressed(&Key::D) {
            camera.move_right(0.1);
        }
        
        camera.update_view();
        
    
        while let Some(event) = input_system.get_event_queue().pop_front() {
            match event {
                InputEvent::KeyPressed(Key::Space) => {
                    println!("Jump");
                }
                _ => {}
            }
        }

        let transform = camera.get_vp_matrix();
        shader_program.set_matrix4fv_uniform("transform", &transform);
        

        window.update();
    }

}


// fn old3() {
//     //old2();
//     //acrust::logger::init();

//     let mut window = Window::new(720, 720, "CUBE!", 60);
//     window.init_gl();

//     //window.set_swap_interval(1);

//     let mut input_system = InputSystem::new();//need to make this so that it is like added on window init or something

//     let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
//     shader_program.bind();

//     shader_program.create_uniform("transform");
//     // shader_program.create_uniform("time");

//     let mut transform = cgmath::Matrix4::from_angle_y(cgmath::Rad(1.0));
//     shader_program.set_matrix4fv_uniform("transform", &transform);

//     // Initialize Voxel Renderer
//     let mut voxel_renderer = voxel_render::VoxelRenderer::new();

//     //later on TODO add an actual like camera but for now I will do it the simple way my pygame iso was perf implimentation maybe will copy that later
//     //let mut camera_position = cgmath::Vector3::new(0.0, 0.0, 5.0);


//     let mut angle: f32 = 0.0;

//     while !window.should_close() {
//         // Clear the screen
//         unsafe {
//             gl::ClearColor(0.3, 0.3, 0.3, 1.0);
//             gl::Clear(gl::COLOR_BUFFER_BIT);
//         }
//         voxel_renderer.render();
        
//         //print!("out");
//         // transform = cgmath::Matrix4::from_angle_y(cgmath::Rad(angle));
//         // shader_program.set_matrix4fv_uniform("transform", &transform);
        

//         // angle += 0.01;

//         // Process input events ok so... This all needs to be added to update in the window function...Eventually
//         window.process_input_events(&mut input_system);

//         //last
//         while let Some(event) = input_system.get_event_queue().pop_front() {
//             match event {
//                 InputEvent::KeyPressed(Key::W) => println!("Move forward"),
//                 InputEvent::KeyReleased(Key::W) => println!("Stop forward"),
//                 InputEvent::KeyPressed(Key::Space) => println!("Jump"),
//                 //InputEvent::MouseMoved(x, y) => println!("Mouse moved to: {}, {}", x, y),
//                 InputEvent::MouseButtonPressed(button) => println!("Mouse button {} pressed", button),
//                 InputEvent::MouseButtonReleased(button) => println!("Mouse button {} released", button),
//                 _ => {}
//             }
//         }

//         window.update();
//     }




//     pub fn rolling_ball_test(){
        
//     }
// }

// fn old2() {
//     acrust::logger::init();

//     let mut window = Window::new(720, 720, "SQUARE!");
//     window.init_gl();

//     //set up the shaders
//     let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");
//     shader_program.bind();

//     shader_program.create_uniform("transform");

//     let transform = cgmath::Matrix4::from_scale(1.0);
//     shader_program.set_matrix4fv_uniform("transform", &transform);

//     //ShaderProgram::unbind();

//     //stopping shaders and now doing verticies and triangles this is like jsut a cube rn

//     let verticies: [f32; 12]= [
//         -0.5, -0.5, 0.0,
//         0.5, -0.5, 0.0,
//         0.5, 0.5, 0.0,
//         -0.5, 0.5, 0.0,
//     ];

//     let indices: [i32; 6]= [
//         0, 1, 2, 
//         2, 3, 0
//     ];

//     //initializing buffers and such
//     let vao = acrust::graphics::gl_wrapper::Vao::new();
//     vao.bind();

//     let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
//     vbo.bind();
//     vbo.store_f32_data(&verticies);

//     let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
//     ebo.bind();
//     ebo.store_i32_data(&indices);

//     //semi important how we render
//     let position_attribute = VertexAttribute::new(
//         0,
//         3,
//         gl::FLOAT,
//         gl::FALSE,
//         3 * mem::size_of::<GLfloat>() as GLsizei,
//         ptr::null(),
//     );

//     position_attribute.enable();

//     let mut input_system = InputSystem::new();


//     //game loop here so all 
//     while !window.should_close(){
//         //this is to just draw the stuff lol
//         unsafe{
//             gl::ClearColor(0.3, 0.7, 0.3, 1.0);
//             gl::Clear(gl::COLOR_BUFFER_BIT);
//             gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null()); //triangles ig
//         }
        
//         //this is initilizing the input system tbh I maybe should just initilize this in windows bc I have having to like init a bunch of stuff but its whatever ig
//         window.process_input_events(&mut input_system);

//         //this just goes throught the quequueueu of like inputs
//         while let Some(event) = input_system.get_event_queue().pop_front() {
//             match event {
//                 InputEvent::KeyPressed(Key::W) => println!("Move forward"),
//                 InputEvent::KeyReleased(Key::W) => println!("Stop forward"),
//                 InputEvent::KeyPressed(Key::Space) => println!("Jump"),
//                 InputEvent::MouseMoved(x, y) => println!("Mouse moved to: {}, {}", x, y),
//                 InputEvent::MouseButtonPressed(button) => {
//                     println!("Mouse button {} pressed", button)
//                 }
//                 _ => {println!("Mouse button fail")}
//             }
//         }

//         //ig this goes last BAKA
//         window.update();
//     }
// }



// ///not used this is like a shitty version of version contorl 😭😭😭
// fn old() {
//     println!("Hello, world!");
//     //println!("Current working directory: {:?}", env::current_dir().unwrap());
//     acrust::logger::init();
//     //error_test(1);

//     let mut window = Window::new(720, 720, "CUBE!");
//     window.init_gl();

//     //type vertex = [f32; 3];//hmmm now like does this having it put and wrapped in a 2d array make it more complicated
//     //no need because you just end up unwrapping this and... there is not a lot of circumstances where I will be
//     //hand writing the verticies

//     // type TriIndexes = [u32; 3];

//     let mut shader_program = ShaderProgram::new("shaders/vertex_shader.glsl", "shaders/fragment_shader.glsl");

//     // Use the ShaderProgram
//     shader_program.bind();

//     // Set uniforms, if needed
//     shader_program.create_uniform("transform");

//     let transform = cgmath::Matrix4::from_scale(1.0);
//     shader_program.set_matrix4fv_uniform("transform", &transform);

//     // Unbind the ShaderProgram
//     ShaderProgram::unbind();


//     // let verticies: [f32; 9]= [
//     //     -0.5, -0.5, 0.5,
//     //     0.5, -0.5, 0.0,
//     //     0.0, 0.5, 0.0,
//     // ];

//     // let verticies: [f32; 12]= [
//     //     -0.5, -0.5, 0.0,
//     //     0.5, -0.5, 0.0,
//     //     0.5, 0.5, 0.0,
//     //     -0.5, 0.5, 0.0,
//     // ];

//     let verticies: [f32; 24] = [
//     -0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
//      0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
//      0.5,  0.5, 0.0, 0.0, 0.0, 1.0,
//     -0.5,  0.5, 0.0, 1.0, 1.0, 0.0,
//     ];

//     let indices: [i32; 6]= [
//         0, 1, 2, 
//         2, 3, 0
//     ];

//     let vao = acrust::graphics::gl_wrapper::Vao::new();
//     vao.bind();

//     let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
//     vbo.bind();

//     vbo.store_f32_data(&verticies);

//     let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
//     ebo.bind();
//     ebo.store_i32_data(&indices);

//     // let position_attribute = VertexAttribute::new(
//     //     0,
//     //     3,
//     //     gl::FLOAT,
//     //     gl::FALSE,
//     //     3 * mem::size_of::<GLfloat>() as GLsizei,
//     //     ptr::null(),
//     // );

//     // position_attribute.enable();

//     let position_attribute = VertexAttribute::new(
//         0,
//         3,
//         gl::FLOAT,
//         gl::FALSE,
//         6 * mem::size_of::<GLfloat>() as GLsizei, // Stride: 6 floats per vertex
//         ptr::null(),
//     );
    
//     position_attribute.enable();
    
//     let color_attribute = VertexAttribute::new(
//         1,
//         3,
//         gl::FLOAT,
//         gl::FALSE,
//         6 * mem::size_of::<GLfloat>() as GLsizei, // Stride: 6 floats per vertex
//         (3 * mem::size_of::<GLfloat>()) as *const GLvoid, // Offset: 3 floats into the vertex data
//     );
    
//     color_attribute.enable();

//     while !window.should_close(){
//         unsafe{
//             gl::ClearColor(0.3, 0.7, 0.3, 1.0);
//             gl::Clear(gl::COLOR_BUFFER_BIT);
//             //gl::DrawArrays(gl::TRIANGLES, 0, 3)
//             gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
//         }
//         window.update();
//     }
// }

// fn error_test(num: i32) -> Result<(), Errors> {
//     if num == 1{
//         acrust::logger::info!("Error");
//         return Err(Errors::TestError.into());
//     }
//     Ok(())
// }