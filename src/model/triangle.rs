use std::sync::{Arc, RwLock};
use cgmath::*;
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
use super::objload::{Model, ModelTrait};
use super::transform::WorldCoords;
use super::mesh::Mesh;

pub struct Triangle {
    base: Model,
}

impl Triangle {
    pub fn new(size: f32, position: Vector3<f32>, rotation: f32, material: Arc<RwLock<Material>>) -> Self {
        // Vertices with positions and normals (normal pointing up in Z direction)
        let vertices: [f32; 18] = [
            // Positions         // Normals
            -size + position.x, -size + position.y, position.z,  0.0, 0.0, 1.0,  // bottom left
             size + position.x, -size + position.y, position.z,  0.0, 0.0, 1.0,  // bottom right
             0.0 + position.x,  size + position.y, position.z,  0.0, 0.0, 1.0,   // top middle
        ];

        let indices: [i32; 3] = [
            0, 1, 2  // Single triangle
        ];

        let mesh = Mesh::new(&vertices, &indices);
        let world_coords = WorldCoords::new(position.x, position.y, position.z, rotation);
        let model = Model::new(mesh, world_coords, material);
        Triangle { base: model }
    }

    pub fn render(&self, texture_manager: &TextureManager) {
        self.get_material().read().unwrap().apply_no_model(texture_manager);
        self.get_mesh().draw();
    }
}

impl ModelTrait for Triangle {
    fn get_mesh(&self) -> &Mesh {
        self.base.get_mesh()
    }

    fn get_world_coords(&self) -> &WorldCoords {
        self.base.get_world_coords()
    }

    fn get_material(&self) -> Arc<RwLock<Material>> {
        self.base.get_material()
    }

    fn attach_to(&mut self, parent: &WorldCoords) {
        // Implementation if needed
    }

    fn detach(&mut self) {
        // Implementation if needed
    }
}