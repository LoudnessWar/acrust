use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

use cgmath::vec4;//crate
use crate::Matrix4;
use acrust::graphics::camera::Camera;
use acrust::model::transform::WorldCoords;
use cgmath::One;
use cgmath::Vector3;
use acrust::graphics::materials::Material;
//use cgmath::{Matrix4, Vector3, Deg, Point3};

pub struct WaterRender {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
    material: Material, // Link the material
    position: WorldCoords,
}

impl WaterRender {
    pub fn new(
        length: f32,
        width: f32,
        quadSize: f32,
        shader: &str, // Pass the shader program to initialize the material
    ) -> Self {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<i32> = Vec::new();
        let mut offset = 0;
        let mut uvs = Vec::new();//literally not even used

        for i in 0..length as i32
        {
            for j in 0..width as i32
            {
                vertices.extend(vec![j as f32 * quadSize, 0.0, i as f32 * quadSize]);
                uvs.extend(vec![j as f32 / width, i as f32 / width]);
            }
        }

        for i in 0..length as i32 {
            for j in 0..width as i32 {
                let start = i * (width as i32 + 1) + j;
                let next_row = start + width as i32 + 1;

                indices.push(start);
                indices.push(next_row);
                indices.push(start + 1);

                indices.push(start + 1);
                indices.push(next_row);
                indices.push(next_row + 1);
            }
        }

        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&vertices);

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW);
        ebo.bind();
        ebo.store_i32_data(&indices);

        let stride = 3 * mem::size_of::<GLfloat>() as GLsizei;//this uuuh is fine...nah but i am curious
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();
        // Create a material and initialize uniforms
        let mut material = Material::new(shader);
        //material.init_uniform("model");
        // material.init_uniform("waterColor");//raaah I want to add back init so that there is just less checks
        // material.init_uniform("waveSpeed");
        // material.init_uniform("waveScale");
        // material.init_uniform("timeFactor");
        // material.init_uniform("waveHeight");
        // material.init_uniform("lightPosition");
        // material.init_uniform("lightColor");
        // material.init_uniform("view");
        // material.init_uniform("projection");
        // material.init_uniform("model");
        //material.init_uniform("lightIntensity");
        //println!("HERE!@");

        WaterRender {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            material,
            position: WorldCoords::new_empty(),
        }
    }

    pub fn render(&mut self, smanager: &mut ShaderManager, time: f32, camera: &Camera) {
        self.vao.bind();

        
        // Set uniform values
        self.material.set_uniform(smanager, "waterColor", UniformValue::Vector4(vec4(0.0, 0.5, 1.0, 0.5)));
        self.material.set_uniform(smanager, "waveSpeed", UniformValue::Float(1.0));
        self.material.set_uniform(smanager, "waveScale", UniformValue::Float(0.1));
        self.material.set_uniform(smanager, "timeFactor", UniformValue::Float(time * 0.1));
        self.material.set_uniform(smanager, "waveHeight", UniformValue::Float(0.5));
        self.material.set_uniform(smanager, "lightPosition", UniformValue::Vector4(vec4(0.0, 10.0, 0.0, 1.0)));
        self.material.set_uniform(smanager, "lightColor", UniformValue::Vector4(vec4(0.0, 1.0, 1.0, 1.0)));
        //self.material.set_float_property("lightIntensity", 10.0);
        // Set transform matrix
        //self.material.set_matrix4fv_uniform("transform", transform);

        // Set camera matrices
        self.material.set_matrix4fv_uniform(smanager, "view", camera.get_view().clone());
        //self.material.set_matrix4fv_uniform("model", &self.position.get_model_matrix());
        self.material.set_matrix4fv_uniform(smanager, "projection",camera.get_p_matrix().clone());
        self.material.init_uniform(smanager, "model");

        // Apply the material
        self.material.apply_no_texture(smanager, &self.position.get_model_matrix());
        // Draw the water surface
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    pub fn set_position(&mut self, new_position: Vector3<f32>){
        self.position.set_position(new_position);
    }

    pub fn get_position(&self) -> Vector3<f32>{
        self.position.get_position()
    }
}
