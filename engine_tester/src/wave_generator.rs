use acrust::graphics::gl_wrapper::*;
use gl::types::*;
use std::mem;
use std::ptr;

use cgmath::vec4;//crate
use crate::Matrix4;

pub struct WaterRender {
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    index_count: i32,
    material: Material, // Link the material
}

impl WaterRender {
    pub fn new(
        length: f32,
        width: f32,
        quad_size: f32,
        shader: ShaderProgram, // Pass the shader program to initialize the material
    ) -> Self {
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<i32> = Vec::new();

        for i in 0..length as i32 {
            for j in 0..width as i32 {
                vertices.extend(vec![j as f32 * quad_size, 0.0, i as f32 * quad_size]);
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

        let stride = 3 * mem::size_of::<GLfloat>() as GLsizei; // Position only
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();

        // Create a material and initialize uniforms
        let mut material = Material::new(shader);
        material.add_uniform("waterColor");
        material.add_uniform("waveSpeed");
        material.add_uniform("waveScale");
        material.add_uniform("timeFactor");
        material.add_uniform("waveHeight");
        material.add_uniform("lightPosition");
        material.add_uniform("lightColor");
        material.add_uniform("transform");

        WaterRender {
            vao,
            vbo,
            ebo,
            index_count: indices.len() as i32,
            material,
        }
    }

    pub fn render(&self, transform: &Matrix4<f32>, time: f32) {
        self.vao.bind();

        // Set uniform values
        self.material.set_property("waterColor", vec4(0.0, 0.5, 1.0, 0.5));
        self.material.set_property("waveSpeed", 1.0);
        self.material.set_property("waveScale", 0.1);
        self.material.set_property("timeFactor", time);
        self.material.set_property("waveHeight", 0.1);
        self.material.set_property("lightPosition", vec4(0.0, 10.0, 0.0, 1.0));
        self.material.set_property("lightColor", vec4(1.0, 1.0, 1.0, 1.0));

        // Set transform matrix
        self.material.set_matrix4fv_uniform("transform", transform);

        // Apply the material
        self.material.apply();

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
}
