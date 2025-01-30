// use cgmath::{Matrix4, Vector3, Vector4};
// use crate::graphics::gl_wrapper::*;
// use crate::graphics::materials::Material;
// use crate::graphics::gl_wrapper::Vao;
// use crate::graphics::gl_wrapper::BufferObject;
// use cgmath::Quaternion;
// use cgmath::Rotation3;

// use super::transform::WorldCoords; // Import the WorldCoords struct

// pub struct Cube {
//     id: u32,
//     transform: WorldCoords, // Use WorldCoords for transformations
//     length: f32,
//     width: f32,
//     height: f32,
//     vertices: Vec<f32>,
//     indices: Vec<i32>,
//     //material: &Material,//hmmm I think I might need to rewrite Material so that each shader holds all the materials that it holds
// }

// impl Cube {
//     pub fn new(id: u32, position: Vector3<f32>, length: f32, width: f32, height: f32) -> Self {
//         let (vertices, indices) = Self::generate_cube_vertices(position, length, width, height);

//         Cube {
//             id,
//             transform: WorldCoords::new(position.x, position.y, position.z, 0.0), // Initialize WorldCoords
//             length,
//             width,
//             height,
//             vertices,
//             indices: indices.iter().map(|&i| i as i32).collect(),//uuuh collect uuuuuh
//             //material,
//         }
//     }

//     pub fn generate_cube_vertices(position: Vector3<f32>, length: f32, width: f32, height: f32) -> (Vec<f32>, Vec<u32>) {
//         let x = position.x;
//         let y = position.y;
//         let z = position.z;

//         let half_length = length / 2.0;
//         let half_width = width / 2.0;
//         let half_height = height / 2.0;

//         let vertices = vec![
//             // Front face
//             x - half_length, y - half_width, z + half_height, 0.0, 0.0, 1.0, // Front-bottom-left
//             x + half_length, y - half_width, z + half_height, 0.0, 0.0, 1.0, // Front-bottom-right
//             x + half_length, y + half_width, z + half_height, 0.0, 0.0, 1.0, // Front-top-right
//             x - half_length, y + half_width, z + half_height, 0.0, 0.0, 1.0, // Front-top-left

//             // Back face
//             x - half_length, y - half_width, z - half_height, 0.0, 0.0, -1.0, // Back-bottom-left
//             x + half_length, y - half_width, z - half_height, 0.0, 0.0, -1.0, // Back-bottom-right
//             x + half_length, y + half_width, z - half_height, 0.0, 0.0, -1.0, // Back-top-right
//             x - half_length, y + half_width, z - half_height, 0.0, 0.0, -1.0, // Back-top-left

//             // Left face
//             x - half_length, y - half_width, z - half_height, -1.0, 0.0, 0.0, // Back-bottom-left
//             x - half_length, y - half_width, z + half_height, -1.0, 0.0, 0.0, // Front-bottom-left
//             x - half_length, y + half_width, z + half_height, -1.0, 0.0, 0.0, // Front-top-left
//             x - half_length, y + half_width, z - half_height, -1.0, 0.0, 0.0, // Back-top-left

//             // Right face
//             x + half_length, y - half_width, z + half_height, 1.0, 0.0, 0.0, // Front-bottom-right
//             x + half_length, y - half_width, z - half_height, 1.0, 0.0, 0.0, // Back-bottom-right
//             x + half_length, y + half_width, z - half_height, 1.0, 0.0, 0.0, // Back-top-right
//             x + half_length, y + half_width, z + half_height, 1.0, 0.0, 0.0, // Front-top-right

//             // Top face
//             x - half_length, y + half_width, z + half_height, 0.0, 1.0, 0.0, // Front-top-left
//             x + half_length, y + half_width, z + half_height, 0.0, 1.0, 0.0, // Front-top-right
//             x + half_length, y + half_width, z - half_height, 0.0, 1.0, 0.0, // Back-top-right
//             x - half_length, y + half_width, z - half_height, 0.0, 1.0, 0.0, // Back-top-left

//             // Bottom face
//             x - half_length, y - half_width, z - half_height, 0.0, -1.0, 0.0, // Back-bottom-left
//             x + half_length, y - half_width, z - half_height, 0.0, -1.0, 0.0, // Back-bottom-right
//             x + half_length, y - half_width, z + half_height, 0.0, -1.0, 0.0, // Front-bottom-right
//             x - half_length, y - half_width, z + half_height, 0.0, -1.0, 0.0, // Front-bottom-left
//         ];

//         let indices = vec![
//             0, 1, 2, 0, 2, 3, // Front
//             6, 5, 4, 7, 6, 4, // Back
//             7, 4, 0, 3, 7, 0, // Left
//             1, 5, 6, 1, 6, 2, // Right
//             3, 2, 6, 3, 6, 7, // Top
//             5, 1, 0, 4, 5, 0, // Bottom
//         ];

//         (vertices, indices)
//     }

//     pub fn render(&self, shader: &ShaderProgram, view_projection_matrix: &Matrix4<f32>) {
//         shader.bind();
//         shader.set_matrix4fv_uniform("model", &self.transform.get_model_matrix()); // Use WorldCoords for the model matrix
//         shader.set_matrix4fv_uniform("viewProjection", view_projection_matrix);

//         let vao = Vao::new();
//         vao.bind();

//         let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
//         vbo.bind();
//         vbo.store_f32_data(&self.vertices);

//         let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
//         ebo.bind();
//         ebo.store_i32_data(&self.indices);

//         unsafe {
//             gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<f32>() as i32, std::ptr::null());
//             gl::EnableVertexAttribArray(0);

//             gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<f32>() as i32, (3 * std::mem::size_of::<f32>()) as *const _);
//             gl::EnableVertexAttribArray(1);

//             gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
//         }

//         vao.unbind();
//     }

//     // Delegate transformation methods to WorldCoords
//     pub fn set_position(&mut self, position: Vector3<f32>) {
//         self.transform.set_position(position);
//     }

//     pub fn get_position(&self) -> Vector3<f32> {
//         self.transform.get_position()
//     }

//     pub fn translate(&mut self, translation: Vector3<f32>) {
//         self.transform.position += translation;
//     }

//     pub fn rotate(&mut self, angle: f32, axis: Vector3<f32>) {
//         self.transform.rotation = self.transform.rotation * Quaternion::from_axis_angle(axis, cgmath::Rad(angle));
//     }

//     pub fn scale(&mut self, scale: Vector3<f32>) {
//         self.transform.scale = scale;
//     }
// }

use cgmath::*;
use crate::graphics::gl_wrapper::{Vao, BufferObject, ShaderManager};
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
use gl::types::*;


// //lol maybe I can just recycle the like cube map from the skybox
// pub struct Cube<'a> {
//     vao: Vao,
//     vbo: BufferObject,
//     ebo: BufferObject,
//     material: &'a Material,
//     transform: Matrix4<f32>,
// }

// impl<'a> Cube<'a> {
//     /// Creates a new cube with a given size, position, and material
//     pub fn new(size: f32, position: Vector3<f32>, material: &'a Material) -> Self {
//         let vertices: [f32; 24] = [
//             // Positions      
//             -size, -size, -size, 
//              size, -size, -size, 
//              size,  size, -size, 
//             -size,  size, -size, 
//             -size, -size,  size, 
//              size, -size,  size, 
//              size,  size,  size, 
//             -size,  size,  size,
//         ];

//         let indices: [i32; 36] = [
//             0, 1, 2, 2, 3, 0,  // Front face
//             1, 5, 6, 6, 2, 1,  // Right face
//             5, 4, 7, 7, 6, 5,  // Back face
//             4, 0, 3, 3, 7, 4,  // Left face
//             3, 2, 6, 6, 7, 3,  // Top face
//             4, 5, 1, 1, 0, 4   // Bottom face
//         ];

//         let vao = Vao::new();
//         let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
//         let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);

//         vao.bind();
//         vbo.bind();
//         vbo.store_f32_data(&vertices);
        
//         ebo.bind();
//         ebo.store_i32_data(&indices);

//         let transform = Matrix4::from_translation(position);

//         Cube {
//             vao,
//             vbo,
//             ebo,
//             material,
//             transform,
//         }
//     }

//     /// Translates the cube
//     pub fn translate(&mut self, translation: Vector3<f32>) {
//         self.transform = Matrix4::from_translation(translation) * self.transform;
//     }

//     /// Scales the cube
//     pub fn scale(&mut self, factor: f32) {
//         self.transform = Matrix4::from_scale(factor) * self.transform;
//     }

//     /// Rotates the cube around an axis
//     pub fn rotate(&mut self, angle: f32, axis: Vector3<f32>) {
//         self.transform = Matrix4::from_axis_angle(axis.normalize(), Rad(angle)) * self.transform;
//     }

//     /// Renders the cube using its material
//     pub fn render(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
//         self.vao.bind();
//         self.material.apply(shader_manager, texture_manager, &self.transform);
//         unsafe {
//             gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, std::ptr::null());
//         }
//     }
// }


use cgmath::*;
//use crate::graphics::gl_wrapper::Material;
//use crate::graphics::gl_wrapper::ShaderManager;
//use crate::graphics::gl_wrapper::TextureManager;
//use crate::graphics::gl_wrapper::Mesh;
//use crate::graphics::renderable::Renderable;
use super::transform::WorldCoords; // Import WorldCoords
use super::mesh::Mesh;

pub struct Cube {
    mesh: Mesh,
    //material: &'a Material,
    world_coords: WorldCoords, // Uses WorldCoords for transformations
}

impl Cube {
    pub fn new(size: f32, position: Vector3<f32>, rotation: f32) -> Self {
        let vertices: [f32; 48] = [
            // Positions         // Normals
            -size, -size, -size,  0.0,  0.0, 1.0,  
             size, -size, -size,  0.0,  0.0, 1.0,  
             size,  size, -size,  0.0,  0.0, 1.0,  
            -size,  size, -size,  0.0,  0.0, 1.0,  
            -size, -size,  size,  0.0,  0.0,  -1.0,  
             size, -size,  size,  0.0,  0.0,  -1.0,  
             size,  size,  size,  0.0,  0.0,  -1.0,  
            -size,  size,  size,  0.0,  0.0,  -1.0,  
        ];

        let indices: [i32; 36] = [
            0, 2, 1, 2, 0, 3,  
            1, 6, 5, 6, 1, 2,  
            5, 7, 4, 7, 5, 6,  
            4, 3, 0, 3, 4, 7,  
            3, 6, 2, 6, 3, 7,  
            4, 1, 5, 1, 4, 0   
        ];

        let mesh = Mesh::new(&vertices, &indices);
        let world_coords = WorldCoords::new(position.x, position.y, position.z, rotation);

        Cube { mesh, world_coords }
    }

    /// BRO
    /// what is all this for really like really
    pub fn move_forward(&mut self, distance: f32) {
        let forward = self.world_coords.get_forward_vector();
        self.world_coords.move_forward(forward, distance);
    }

    pub fn move_backward(&mut self, distance: f32) {
        let forward = self.world_coords.get_forward_vector();
        self.world_coords.move_backward(forward, distance);
    }

    pub fn move_left(&mut self, distance: f32) {
        let left = self.world_coords.get_left_vector();
        self.world_coords.move_left(left, distance);
    }

    pub fn move_right(&mut self, distance: f32) {
        let left = self.world_coords.get_left_vector();
        self.world_coords.move_right(left, distance);
    }

    pub fn scale(&mut self, factor: f32) {
        self.world_coords.scale *= factor;
    }

    // pub fn rotate_y(&mut self, angle: f32) {
    //     self.world_coords.rotation *= Quaternion::from_angle_y(Rad(angle));
    // }

    pub fn render(&self, material: &Material, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
        material.apply(shader_manager, texture_manager, &self.world_coords.get_model_matrix());
        //println!("HERE");
        self.mesh.draw();
    }
}


//like this is fucking useless like immma prolly keep on doing it like this, but like comme eoooonn
// impl<'a> Renderable for Cube<'a> {
//     fn render(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
//         self.material.apply(shader_manager, texture_manager, &self.world_coords.get_model_matrix());
//         self.mesh.draw();
//     }
// }
