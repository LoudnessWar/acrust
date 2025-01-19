use crate::graphics::gl_wrapper::*;
//use image::GenericImageView;

use gl::types::*;
// use std::mem;
// use std::ptr;

pub struct Skybox {
    vao: Vao,
    vbo: BufferObject,
    texture_id: GLuint,
}

impl Skybox {
    pub fn new(faces: &[&str]) -> Self {
        // Cube vertices for skybox (just position)
        let vertices: [f32; 108] = [
            // Positions          
            -1.0,  1.0, -1.0,
            -1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
             1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,

            -1.0, -1.0,  1.0,
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0, -1.0,
            -1.0,  1.0,  1.0,
            -1.0, -1.0,  1.0,

             1.0, -1.0, -1.0,
             1.0, -1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,

            -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
             1.0, -1.0,  1.0,
            -1.0, -1.0,  1.0,

            -1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0,  1.0,  1.0,
             1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            -1.0,  1.0, -1.0,

            -1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0, -1.0,
             1.0, -1.0, -1.0,
            -1.0, -1.0,  1.0,
             1.0, -1.0,  1.0
        ];

        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
        vbo.bind();
        vbo.store_f32_data(&vertices);

        // Configure vertex attributes
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                3 * std::mem::size_of::<GLfloat>() as GLsizei, 
                std::ptr::null()
            );
        }

        // Load cubemap texture (you'll replace this with your actual texture loading)
        let texture_id = Self::load_cubemap(faces);

        Skybox {
            vao,
            vbo,
            texture_id,
        }
    }

    fn load_cubemap(faces: &[&str]) -> GLuint {
        let mut texture_id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture_id);
    
            const TARGET_SIZE: u32 = 1024; // All faces will be this size
    
            for (i, face) in faces.iter().enumerate() {
                let img = image::open(face).expect(&format!("Failed to load texture face: {}", face));
                // Resize image to target size
                let resized = img.resize_exact(TARGET_SIZE, TARGET_SIZE, image::imageops::FilterType::Lanczos3);
                let data = resized.to_rgba8();
                
                println!("Loading face {} ({}) with size {}x{}", i, face, TARGET_SIZE, TARGET_SIZE);
                
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::RGBA as i32,
                    TARGET_SIZE as i32,
                    TARGET_SIZE as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as *const _
                );
                
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    println!("OpenGL error while loading face {}: {}", i, error);
                }
            }
    
            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
        }
        texture_id
    }


    //ok so this is for if you want to render the skybox seperate, if I were doing it someway else I could use this code and just put it in my other render
    pub fn render(&self, shader_program: &ShaderProgram, view_matrix: &cgmath::Matrix4<f32>, projection_matrix: &cgmath::Matrix4<f32>) {
        // Create a rotation-only view matrix

    
        unsafe {
            // Save current OpenGL state
            let mut previous_depth_func = 0;
            gl::GetIntegerv(gl::DEPTH_FUNC, &mut previous_depth_func);
            
            // Setup skybox rendering state
            gl::DepthMask(gl::FALSE);
            gl::DepthFunc(gl::LEQUAL);
            
            self.vao.bind();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture_id);
            
            // Fix the CString lifetime issue
            let skybox_str = std::ffi::CString::new("skybox").unwrap();
            let loc = gl::GetUniformLocation(shader_program.get_program_handle(), skybox_str.as_ptr());
            
            // Print uniform location for debugging
            println!("Skybox uniform location: {}", loc);
            
            if loc >= 0 {
                gl::Uniform1i(loc, 0);
            } else {
                println!("Warning: Skybox uniform not found!");
            }
            
            // Check for any errors before drawing
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL error before skybox draw: {}", error);
            }
            
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            
            // Check for any errors after drawing
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                println!("OpenGL error after skybox draw: {}", error);
            }
            
            // Restore OpenGL state
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(previous_depth_func as u32);
        }
    }

    pub fn get_texture_id(&self) -> GLuint{
        self.texture_id
    }

}