use std::collections::HashMap;
use cgmath::*;
use super::gl_wrapper::{ShaderManager, ShaderProgram, UniformValue};
use super::texture_manager::TextureManager;
use gl::types::*;

/// Material struct now only references a shader by name instead of storing a ShaderProgram instance.
/// 
/// 
/// ok I have an issue with the current implimentation...
/// that beign all the uniforms are like not references when I beleive it wopuld be best if they were,
/// I should take insperation from the old one and have another fucntion for uniforms that are borrowed
/// 
/// 
/// one thing I do not like about this new implimentation is that any shader with a material must but in the
/// shader manager like I cant have a free shader material like how skybox was.. this would be benefitial for single
/// use shaders
/// 
/// maybe for some things keeps a ref to the shader here to make it faster
/// 
/// raah need a spot for stuff to not be cloned
pub struct Material {
    shader_name: String, // Reference to shader stored in ShaderManager
    uniforms: HashMap<String, UniformValue>,
    texture_names: HashMap<String, String>, // Maps uniform name to texture file path
}

impl Material {
    pub fn new(shader_name: &str) -> Self {
        Material {
            shader_name: shader_name.to_string(),
            uniforms: HashMap::new(),
            texture_names: HashMap::new(),
        }
    }

    /// Set a uniform variable (float, vector, matrix)
    pub fn set_uniform(&mut self, shader_manager: &mut ShaderManager, name: &str, value: UniformValue) {
        if let Some(shader) = shader_manager.get_shader_mut(&self.shader_name) {
            shader.create_uniform(name); // Ensure uniform exists
            self.uniforms.insert(name.to_string(), value);
        } else {
            eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
        }
    }

    pub fn set_texture(&mut self, shader_manager: &mut ShaderManager, uniform_name: &str, texture_file: &str) {
        if let Some(shader) = shader_manager.get_shader_mut(&self.shader_name) {
            shader.create_uniform(uniform_name); // Ensure uniform exists
            self.texture_names.insert(uniform_name.to_string(), texture_file.to_string());
        } else {
            eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
        }
    }

    pub fn init_uniform(&mut self, shader_manager: &mut ShaderManager, name: &str) {
        if let Some(shader) = shader_manager.get_shader_mut(&self.shader_name) {
            shader.create_uniform(name); // Ensure uniform exists
            self.uniforms.insert(name.to_string(), UniformValue::Empty());
        } else {
            eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
        }
    }

    //i am really not a fan of how this needs so much to funciton like oh no everthing needs models and not everything needs textures and raaah itst annoying
    pub fn apply(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager, model_matrix: &Matrix4<f32>) {
        if let Some(shader) = shader_manager.get_shader(&self.shader_name) {
            shader.bind();
            shader.set_matrix4fv_uniform("model", model_matrix);

            // Apply stored uniforms
            for (name, value) in &self.uniforms {
                match value {
                    UniformValue::Float(f) => shader.set_uniform1f(name, *f),
                    UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
                    UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
                    _ => {}
                }
            }

            // Bind textures
            let mut texture_unit = 0;
            for (uniform_name, texture_path) in &self.texture_names {
                if let Some(texture_id) = texture_manager.get_texture(texture_path) {
                    unsafe {
                        gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
                        gl::BindTexture(gl::TEXTURE_2D, texture_id);
                    }
                    shader.set_uniform1i(uniform_name, &(texture_unit as i32));
                    texture_unit += 1;
                } else {
                    eprintln!("Warning: Texture '{}' not found!", texture_path);
                }
            }
        } else {
            eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
        }
    }

    pub fn apply_no_model(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
        if let Some(shader) = shader_manager.get_shader(&self.shader_name) {
            shader.bind();

            // Set uniforms
            for (name, value) in &self.uniforms {
                match value {
                    UniformValue::Float(f) => shader.set_uniform1f(name, *f),
                    UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
                    UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
                    _ => {}
                }
            }

            // Bind textures
            let mut texture_unit = 0;
            for (uniform_name, texture_path) in &self.texture_names {
                if let Some(texture_id) = texture_manager.get_texture(texture_path) {
                    unsafe {
                        gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
                        gl::BindTexture(gl::TEXTURE_2D, texture_id);
                    }
                    shader.set_uniform1i(uniform_name, &(texture_unit as i32));
                    texture_unit += 1;
                } else {
                    eprintln!("Warning: Texture '{}' not found!", texture_path);
                }
            }
        } else {
            eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
        }
    }

    pub fn apply_no_texture(&self, shader_manager: &ShaderManager, model_matrix: &Matrix4<f32>) {
        if let Some(shader) = shader_manager.get_shader(&self.shader_name) {
            shader.bind();
            shader.set_matrix4fv_uniform("model", model_matrix);

            // Apply stored uniforms
            for (name, value) in &self.uniforms {
                match value {
                    UniformValue::Float(f) => shader.set_uniform1f(name, *f),
                    UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
                    UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
                    _ => {}
                }
            }
        } else {
            eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
        }
    }

    // pub fn set_float_property(&mut self, key: &str, value: f32, shader_manager: &ShaderManager) {
    //     self.set_uniform(key, UniformValue::Float(value));
    // }

    // pub fn set_vector4_property(&mut self, key: &str, value: Vector4<f32>) {
    //     self.set_uniform(key, UniformValue::Vector4(value));
    // }

    pub fn set_matrix4fv_uniform(&mut self, manager: &mut ShaderManager, key: &str, value: Matrix4<f32>) {
        self.set_uniform(manager, key, UniformValue::Matrix4(value));
    }

    // pub fn set_matrix4_property(&mut self, key: &str, value: Matrix4<f32>) {
    //     self.set_uniform(key, UniformValue::Matrix4(value));
    // }
}
