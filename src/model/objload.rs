use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use cgmath::Vector3;
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
use crate::graphics::gl_wrapper::ShaderManager;

use super::mesh::Mesh;
use super::transform::WorldCoords;

// pub trait Model {
//     fn render(&self, material: &Material, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
//         // material.apply(shader_manager, texture_manager, &self.world_coords.get_model_matrix());
//         // //println!("HERE");
//         // self.mesh.draw();
//     }
// }

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
            "v" => { // Vertex positions
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                positions.push(Vector3::new(x, y, z));
            }
            "vn" => { // Vertex normals
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                normals.push(Vector3::new(x, y, z));
            }
            "f" => { // Faces (Triangular)
                for i in 1..=3 {
                    let vertex_data: Vec<&str> = parts[i].split('/').collect();
                    let vertex_idx: usize = vertex_data[0].parse().unwrap();

                    let pos = positions[vertex_idx - 1]; // Convert OBJ 1-based to 0-based

                    // Check if normal index exists
                    let norm = if vertex_data.len() > 2 && !vertex_data[2].is_empty() {
                        let normal_idx: usize = vertex_data[2].parse().unwrap();
                        normals[normal_idx - 1]
                    } else {
                        Vector3::new(0.0, 0.0, 0.0) // Default normal if missing
                    };

                    vertices.extend_from_slice(&[pos.x, pos.y, pos.z, norm.x, norm.y, norm.z]);
                    indices.push(indices.len() as i32); // Sequential indices
                }
            }
            _ => {} // Ignore other lines
        }
    }

    Mesh::new(&vertices, &indices)
}

pub struct Model{
    mesh: Mesh,
    world_coords: WorldCoords,
    material: Arc<Material>,
}

impl Model {
    pub fn new(mesh: Mesh, material: &Arc<Material>) -> Model{
        Model {
            mesh: mesh,
            world_coords: WorldCoords::new_empty(),
            material: Arc::clone(material)
        }
    }
    pub fn new_passClonedArc(mesh: Mesh, material: Arc<Material>) -> Model{
        Model {
            mesh: mesh,
            world_coords: WorldCoords::new_empty(),
            material: material
        }
    }
}