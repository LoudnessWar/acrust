use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
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
/// 
/// ok I think the enum could work here mainly because there are so many states of like some have textures some dont
/// yada yada for application of like the material
pub struct Material {
    shader: Arc<Mutex<ShaderProgram>>, // Reference to shader stored in ShaderManager
    uniforms: HashMap<String, UniformValue>,
    texture_names: HashMap<String, String>, // Maps uniform name to texture file path
}

impl Material {
    pub fn new(shader: Arc<Mutex<ShaderProgram>>) -> Self {
        Material {
            shader,
            uniforms: HashMap::new(),
            texture_names: HashMap::new(),
        }
    }

    pub fn new_unlocked(shader: ShaderProgram) -> Self {
        Material {
            shader: Arc::new(Mutex::new(shader)),
            uniforms: HashMap::new(),
            texture_names: HashMap::new(),
        }
    }

    pub fn new_from_name(shader: &str, smader: &ShaderManager) -> Self {
        Material {
            shader: smader.get_shader(shader).unwrap(),
            uniforms: HashMap::new(),
            texture_names: HashMap::new(),
        }
    }



    //instead of all these would an enum plus pattern mathcing be better...
    pub fn apply(&self, texture_manager: &TextureManager, model_matrix: &Matrix4<f32>) {
        let mut currShader = self.shader.lock().unwrap();
        currShader.bind();
        currShader.set_matrix4fv_uniform("model", model_matrix);//this could cause errors should check to make sure that
        //the model matrix uniform is created

        for (name, value) in &self.uniforms {
            match value {
                UniformValue::Float(f) => currShader.set_uniform1f(name, *f),
                UniformValue::Vector4(v) => currShader.set_uniform4f(name, v),
                UniformValue::Matrix4(m) => currShader.set_matrix4fv_uniform(name, m),
                _ => {}
            }
        }

        let mut texture_unit = 0;
        for (uniform_name, texture_path) in &self.texture_names {
            if let Some(texture_id) = texture_manager.get_texture(texture_path) {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
                    gl::BindTexture(gl::TEXTURE_2D, texture_id);
                }
                currShader.set_uniform1i(uniform_name, &(texture_unit as i32));
                texture_unit += 1;
            } else {
                eprintln!("Warning: Texture '{}' not found!", texture_path);
            }
        }
    }

    pub fn apply_no_model(&self, texture_manager: &TextureManager) {
        let mut currShader = self.shader.lock().unwrap();
        currShader.bind();
        //currShader.set_matrix4fv_uniform("model", model_matrix);//like uuuuh dont do it like this chekc if made

        for (name, value) in &self.uniforms {
            match value {
                UniformValue::Float(f) => currShader.set_uniform1f(name, *f),
                UniformValue::Vector4(v) => currShader.set_uniform4f(name, v),
                UniformValue::Matrix4(m) => currShader.set_matrix4fv_uniform(name, m),
                _ => {}
            }
        }

        let mut texture_unit = 0;
        for (uniform_name, texture_path) in &self.texture_names {
            if let Some(texture_id) = texture_manager.get_texture(texture_path) {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
                    gl::BindTexture(gl::TEXTURE_2D, texture_id);
                }
                currShader.set_uniform1i(uniform_name, &(texture_unit as i32));
                texture_unit += 1;
            } else {
                eprintln!("Warning: Texture '{}' not found!", texture_path);
            }
        }
    }

    pub fn apply_no_texture(&self, model_matrix: &Matrix4<f32>) {
        let mut currShader = self.shader.lock().unwrap();
        currShader.bind();
        currShader.set_matrix4fv_uniform("model", model_matrix);

        for (name, value) in &self.uniforms {
            match value {
                UniformValue::Float(f) => currShader.set_uniform1f(name, *f),
                UniformValue::Vector4(v) => currShader.set_uniform4f(name, v),
                UniformValue::Matrix4(m) => currShader.set_matrix4fv_uniform(name, m),
                _ => {}
            }
        }
    }

    /// Set a uniform variable (float, vector, matrix)
    // pub fn set_uniform(&mut self, shader_manager: &mut ShaderManager, name: &str, value: UniformValue) {
    //     if let Some(shader) = shader_manager.get_shader_mut(&self.shader_name) {
    //         shader.create_uniform(name); // Ensure uniform exists
    //         self.uniforms.insert(name.to_string(), value);
    //     } else {
    //         eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
    //     }
    // }

    // pub fn set_texture(&mut self, shader_manager: &mut ShaderManager, uniform_name: &str, texture_file: &str) {
    //     if let Some(shader) = shader_manager.get_shader_mut(&self.shader_name) {
    //         shader.create_uniform(uniform_name); // Ensure uniform exists
    //         self.texture_names.insert(uniform_name.to_string(), texture_file.to_string());
    //     } else {
    //         eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
    //     }
    // }

    // pub fn init_uniform(&mut self, shader_manager: &mut ShaderManager, name: &str) {
    //     if let Some(shader) = shader_manager.get_shader_mut(&self.shader_name) {
    //         shader.create_uniform(name); // Ensure uniform exists
    //         self.uniforms.insert(name.to_string(), UniformValue::Empty());
    //     } else {
    //         eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
    //     }
    // }

    //i am really not a fan of how this needs so much to funciton like oh no everthing needs models and not everything needs textures and raaah itst annoying
    // pub fn apply(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager, model_matrix: &Matrix4<f32>) {
    //     if let Some(shader) = shader_manager.get_shader(&self.shader_name) {
    //         shader.bind();
    //         shader.set_matrix4fv_uniform("model", model_matrix);

    //         // Apply stored uniforms
    //         for (name, value) in &self.uniforms {
    //             match value {
    //                 UniformValue::Float(f) => shader.set_uniform1f(name, *f),
    //                 UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
    //                 UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
    //                 _ => {}
    //             }
    //         }

    //         // Bind textures
    //         let mut texture_unit = 0;
    //         for (uniform_name, texture_path) in &self.texture_names {
    //             if let Some(texture_id) = texture_manager.get_texture(texture_path) {
    //                 unsafe {
    //                     gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
    //                     gl::BindTexture(gl::TEXTURE_2D, texture_id);
    //                 }
    //                 shader.set_uniform1i(uniform_name, &(texture_unit as i32));
    //                 texture_unit += 1;
    //             } else {
    //                 eprintln!("Warning: Texture '{}' not found!", texture_path);
    //             }
    //         }
    //     } else {
    //         eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
    //     }
    // }

    // pub fn apply_no_model(&self, shader_manager: &ShaderManager, texture_manager: &TextureManager) {
    //     if let Some(shader) = shader_manager.get_shader(&self.shader_name) {
    //         shader.bind();

    //         // Set uniforms
    //         for (name, value) in &self.uniforms {
    //             match value {
    //                 UniformValue::Float(f) => shader.set_uniform1f(name, *f),
    //                 UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
    //                 UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
    //                 _ => {}
    //             }
    //         }

    //         // Bind textures
    //         let mut texture_unit = 0;
    //         for (uniform_name, texture_path) in &self.texture_names {
    //             if let Some(texture_id) = texture_manager.get_texture(texture_path) {
    //                 unsafe {
    //                     gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
    //                     gl::BindTexture(gl::TEXTURE_2D, texture_id);
    //                 }
    //                 shader.set_uniform1i(uniform_name, &(texture_unit as i32));
    //                 texture_unit += 1;
    //             } else {
    //                 eprintln!("Warning: Texture '{}' not found!", texture_path);
    //             }
    //         }
    //     } else {
    //         eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
    //     }
    // }

    // pub fn apply_no_texture(&self, shader_manager: &ShaderManager, model_matrix: &Matrix4<f32>) {
    //     if let Some(shader) = shader_manager.get_shader(&self.shader_name) {
    //         shader.bind();
    //         shader.set_matrix4fv_uniform("model", model_matrix);

    //         // Apply stored uniforms
    //         for (name, value) in &self.uniforms {
    //             match value {
    //                 UniformValue::Float(f) => shader.set_uniform1f(name, *f),
    //                 UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
    //                 UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
    //                 _ => {}
    //             }
    //         }
    //     } else {
    //         eprintln!("Warning: Shader '{}' not found in ShaderManager!", self.shader_name);
    //     }
    // }

    // pub fn set_float_property(&mut self, key: &str, value: f32, shader_manager: &ShaderManager) {
    //     self.set_uniform(key, UniformValue::Float(value));
    // }

    // pub fn set_vector4_property(&mut self, key: &str, value: Vector4<f32>) {
    //     self.set_uniform(key, UniformValue::Vector4(value));
    // }

    // pub fn set_matrix4fv_uniform(&mut self, manager: &mut ShaderManager, key: &str, value: Matrix4<f32>) {
    //     self.set_uniform(manager, key, UniformValue::Matrix4(value));
    // }

    // pub fn set_matrix4_property(&mut self, key: &str, value: Matrix4<f32>) {
    //     self.set_uniform(key, UniformValue::Matrix4(value));
    // }


    //add one to like init and like set at the same time

    //maybe add something to check if already exists bro
    pub fn init_uniform(&mut self, key: &str)
    {
        self.shader.lock().unwrap().create_uniform(key);
        self.uniforms.insert(key.to_string(), UniformValue::Empty());
    }

    //im not gonna lie, the system of where things are stored is... dummay dumb
    //like I am abusing the unsafe in the gl_wrapper for the set class to get around making it mutable here
    //even through we are in fact editing shader values
    pub fn set_uniform(&self, key: &str, value: &UniformValue)
    {
        let currShader = self.shader.lock().unwrap();
        match value {
            UniformValue::Float(f) => currShader.set_uniform1f(key, *f),
            UniformValue::Vector4(v) => currShader.set_uniform4f(key, v),
            UniformValue::Matrix4(m) => currShader.set_matrix4fv_uniform(key, m),
            _ => {}
        }
    }

    pub fn set_matrix4fv_uniform(&mut self, key: &str, value: &Matrix4<f32>) {
        self.shader.lock().unwrap().set_matrix4fv_uniform(key, value);
    }
}

//there is a reason I used lock here i uuh just dont remember why
pub struct MaterialManager {
    materials: RwLock<HashMap<String, Arc<RwLock<Material>>>>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: RwLock::new(HashMap::new()),
        }
    }

    // fn priv_get_mat(&self, name: &str) -> Option<&Arc<RwLock<Material>>>{
    //     if let mat = self.materials.read().unwrap().get(name){
    //         mat
    //     } else {
    //         panic!("Material '{}' not found when apply!", name);//hmmm
    //         //Some(Err(("Material '{}' not found when apply!", name)));
    //     }
    // }

    pub fn load_material(&self, name: &str, shader_manager: &ShaderManager, shader_name: &str) -> Arc<RwLock<Material>> {
        let mut materials = self.materials.write().unwrap();

        if let Some(mat) = materials.get(name) {
            return Arc::clone(mat);
        }

        if let Some(shader) = shader_manager.get_shader(shader_name) {
            let new_material = Arc::new(RwLock::new(Material::new(shader)));
            materials.insert(name.to_string(), Arc::clone(&new_material));
            new_material
        } else {
            panic!("Shader '{}' not found in ShaderManager from manager!", shader_name);
        }
    }

    pub fn edit_material<F>(&self, name: &str, edit_fn: F)
    where
        F: FnOnce(&mut Material),
    {
        if let Some(material) = self.materials.read().unwrap().get(name) {
            let mut mat = material.write().unwrap();
            edit_fn(&mut mat);
        } else {
            eprintln!("Material '{}' not found when edit!", name);
        }
    }

    //pub fn set_matrix4fv_uniform()

    pub fn apply(&self, name: &str, texture_manager: &TextureManager, model_matrix: &Matrix4<f32>) {
        if let Some(material) = self.materials.read().unwrap().get(name) {
            material.write().unwrap().apply(texture_manager, model_matrix);
        } else {
            eprintln!("Material '{}' not found when apply!", name);
        }
        // if let Some(shader) = shader_manager.get_shader(shader_name) {
        //     let new_material = Arc::new(RwLock::new(Material::new(shader)));

        // } else {
        //     panic!("Shader '{}' not found in ShaderManager!", shader_name);
        // }
    }

    pub fn init_uniform(&self, name: &str, key: &str)
    {
        if let Some(mat) = self.materials.read().unwrap().get(name){//bruh nnnaaaaaahhhhhh
            mat.write().unwrap().init_uniform(key);
        }
    }


    //ok my question is if with the key would there be a better way to find the Type of the generics, maybe 
    pub fn update_uniform<T>(&self, name: &str, key: &str, value: T)
    where
            UniformValue: TryFrom<T>,
    {
        let utype: Result<UniformValue, _> = UniformValue::try_from(value);
        if let Some(mat) = self.materials.read().unwrap().get(name){//bruh nnnaaaaaahhhhhh
            match utype {
                Ok(val) => {
                    match val {
                        UniformValue::Float(f) => mat.write().unwrap().set_uniform(name, &val),
                        UniformValue::Vector4(v) => mat.write().unwrap().set_uniform(name, &val),
                        UniformValue::Matrix4(m) => mat.write().unwrap().set_uniform(name, &val),
                        _ => {}
                    }
                    println!("Uniform set: {:?}", val)
                },
                Err(e) => println!("Failed to convert uniform: {}, {}", name, key),
            }
        } else {
            panic!("Material '{}' not found when apply!", name);
        }
    }

    //     let mat = self.priv_get_mat(name).unwrap();
    //     mat.write().unwrap().set_uniform(key, value);
    // }
        // where
        //     UniformValue: TryFrom<T>,
        // {
        //     let utype: Result<UniformValue, _> = UniformValue::try_from(value);
        //     let mat = self.priv_get_mat(name).unwrap();
        //     match utype {
        //         Ok(val) => {
        //             match val {
        //                 UniformValue::Float(f) => mat.set_uniform1f(name, *f),
        //                 UniformValue::Vector4(v) => shader.set_uniform4f(name, v),
        //                 UniformValue::Matrix4(m) => shader.set_matrix4fv_uniform(name, m),
        //                 _ => {}
        //             }
        //             println!("Uniform set: {:?}", val)
        //         },
        //         Err(e) => println!("Failed to convert uniform: {}, {}", name, key),
        //     }
        //}

        // if let Some(&uniform_location) = self.shader.uniform_ids.get(key) {
        //                  match value {
        //                     UniformValue::Float(f) => unsafe {
        //                          gl::Uniform1f(uniform_location, *f);
        //                      },
        //                      UniformValue::Vector4(v) => unsafe {
        //                        gl::Uniform4fv(uniform_location, 1, v.as_ptr());
        //                     },
        //                    UniformValue::Matrix4(m) => unsafe {
        //                         gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, m.as_ptr());
        //                     },
        //                 }
        //         }
}

