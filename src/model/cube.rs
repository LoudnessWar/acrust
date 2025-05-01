use std::sync::{Arc, RwLock};

use cgmath::*;
//use crate::graphics::gl_wrapper::{Vao, BufferObject, ShaderManager};
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
//use crate::graphics::texture_manager::TextureManager;


use super::objload::{Model, ModelTrait};
use super::transform::WorldCoords; // Import WorldCoords
use super::mesh::Mesh;

pub struct Cube {
    base: Model,
}

impl Cube {
    pub fn new(size: f32, position: Vector3<f32>, rotation: f32, material: Arc<RwLock<Material>>) -> Self {
        let vertices: [f32; 48] = [//ok so thins cube is weird like most things if no back face and depth its inverse but still whatever
            // Positions         // Normals or color... it depends on the like shaders... Theoretically though these should just be normals
            -size + position.x, -size + position.y, -size + position.z,  1.0,  0.0, 1.0,  
             size + position.x, -size + position.y, -size + position.z,  1.0,  1.0, 1.0,  
             size + position.x,  size + position.y, -size + position.z,  0.0,  0.0, 1.0,  
            -size + position.x,  size + position.y, -size + position.z,  0.0,  0.0, 1.0,  
            -size + position.x, -size + position.y,  size + position.z,  0.0,  1.0,  -1.0,  
             size + position.x, -size + position.y,  size + position.z,  0.0,  1.0,  -1.0,  
             size + position.x,  size + position.y,  size + position.z,  0.0,  0.0,  -1.0,  
            -size + position.x,  size + position.y,  size + position.z,  1.0,  1.0,  1.0,  
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
        let model = Model::new(mesh, world_coords, material);
        Cube { base: model }
    }

    //uuggghhhh Idk if like
    //Arc and Rwlock was the right move guys
    //this would be better actually though if I just passed material manager
    pub fn render(&self, texture_manager: &TextureManager) {
        self.get_material().read().unwrap().apply_no_model(texture_manager);
        self.get_mesh().draw();
    }
}

impl ModelTrait for Cube{
    fn get_mesh(&self) -> &Mesh{
        self.base.get_mesh()
    }

    fn get_world_coords(&self) -> &WorldCoords{
        self.base.get_world_coords()
    }

    fn get_material(&self) -> Arc<RwLock<Material>>{
        self.base.get_material()
    }

    fn set_position(&mut self, position: Vector3<f32>){//ok so not everything needs to move but it just will make life easier if its here... I think?
        //we can have a seperate thing for moving camera ig
        self.base.set_position(position);
    }

    // fn attach_to(&mut self, parent: &WorldCoords) {
    //     //self.parent = Some(parent as *const WorldCoords);
    // }

    // fn detach(&mut self) {
    //     //self.parent = None;
    // }
}




// pub struct Cube {
//     mesh: Mesh,
//     material: Arc<RwLock<Material>,//uuuuuhhhh like I hate this yo
//     world_coords: WorldCoords, // Uses WorldCoords for transformations
// }

// impl Cube {
//     pub fn new(size: f32, position: Vector3<f32>, rotation: f32) -> Self {
//         let vertices: [f32; 48] = [
//             // Positions         // Normals
//             -size, -size, -size,  0.0,  0.0, 1.0,  
//              size, -size, -size,  0.0,  0.0, 1.0,  
//              size,  size, -size,  0.0,  0.0, 1.0,  
//             -size,  size, -size,  0.0,  0.0, 1.0,  
//             -size, -size,  size,  0.0,  0.0,  -1.0,  
//              size, -size,  size,  0.0,  0.0,  -1.0,  
//              size,  size,  size,  0.0,  0.0,  -1.0,  
//             -size,  size,  size,  0.0,  0.0,  -1.0,  
//         ];

//         let indices: [i32; 36] = [
//             0, 2, 1, 2, 0, 3,  
//             1, 6, 5, 6, 1, 2,  
//             5, 7, 4, 7, 5, 6,  
//             4, 3, 0, 3, 4, 7,  
//             3, 6, 2, 6, 3, 7,  
//             4, 1, 5, 1, 4, 0   
//         ];

//         let mesh = Mesh::new(&vertices, &indices);
//         let world_coords = WorldCoords::new(position.x, position.y, position.z, rotation);

//         Cube { mesh, world_coords }
//     }

//     /// BRO
//     /// what is all this for really like really
//     pub fn move_forward(&mut self, distance: f32) {
//         let forward = self.world_coords.get_forward_vector();
//         self.world_coords.move_forward(forward, distance);
//     }

//     pub fn move_backward(&mut self, distance: f32) {
//         let forward = self.world_coords.get_forward_vector();
//         self.world_coords.move_backward(forward, distance);
//     }

//     pub fn move_left(&mut self, distance: f32) {
//         let left = self.world_coords.get_left_vector();
//         self.world_coords.move_left(left, distance);
//     }

//     pub fn move_right(&mut self, distance: f32) {
//         let left = self.world_coords.get_left_vector();
//         self.world_coords.move_right(left, distance);
//     }

//     pub fn scale(&mut self, factor: f32) {
//         self.world_coords.scale *= factor;
//     }

//     // pub fn rotate_y(&mut self, angle: f32) {
//     //     self.world_coords.rotation *= Quaternion::from_angle_y(Rad(angle));
//     // }

//     pub fn render(&self, material: &Material, texture_manager: &TextureManager) {
//         material.apply(texture_manager, &self.world_coords.get_model_matrix());
//         //println!("HERE");
//         self.mesh.draw();
//     }
// }


// //like this is fucking useless like immma prolly keep on doing it like this, but like comme eoooonn
// // impl<'a> Renderable for Cube<'a> {
// //     fn render(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
// //         self.material.apply(shader_manager, texture_manager, &self.world_coords.get_model_matrix());
// //         self.mesh.draw();
// //     }
// // }
