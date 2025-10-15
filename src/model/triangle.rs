use std::sync::{Arc, RwLock};
use cgmath::*;
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
use super::objload::{Model, ModelTrait};
use super::transform::WorldCoords;
use super::transform::Coords;
use super::mesh::Mesh;

pub struct Triangle {
    base: Model,
    //pub parent: Option<*const WorldCoords>,//ok yeah so this is causing errors crazy i know
    //ok so there are two like
    //things with this/issues with not using like a child based system
    //when I update I dont want to constantly check if parent has moved
    //I just want to move the parent and that tell the child to move
    //I can get around this by moving the child which then moves the parent
    //the thing is with this is that attach to can be whatever I want it to be as a function
}

impl Triangle {
    pub fn new(size: f32, height: f32, position: Vector3<f32>, rotation: f32, material: Arc<RwLock<Material>>) -> Self {
        // Base triangle vertices
        let p0 = Vector3::new(-size + position.x, -size + position.y, position.z);
        let p1 = Vector3::new( size + position.x, -size + position.y, position.z);
        let p2 = Vector3::new( 0.0 + position.x,  size + position.y, position.z);
        let apex = Vector3::new( position.x, position.y, position.z + height);

        // Calculate face normals
        let base_normal = Vector3::new(0.0, 0.0, -1.0); // Facing down
        
        // Side 1 normal (p0, p1, apex)
        let side1_normal = (p1 - p0).cross(apex - p0).normalize();
        
        // Side 2 normal (p1, p2, apex)
        let side2_normal = (p2 - p1).cross(apex - p1).normalize();
        
        // Side 3 normal (p2, p0, apex)
        let side3_normal = (p0 - p2).cross(apex - p2).normalize();

        // Positions + normals for 4 vertices, each face will define its normals
        let vertices: [f32; 72] = [
            // Base triangle (facing down)
            p0.x, p0.y, p0.z,  base_normal.x, base_normal.y, base_normal.z,
            p1.x, p1.y, p1.z,  base_normal.x, base_normal.y, base_normal.z,
            p2.x, p2.y, p2.z,  base_normal.x, base_normal.y, base_normal.z,

            // Side 1 (p0, p1, apex)
            p0.x, p0.y, p0.z,  side1_normal.x, side1_normal.y, side1_normal.z,
            p1.x, p1.y, p1.z,  side1_normal.x, side1_normal.y, side1_normal.z,
            apex.x, apex.y, apex.z,  side1_normal.x, side1_normal.y, side1_normal.z,

            // Side 2 (p1, p2, apex)
            p1.x, p1.y, p1.z,  side2_normal.x, side2_normal.y, side2_normal.z,
            p2.x, p2.y, p2.z,  side2_normal.x, side2_normal.y, side2_normal.z,
            apex.x, apex.y, apex.z,  side2_normal.x, side2_normal.y, side2_normal.z,

            // Side 3 (p2, p0, apex)
            p2.x, p2.y, p2.z,  side3_normal.x, side3_normal.y, side3_normal.z,
            p0.x, p0.y, p0.z,  side3_normal.x, side3_normal.y, side3_normal.z,
            apex.x, apex.y, apex.z,  side3_normal.x, side3_normal.y, side3_normal.z,
        ];

        let indices: [i32; 12] = [
            0, 2, 1,  // base why the flip like i geniuenly dont understand why they need to be flipped
            3, 4, 5,  // side 1
            6, 7, 8,  // side 2
            9, 10,11  // side 3
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

    fn set_position(&mut self, position: Vector3<f32>){//ok so not everything needs to move but it just will make life easier if its here... I think?
        //we can have a seperate thing for moving camera ig
        self.base.set_position(position);//aahhh the trait is so beautiful bro
    }

    fn set_rotation(&mut self, rotation: f32) {
        self.base.set_rotation(rotation);
    }

    fn set_rotation_from_quaternion(&mut self, rotation: Quaternion<f32>) {
        self.base.set_rotation_from_quaternion(rotation);
    }

    // fn attach_to(&mut self, parent: &WorldCoords) {
    //     self.parent = Some(parent as *const WorldCoords);
    // }

    // fn detach(&mut self) {
    //     // Implementation if needed
    // }
}

//why is this seperate from modelTrait? simple
//because althought they seem to be like one in the same they serve very different purpouses
//every model had coords but not everything that has coords is a model take the camera for example or like
//idk a light or something
//also not all models move so why give them that unneeded funcitonality


// impl Coords for Triangle {
//     fn update_position(&mut self) {
//         let global_position = if let Some(parent) = self.parent {
//             let parent_transform = unsafe { &*parent };
//             parent_transform.position + self.get_world_coords().position
//         } else {
//             self.get_world_coords().position
//         };

//         self.base.set_position(global_position);
//     }
// }


// use std::sync::{Arc, RwLock};
// use cgmath::*;
// use crate::graphics::materials::Material;
// use crate::graphics::texture_manager::TextureManager;
// use super::objload::{Model, ModelTrait};
// use super::transform::WorldCoords;
// use super::mesh::Mesh;

// pub struct Triangle {
//     base: Model,
// }

// impl Triangle {
//     pub fn new(size: f32, position: Vector3<f32>, rotation: f32, material: Arc<RwLock<Material>>) -> Self {
//         // Vertices with positions and normals (normal pointing up in Z direction)
//         let vertices: [f32; 18] = [
//             // Positions         // Normals
//             -size + position.x, -size + position.y, position.z,  0.0, 0.0, 1.0,  // bottom left
//              size + position.x, -size + position.y, position.z,  0.0, 0.0, 1.0,  // bottom right
//              0.0 + position.x,  size + position.y, position.z,  0.0, 0.0, 1.0,   // top middle
//         ];

//         let indices: [i32; 3] = [
//             0, 1, 2  // Single triangle
//         ];

//         let mesh = Mesh::new(&vertices, &indices);
//         let world_coords = WorldCoords::new(position.x, position.y, position.z, rotation);
//         let model = Model::new(mesh, world_coords, material);
//         Triangle { base: model }
//     }

//     pub fn render(&self, texture_manager: &TextureManager) {
//         self.get_material().read().unwrap().apply_no_model(texture_manager);
//         self.get_mesh().draw();
//     }
// }

// impl ModelTrait for Triangle {
//     fn get_mesh(&self) -> &Mesh {
//         self.base.get_mesh()
//     }

//     fn get_world_coords(&self) -> &WorldCoords {
//         self.base.get_world_coords()
//     }

//     fn get_material(&self) -> Arc<RwLock<Material>> {
//         self.base.get_material()
//     }

//     fn attach_to(&mut self, parent: &WorldCoords) {
//         // Implementation if needed
//     }

//     fn detach(&mut self) {
//         // Implementation if needed
//     }
// }