use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};
use cgmath::{Matrix4, Vector3};
use crate::graphics::camera::Camera;
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
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
            "v" => {//Todo use resultr I mean this would be nice if I grouped them in 3 bud itdk how
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
    //pick back up heres
    Mesh::new(&vertices, &indices)//thisd is kinda like eeeehhh bc no normals for mesh ig like they aint easy ios what im sayin 
}

//could I just make this like one line of code in the other one that takes a boolean then either runs new or new_normals...
//yes TODO add that later if you want. This isnt terrible either
pub fn load_obj_new_normals(file_path: &str) -> Mesh {
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
            "v" => {//Todo use resultr I mean this would be nice if I grouped them in 3 bud itdk how
                let x: f32 = parts[1].parse().unwrap();//? better then unwrap sometimes
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                positions.push(Vector3::new(x, y, z));
            }
            // "vn" => { //since no normals just comment out shrug emoji
            //     let x: f32 = parts[1].parse().unwrap();
            //     let y: f32 = parts[2].parse().unwrap();
            //     let z: f32 = parts[3].parse().unwrap();
            //     normals.push(Vector3::new(x, y, z));
            // }
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
    //pick back up heres
    Mesh::new_normals(&vertices, &indices)//thisd is kinda like eeeehhh bc no normals for mesh ig like they aint easy ios what im sayin 
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

//I need to add something like a model id or something
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
        //material.write().expect("cannot create new model due to issue with materials or something").init_uniform("model");
        
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

    //maybe add this to trait
    pub fn render(&self, texture_manager: &TextureManager) {
        self.get_material().write().expect("Cannot Render Model due").apply(texture_manager, &self.get_world_coords().get_model_matrix());
        self.get_mesh().draw();
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

pub struct GeneralModel{
    base: Model,
    pub parent: Option<*const WorldCoords>,
}

impl GeneralModel{
    pub fn new(mesh: Mesh, coords: WorldCoords, material: Arc<RwLock<Material>>) -> GeneralModel{//this is useless
        // material.write().expect("cannot create new model due to issue when writing to material").init_uniform("model");//i should make something to mass do this
        // material.write().expect("cannot create new model due to issue when writing to material").init_uniform("view");
        // material.write().expect("cannot create new model due to when writing to material").init_uniform("projection");
        // material.write().expect("cannot create new model due to when writing to material").init_uniform("lightDir");
        // material.write().expect("cannot create new model due to when writing to material").init_uniform("lightColor");
        // material.write().expect("cannot create new model due to when writing to material").init_uniform("objectColor");

        //if they are already there is that what is causing issue?
        material.write().expect("cannot create new model due to issue when writing to material").init_uniforms(vec!["model", "view", "projection", "lightDir", "lightColor", "objectColor"]); 
        GeneralModel {
            base: Model::new(mesh,  coords, material),
            parent: None,
        }
    }

    pub fn new_no_coords(mesh: Mesh, material: Arc<RwLock<Material>>) -> GeneralModel{//this one is way more useful
        material.write().expect("cannot create new model due to issue when writing to material").init_uniforms(vec!["model", "view", "projection", "lightDir", "lightColor", "objectColor"]); 
        GeneralModel {
            base: Model::new(mesh,  WorldCoords::new_empty(), material),
            parent: None,
        }
    }

    //maybe add this to trait
    //this ok like I could pass camera here but then idk like idk ok bro!!! nah the issue with that is that it is kinda restrictive... I think
    pub fn render(&self, texture_manager: &TextureManager, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").set_matrix4fv_uniform("view", view);
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").set_matrix4fv_uniform("projection", projection);
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").apply(texture_manager, &self.get_world_coords().get_model_matrix());
        self.get_mesh().draw();
    }


    //my guess rn as to why no color is becaause it is parsinbg for it but model doesnt provide it i thing 
    pub fn simple_render(&self, texture_manager: &TextureManager, camera: &Camera) {
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").set_matrix4fv_uniform("view", camera.get_view());
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").set_matrix4fv_uniform("projection", camera.get_p_matrix());
        self.get_material().write().expect("Cannot Render Model due").apply(texture_manager, &self.get_world_coords().get_model_matrix());
        self.get_mesh().draw();
    }

    pub fn set_uniforms(&self, texture_manager: &TextureManager, camera: &Camera) {
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").set_matrix4fv_uniform("view", camera.get_view());
        self.get_material().write().expect("Cannot Render Model due to issue while righting to material").set_matrix4fv_uniform("projection", camera.get_p_matrix());
        self.get_material().write().expect("Cannot Render Model due").apply(texture_manager, &self.get_world_coords().get_model_matrix());
    }
}

impl ModelTrait for GeneralModel{
    fn get_mesh(&self) -> &Mesh{
        self.base.get_mesh()
    }

    fn get_world_coords(&self) -> &WorldCoords{//need to add parental consideration to world cords look at update_view in camera for example
        self.base.get_world_coords()
    }

    fn get_material(&self) -> Arc<RwLock<Material>>{
        self.base.get_material()
    }

    fn attach_to(&mut self, parent: &WorldCoords) {
        self.parent = Some(parent as *const WorldCoords);
    }

    fn detach(&mut self) {
        self.parent = None;
    }
}
