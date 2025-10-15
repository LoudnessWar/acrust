use std::sync::{Arc, RwLock};

use cgmath::*;
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;


use super::objload::{Model, ModelTrait};
use super::transform::WorldCoords; // Import WorldCoords
use super::mesh::Mesh;

pub struct Sphere {
    base: Model,
}

impl Sphere {
    pub fn new(size: f32, position: Vector3<f32>, rotation: f32, material: Arc<RwLock<Material>>) -> Self {


        let vertices: [f32; 24 * 6] = [
            // Front face
            -size + position.x, -size + position.y,  size + position.z,   0.0, 0.0, 1.0,
            size + position.x, -size + position.y,  size + position.z,   0.0, 0.0, 1.0,
            size + position.x,  size + position.y,  size + position.z,   0.0, 0.0, 1.0,
            -size + position.x,  size + position.y,  size + position.z,   0.0, 0.0, 1.0,

            // Back face
            -size + position.x, -size + position.y, -size + position.z,   0.0, 0.0, -1.0,
            size + position.x, -size + position.y, -size + position.z,   0.0, 0.0, -1.0,
            size + position.x,  size + position.y, -size + position.z,   0.0, 0.0, -1.0,
            -size + position.x,  size + position.y, -size + position.z,   0.0, 0.0, -1.0,

            // Right face
            size + position.x, -size + position.y, -size + position.z,   1.0, 0.0, 0.0,
            size + position.x,  size + position.y, -size + position.z,   1.0, 0.0, 0.0,
            size + position.x,  size + position.y,  size + position.z,   1.0, 0.0, 0.0,
            size + position.x, -size + position.y,  size + position.z,   1.0, 0.0, 0.0,

            // Left face
            -size + position.x, -size + position.y, -size + position.z,  -1.0, 0.0, 0.0,
            -size + position.x,  size + position.y, -size + position.z,  -1.0, 0.0, 0.0,
            -size + position.x,  size + position.y,  size + position.z,  -1.0, 0.0, 0.0,
            -size + position.x, -size + position.y,  size + position.z,  -1.0, 0.0, 0.0,

            // Top face
            -size + position.x,  size + position.y, -size + position.z,   0.0, 1.0, 0.0,
            size + position.x,  size + position.y, -size + position.z,   0.0, 1.0, 0.0,
            size + position.x,  size + position.y,  size + position.z,   0.0, 1.0, 0.0,
            -size + position.x,  size + position.y,  size + position.z,   0.0, 1.0, 0.0,

            // Bottom face
            -size + position.x, -size + position.y, -size + position.z,   0.0, -1.0, 0.0,
            size + position.x, -size + position.y, -size + position.z,   0.0, -1.0, 0.0,
            size + position.x, -size + position.y,  size + position.z,   0.0, -1.0, 0.0,
            -size + position.x, -size + position.y,  size + position.z,   0.0, -1.0, 0.0,
        ];

        let indices: [i32; 36] = [
            // Front face
            0, 1, 2,  2, 3, 0,
            // Back face
            4, 6, 5,  6, 4, 7,
            // Right face
            8, 9,10, 10,11, 8,
            // Left face
            12,14,13, 14,12,15,
                // Top face
            16,18,17, 18,16,19,
                // Bottom face
            20,21,22, 22,23,20
        ];

        let mesh = Mesh::new(&vertices, &indices);
        let world_coords = WorldCoords::new(position.x, position.y, position.z, rotation);
        let model = Model::new(mesh, world_coords, material);
        Sphere { base: model }
    }

    //uuggghhhh Idk if like
    //Arc and Rwlock was the right move guys
    //this would be better actually though if I just passed material manager
    pub fn render(&self, texture_manager: &TextureManager) {
        self.get_material().read().unwrap().apply_no_model(texture_manager);
        self.get_mesh().draw();
    }
}

impl ModelTrait for Sphere{
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

    fn set_rotation(&mut self, rotation: f32) {
        self.base.set_rotation(rotation);
    }

    fn set_rotation_from_quaternion(&mut self, rotation: Quaternion<f32>) {
        self.base.set_rotation_from_quaternion(rotation);
    }

}

