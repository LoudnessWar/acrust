use cgmath::{Matrix4, Vector3, Vector4};
use crate::graphics::gl_wrapper::*;
use crate::graphics::material::Material;
use crate::transform::WorldCoords; // Import the WorldCoords struct

pub struct Cube {
    id: u32,
    transform: WorldCoords, // Use WorldCoords for transformations
    length: f32,
    width: f32,
    height: f32,
    vertices: Vec<f32>,
    indices: Vec<u32>,
    material: Material,
}

impl Cube {
    pub fn new(id: u32, position: Vector3<f32>, length: f32, width: f32, height: f32, material: Material) -> Self {
        let (vertices, indices) = Self::generate_cube_vertices(position, length, width, height);

        Cube {
            id,
            transform: WorldCoords::new(position.x, position.y, position.z, 0.0), // Initialize WorldCoords
            length,
            width,
            height,
            vertices,
            indices: indices.iter().map(|&i| i as u32).collect(),
            material,
        }
    }

    pub fn generate_cube_vertices(position: Vector3<f32>, length: f32, width: f32, height: f32) -> (Vec<f32>, Vec<u32>) {
        let x = position.x;
        let y = position.y;
        let z = position.z;

        let half_length = length / 2.0;
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let vertices = vec![
            // Front face
            x - half_length, y - half_width, z + half_height, 0.0, 0.0, 1.0, // Front-bottom-left
            x + half_length, y - half_width, z + half_height, 0.0, 0.0, 1.0, // Front-bottom-right
            x + half_length, y + half_width, z + half_height, 0.0, 0.0, 1.0, // Front-top-right
            x - half_length, y + half_width, z + half_height, 0.0, 0.0, 1.0, // Front-top-left

            // Back face
            x - half_length, y - half_width, z - half_height, 0.0, 0.0, -1.0, // Back-bottom-left
            x + half_length, y - half_width, z - half_height, 0.0, 0.0, -1.0, // Back-bottom-right
            x + half_length, y + half_width, z - half_height, 0.0, 0.0, -1.0, // Back-top-right
            x - half_length, y + half_width, z - half_height, 0.0, 0.0, -1.0, // Back-top-left

            // Left face
            x - half_length, y - half_width, z - half_height, -1.0, 0.0, 0.0, // Back-bottom-left
            x - half_length, y - half_width, z + half_height, -1.0, 0.0, 0.0, // Front-bottom-left
            x - half_length, y + half_width, z + half_height, -1.0, 0.0, 0.0, // Front-top-left
            x - half_length, y + half_width, z - half_height, -1.0, 0.0, 0.0, // Back-top-left

            // Right face
            x + half_length, y - half_width, z + half_height, 1.0, 0.0, 0.0, // Front-bottom-right
            x + half_length, y - half_width, z - half_height, 1.0, 0.0, 0.0, // Back-bottom-right
            x + half_length, y + half_width, z - half_height, 1.0, 0.0, 0.0, // Back-top-right
            x + half_length, y + half_width, z + half_height, 1.0, 0.0, 0.0, // Front-top-right

            // Top face
            x - half_length, y + half_width, z + half_height, 0.0, 1.0, 0.0, // Front-top-left
            x + half_length, y + half_width, z + half_height, 0.0, 1.0, 0.0, // Front-top-right
            x + half_length, y + half_width, z - half_height, 0.0, 1.0, 0.0, // Back-top-right
            x - half_length, y + half_width, z - half_height, 0.0, 1.0, 0.0, // Back-top-left

            // Bottom face
            x - half_length, y - half_width, z - half_height, 0.0, -1.0, 0.0, // Back-bottom-left
            x + half_length, y - half_width, z - half_height, 0.0, -1.0, 0.0, // Back-bottom-right
            x + half_length, y - half_width, z + half_height, 0.0, -1.0, 0.0, // Front-bottom-right
            x - half_length, y - half_width, z + half_height, 0.0, -1.0, 0.0, // Front-bottom-left
        ];

        let indices = vec![
            0, 1, 2, 0, 2, 3, // Front
            6, 5, 4, 7, 6, 4, // Back
            7, 4, 0, 3, 7, 0, // Left
            1, 5, 6, 1, 6, 2, // Right
            3, 2, 6, 3, 6, 7, // Top
            5, 1, 0, 4, 5, 0, // Bottom
        ];

        (vertices, indices)
    }

    pub fn render(&self, shader: &ShaderProgram, view_projection_matrix: &Matrix4<f32>) {
        shader.apply();
        shader.set_matrix4fv_uniform("model", &self.transform.get_model_matrix()); // Use WorldCoords for the model matrix
        shader.set_matrix4fv_uniform("viewProjection", view_projection_matrix);

        let vao = VertexArrayObject::new();
        vao.bind();

        let vbo = VertexBufferObject::new();
        vbo.bind();
        vbo.buffer_data(&self.vertices);

        let ebo = ElementBufferObject::new();
        ebo.bind();
        ebo.buffer_data(&self.indices);

        unsafe {
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<f32>() as i32, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 6 * std::mem::size_of::<f32>() as i32, (3 * std::mem::size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);

            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
        }

        vao.unbind();
    }

    // Delegate transformation methods to WorldCoords
    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.transform.set_position(position);
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.transform.get_position()
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.transform.position += translation;
    }

    pub fn rotate(&mut self, angle: f32, axis: Vector3<f32>) {
        self.transform.rotation = self.transform.rotation * Quaternion::from_axis_angle(axis, cgmath::Rad(angle));
    }

    pub fn scale(&mut self, scale: Vector3<f32>) {
        self.transform.scale = scale;
    }
}