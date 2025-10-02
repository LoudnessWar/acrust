// use std::rc::Rc;

// use crate::graphics::gl_wrapper::*;
// use crate::graphics::lightManager::*;

// struct SceneObject {
//     vao: VAO,
//     index_count: usize,
// }

// fn initialize_scene() -> Vec<SceneObject> {
//     vec![
//         // TODO: Initialize your VAOs and index counts here
//         // SceneObject { vao: ..., index_count: ... },
//     ]
// }

// fn initialize_depth_shader() -> ShaderProgram {//i could make this dynamic but like bruh
//     ShaderProgram::new("shaders/depth_prepass.vert","shaders/depth_prepass.frag")
// }

// fn render_frame(
//     scene: &[SceneObject],
//     depth_shader: &ShaderProgram,
//     light_manager: &mut LightManager,
//     width: u32,
//     height: u32,
// ) {
//     let framebuffer = Framebuffer::new_depth_only(width, height);

//     let vao_data: Vec<(&VAO, usize)> = scene.iter().map(|obj| (&obj.vao, obj.index_count)).collect();

//     run_depth_prepass(
//         depth_shader,
//         &framebuffer,
//         &vao_data,
//         light_manager,
//         width,
//         height,
//     );

//     // Add lighting pass / forward+ rendering next
// }

// fn main() {
//     // Setup window, GL context

//     let mut light_manager = LightManager::new();
//     let depth_shader = initialize_depth_shader();
//     let scene = initialize_scene();

//     let window_width = 1280;
//     let window_height = 720;

//     loop {
//         // Input handling...

//         render_frame(
//             &scene,
//             &depth_shader,
//             &mut light_manager,
//             window_width,
//             window_height,
//         );

//         // Swap buffers, etc...
//     }
// }
