use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};
use cgmath::Vector3;
use crate::graphics::materials::Material;
// use crate::graphics::texture_manager::TextureManager;
// use crate::graphics::gl_wrapper::ShaderManager;

use super::mesh::Mesh;
use super::transform::WorldCoords;

// pub trait Model {
//     fn render(&self, material: &Material, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
//         // material.apply(shader_manager, texture_manager, &self.world_coords.get_model_matrix());
//         // //println!("HERE");
//         // self.mesh.draw();
//     }
// }

//raaah idk why I did this by hand
pub fn load_obj(file_path: &str) -> Mesh {
    let file = File::open(file_path).expect("Failed to open OBJ file");
    let reader = BufReader::new(file);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut vertices = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {//Todo use resultr
                let x: f32 = parts[1].parse().unwrap();//? better then unwrap sometimes
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                positions.push(Vector3::new(x, y, z));
            }
            "vn" => { //normals
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                normals.push(Vector3::new(x, y, z));
            }
            "f" => {
                for i in 1..=3 {
                    let vertex_data: Vec<&str> = parts[i].split('/').collect();
                    let vertex_idx: usize = vertex_data[0].parse().unwrap();

                    let pos = positions[vertex_idx - 1];

                    let norm = if vertex_data.len() > 2 && !vertex_data[2].is_empty() {
                        let normal_idx: usize = vertex_data[2].parse().unwrap();
                        normals[normal_idx - 1]
                    } else {
                        Vector3::new(0.0, 0.0, 0.0)
                    };

                    vertices.extend_from_slice(&[pos.x, pos.y, pos.z, norm.x, norm.y, norm.z]);
                    indices.push(indices.len() as i32);
                }
            }
            _ => {}
        }
    }

    Mesh::new(&vertices, &indices)
}

//ok now... I should probably... PROBABLY have new be a function in the trait
//but idk if I need to change later I will

pub trait ModelTrait {
    fn get_mesh(&self) -> &Mesh;
    fn get_world_coords(&self) -> &WorldCoords;
    fn get_material(&self) -> Arc<RwLock<Material>>;
    fn attach_to(&mut self, parent: &WorldCoords);
    fn detach(&mut self);
    //maybe render func the only thing is that some need texture manager some dont 
    //fn render(&self);
}

pub struct Model{
    mesh: Mesh,//eeeh just make these public later lowkey maybe
    world_coords: WorldCoords,
    material: Arc<RwLock<Material>>,
}

impl Model {
    pub fn new_no_coords(mesh: Mesh, material: &Arc<RwLock<Material>>) -> Model{//this is useless
        Model {
            mesh: mesh,
            world_coords: WorldCoords::new_empty(),
            material: Arc::clone(material)
        }
    }

    pub fn new(mesh: Mesh, coords: WorldCoords, material: Arc<RwLock<Material>>) -> Model{//this is useless
        Model {
            mesh: mesh,
            world_coords: coords,
            material: material
        }
    }

    pub fn new_pass_cloned_arc(mesh: Mesh, material: Arc<RwLock<Material>>) -> Model{//this one is way more useful
        Model {
            mesh: mesh,
            world_coords: WorldCoords::new_empty(),
            material: material
        }
    }
}

impl ModelTrait for Model {
    fn get_mesh(&self) -> &Mesh{
        &self.mesh
    }

    fn get_world_coords(&self) -> &WorldCoords{
        &self.world_coords
    }

    fn get_material(&self) -> Arc<RwLock<Material>>{
        self.material.clone()//so I can pass a ref here but I think since ARC is already ref just pass another arc
    }

    //some might not want parents so just over ride the base to have parent
    //in cases which you want to have them
    fn attach_to(&mut self, _parent: &WorldCoords) {
        //self.parent = Some(parent as *const WorldCoords);
    }

    fn detach(&mut self) {
        //self.parent = None;
    }
}