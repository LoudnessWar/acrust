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
            // Positions         // Normals or color? it was color btw which i like literally dont want
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

    fn attach_to(&mut self, parent: &WorldCoords) {
        //self.parent = Some(parent as *const WorldCoords);
    }

    fn detach(&mut self) {
        //self.parent = None;
    }
}
