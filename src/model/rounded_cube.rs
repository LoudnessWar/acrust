use std::sync::{Arc, RwLock};

use cgmath::*;
//use crate::graphics::gl_wrapper::{Vao, BufferObject, ShaderManager};
use crate::graphics::materials::Material;
use crate::graphics::texture_manager::TextureManager;
//use crate::graphics::texture_manager::TextureManager;


use super::objload::{Model, ModelTrait};
use super::transform::WorldCoords; // Import WorldCoords
use super::mesh::Mesh;

pub struct RoundedCube {
    base: Model,
}

impl RoundedCube {
    pub fn new(size_x: f32, size_y: f32, size_z: f32, position: Vector3<f32>, rotation: f32, material: Arc<RwLock<Material>>) -> Self {

        let corner_verticies = 8;
        let edge_verticies = (size_x as usize + size_y as usize + size_z as usize - 3) * 4;
        let face_verticies = (
            (size_x as usize - 1) * (size_y as usize - 1) +
            (size_y as usize - 1) * (size_z as usize - 1) +
            (size_z as usize - 1) * (size_x as usize - 1)
        ) * 2;

        let mut vertices: Vec<Vector3<i32>> = Vec::with_capacity(//how do  spell verticies vertices ...
            corner_verticies as usize + edge_verticies as usize + face_verticies as usize
        );

        let mut v = 0;


        for y in 0..=size_y as i32 {
            //this ppart right here is like if you are drawling a square on a paper and dont pick up your pen like you start bottm left then draw to the right
            for x in 0..=size_x as i32 {
                vertices.push(Vector3::new(x, y, 0));
            }
            //then you draw up
            for z in 1..=size_z as i32 {
                vertices.push(Vector3::new(size_x as i32, y, z));
            }
            //then you go to the left
            for x in (0..size_x as i32).rev() {
                vertices.push(Vector3::new(x, y, size_z as i32));
            }
            //and finally you go back and down and connect to your start
            for z in (1..size_z as i32).rev() {
                vertices.push(Vector3::new(0, y, z));
            }
            //then since its 3d we can move up a layer and go again like 3d printing or something
        }

        //now to fill in the top
        for z in 1..size_z as i32 { //not inclusive because we have already drawn the edges
            for x in 1..size_x as i32 {
                vertices.push(Vector3::new(x, size_y as i32, z));
            }
        }
        //now the bottom
        for z in 1..size_z as i32 { // again not inclusive because we have already drawn the edges
            for x in 1..size_x as i32 {
                vertices.push(Vector3::new(x, 0, z));
            }
        }


        // let quads = (size_x * size_y + size_y * size_z + size_z * size_x) * 2.0;
        let quads = (size_x * size_y + size_x * size_z + size_y * size_z) * 2.0;
        println!("quad count {}", quads);
        let mut triangels = vec![0; (quads as i32 * 6) as usize];

        let ring = ((size_x + size_z) * 2.0) as i32;//this is to incriment to the next row its the size of one loop around the cube on likea 2d plane
        let mut t = 0;
        let mut v = 0;

        println!("Ring size: {}", ring);
        println!("Expected vertices per ring: {}", (size_x as i32 + 1) + size_z as i32 + size_x as i32 + (size_z as i32 - 1));

        
        println!("\n=== VERTEX DEBUG ===");
        println!("First ring (y=0):");
        for i in 0..ring as usize {
            println!("  v[{}] = {:?}", i, vertices[i]);
        }
        println!("\nSecond ring (y=1):");
        for i in ring as usize..(ring * 2) as usize {
            println!("  v[{}] = {:?}", i, vertices[i]);
        }
        println!("\nLast few vertices (top/bottom fill):");
        let start = vertices.len().saturating_sub(10);
        for i in start..vertices.len() {
            println!("  v[{}] = {:?}", i, vertices[i]);
        }

        println!("Vertex count: {}", vertices.len());
        println!("Triangle array size: {}", triangels.len());
        println!("Expected triangles to write: {}", quads as i32 * 6);

        for y in 0..size_y as i32 { 
            for q in 0..ring-1{

                t = RoundedCube::set_quad(&mut triangels, t, v, v+1, v + ring, v + ring + 1);
                v += 1;
            }
            t = RoundedCube::set_quad(&mut triangels, t, v, v - ring + 1, v + ring, v + 1);//sets us back to the first one because we are creating indicies in for the verticies we made we get all the way around and
        //then we try to incriment by one again, that corrisponds to the next row up not the first verticie preventing the completion of a proper loop, so this sets v back to the start by doing - ring then adds one so its back
        //at the start again because v will actually be one short
            v += 1;
        }

        // t = RoundedCube::create_top_face(&mut triangels, t, ring, size_x, size_y, size_z);
        // t = RoundedCube::create_bottom_face(&mut triangels, t, ring, size_x, size_y, size_z, (vertices.len()) as i32);

        println!("t after rings: {}", t);
        t = RoundedCube::create_top_face(&mut triangels, t, ring, size_x, size_y, size_z);
        println!("t after top face: {}", t);
        t = RoundedCube::create_bottom_face(&mut triangels, t, ring, size_x, size_y, size_z, vertices.len() as i32);
        println!("t after bottom face: {}", t);

        println!("\nFirst 6 triangle indices: {:?}", &triangels[0..6]);
        println!("Last 6 triangle indices: {:?}", &triangels[570..576]);
        println!("Top face first 6 indices (at 384): {:?}", &triangels[384..390]);
        println!("Top face last 6 indices: {:?}", &triangels[570..576]);
        println!("Bottom face first 6 indices (at 486): {:?}", &triangels[486..492]);
        println!("Top face indices 384-390: {:?}", &triangels[384..390]);
        println!("Top face indices 450-456: {:?}", &triangels[450..456]);
        println!("Top face indices 468-474: {:?}", &triangels[468..474]);
        println!("Top face indices 474-480: {:?}", &triangels[474..480]);
        println!("Bottom face indices 480-486: {:?}", &triangels[480..486]);
        println!("Bottom face indices 486-492: {:?}", &triangels[486..492]);
        println!("Bottom face indices 570-576: {:?}", &triangels[570..576]);

        println!("\nKey vertices to check:");
        println!("v[64] (top ring start): {:?}", vertices[64]);
        println!("v[69] (vMax): {:?}", vertices[69]);
        println!("v[79] (vMin): {:?}", vertices[79]);
        println!("v[80] (vMid, first top interior): {:?}", vertices[80]);
        println!("v[88] (last top interior): {:?}", vertices[88]);

        println!("\nFirst top face quad vertices:");
        println!("v={} at {:?}", 64, vertices[64]);
        println!("v+1={} at {:?}", 65, vertices[65]);
        println!("v+ring-1={} at {:?}", 79, vertices[79]);
        println!("v+ring={} at {:?}", 80, vertices[80]);

        // Count vertex usage
        let mut vertex_usage = std::collections::HashMap::new();
        for &idx in &triangels {
            *vertex_usage.entry(idx).or_insert(0) += 1;
        }

        // Print vertices used most frequently
        let mut usage_vec: Vec<_> = vertex_usage.iter().collect();
        usage_vec.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

        println!("\nTop 10 most-used vertices:");
        for (vertex_idx, count) in usage_vec.iter().take(10) {
            println!("  Vertex {} at {:?} used {} times", 
                vertex_idx, 
                vertices[**vertex_idx as usize], 
                count);
        }

        println!("\nVertices used MORE than 6 times (suspicious):");
        for (vertex_idx, count) in &vertex_usage {
            if *count > 6 {
                println!("  Vertex {} at {:?} used {} times", 
                    vertex_idx, 
                    vertices[*vertex_idx as usize], 
                    count);
            }
        }

        println!("\nVertices used LESS than 4 times (might be edge cases):");
        for (vertex_idx, count) in &vertex_usage {
            if *count < 4 {
                println!("  Vertex {} at {:?} used {} times", 
                    vertex_idx, 
                    vertices[*vertex_idx as usize], 
                    count);
            }
        }

        let mut verts: Vec<f32> = vertices
            .iter()
            .flat_map(|k| [k.x as f32, k.y as f32, k.z as f32])
            .collect();

        let mesh = Mesh::new_normals(&mut verts, &triangels);
        let world_coords = WorldCoords::new(position.x, position.y, position.z, rotation);
        let model = Model::new(mesh, world_coords, material);
        RoundedCube { base: model }
    }

    pub fn set_quad(tringles: &mut Vec<i32>, t: usize, v00: i32, v10: i32, v01: i32, v11: i32 ) -> usize{
                //print!("triangles: {:#?}", tringles);
                if t + 5 >= tringles.len() {
                    println!("ERROR: About to overflow! t={}, tringles.len()={}", t, tringles.len());
                    println!("v00={}, v10={}, v01={}, v11={}", v00, v10, v01, v11);
                    panic!("Stopping here");
                }
                tringles[t] = v00;
                tringles[t + 1] = v01;
                tringles[t + 2] = v10;
                tringles[t + 3] = v10;
                tringles[t + 4] = v01;
                tringles[t + 5] = v11;
                t + 6
    }

    pub fn set_quad_reversed(tringles: &mut Vec<i32>, t: usize, v00: i32, v10: i32, v01: i32, v11: i32) -> usize {
        tringles[t] = v00;
        tringles[t + 1] = v10;  // Swapped with v01
        tringles[t + 2] = v01;  // Swapped with v10
        tringles[t + 3] = v01;
        tringles[t + 4] = v10;
        tringles[t + 5] = v11;
        t + 6
    }

    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    pub fn create_top_face(tringles: &mut Vec<i32>, mut t: usize, ring: i32, size_x: f32, size_y: f32, size_z: f32) -> usize{
        let mut v = ring * size_y as i32;
        println!("\n=== TOP FACE DEBUG ===");
        println!("v initial: {}", v);
        for x in 0..size_x as i32 - 1 {
            t = RoundedCube::set_quad(tringles, t, v, v+1, v + ring - 1, v + ring);
            v+=1;
        }
        t = RoundedCube::set_quad(tringles, t, v, v+1, v + ring - 1, v + 2);

        let mut vMin = ring * (size_y as i32 + 1) -1;
        let mut vMid = vMin + 1;
        let mut vMax = v + 2;

        println!("vMin: {}", vMin);
        println!("vMid: {}", vMid);
        println!("vMax: {}", vMax);

        for z in 1 .. size_z as i32 -1 {
            t = RoundedCube::set_quad(tringles, t, vMin, vMid, vMin - 1, vMid + size_x as i32 - 1);
            for x in 1..size_x as i32 -1 {
                t = RoundedCube::set_quad(tringles, t, vMid, vMid + 1, vMid + size_x as i32 - 1, vMid + size_x as i32);
                vMid += 1;
            }
            t = RoundedCube::set_quad(tringles, t, vMid, vMax, vMid + size_x as i32 - 1, vMax + 1);
            vMid += 1;
            vMin -= 1;
            vMax += 1;
        }

        let mut vTop = vMin - 2;
        t = RoundedCube::set_quad(tringles, t, vMin, vMid, vTop + 1, vTop);

        for x in 1..size_x as i32 - 1 {
            t = RoundedCube::set_quad(tringles, t, vMid, vMid + 1, vTop, vTop - 1);
            vTop -= 1;
            vMid += 1;
        }
        println!("Last top quad: vMid={}, vTop-2={}, vTop={}, vTop-1={}", vMid, vTop - 2, vTop, vTop - 1);
        t = RoundedCube::set_quad(tringles, t, vMid, vTop - 2, vTop, vTop - 1);
        println!("Top face ACTUAL last 6 indices (at 474): {:?}", &tringles[474..480]);
        t
    }

    #[allow(unused_variables)]
    #[allow(non_snake_case)]
    pub fn create_bottom_face(tringles: &mut Vec<i32>, mut t: usize, ring: i32, size_x: f32, size_y: f32, size_z: f32, len: i32) -> usize{
        let mut v = 1;
        println!("\n=== Bottom FACE DEBUG ===");
        println!("v initial: {}", v);
        let mut vMid = len - (size_x as i32 - 1) * (size_z as i32 - 1);
        t = RoundedCube::set_quad(tringles, t, ring - 1, vMid, 0, 1);
        for x in 1..size_x as i32 - 1{
            t = RoundedCube::set_quad(tringles, t, vMid, vMid + 1, v, v + 1);
            vMid += 1;
            v+=1;
        }
        t = RoundedCube::set_quad(tringles, t, vMid, v + 2, v, v + 1);

        let mut vMin = ring - 2;
        vMid -= size_x as i32 - 2;
        let mut vMax = v + 2;

        
        println!("vMin: {}", vMin);
        println!("vMid: {}", vMid);
        println!("vMax: {}", vMax);

        for z in 1 .. size_z as i32 -1 {
            t = RoundedCube::set_quad(tringles, t, vMin, vMid + size_x as i32 - 1, vMin + 1, vMid);
            for x in 1..size_x as i32 -1 {
                t = RoundedCube::set_quad(tringles, t, vMid + size_x as i32 - 1, vMid + size_x as i32, vMid, vMid + 1);
                vMid += 1;
            }
            t = RoundedCube::set_quad(tringles, t, vMid + size_x as i32 - 1, vMax + 1, vMid, vMax);
            vMid += 1;
            vMin -= 1;
            vMax += 1;
        }

        let mut vTop = vMin - 1;
        t = RoundedCube::set_quad(tringles, t, vTop + 1, vTop, vTop + 2, vMid);

        for x in 1..size_x as i32 - 1 {
            t = RoundedCube::set_quad(tringles, t, vTop, vTop - 1, vMid, vMid + 1);
            vMid += 1;
            vTop -= 1;
        }
        println!("Last bottom quad: vTop={}, vTop-1={}, vMid={}, vTop-2={}", vTop, vTop - 1, vMid, vTop - 2);
        t = RoundedCube::set_quad(tringles, t, vTop, vTop - 1, vMid, vTop - 2);

        t
    }


    //uuggghhhh Idk if like
    //Arc and Rwlock was the right move guys
    //this would be better actually though if I just passed material manager
    pub fn render(&self, texture_manager: &TextureManager) {
        self.get_material().read().unwrap().apply_no_model(texture_manager);
        self.get_mesh().draw();
    }
}

impl ModelTrait for RoundedCube{
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