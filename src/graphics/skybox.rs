use crate::graphics::gl_wrapper::*;
//use image::GenericImageView;

use gl::types::*;
use std::mem;
use std::ptr;

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
    
            for (i, face) in faces.iter().enumerate() {
                let img = image::open(face).expect("Failed to load texture");
                let data = img.to_rgba8();
                let width = img.width();
                let height = img.height();
    
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, 
                    0, 
                    gl::RGBA as i32, 
                    width as i32, 
                    height as i32, 
                    0, 
                    gl::RGBA, 
                    gl::UNSIGNED_BYTE, 
                    data.as_ptr() as *const _
                );
            }
    
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
        }
    
        texture_id
    }


    //ok so this is for if you want to render the skybox seperate, if I were doing it someway else I could use this code and just put it in my other render
    pub fn render(&self,  shader_program: &ShaderProgram, view_matrix: &cgmath::Matrix4<f32>, projection_matrix: &cgmath::Matrix4<f32>) {
        //shader_program.bind();
        
        let mut view = view_matrix.clone();//ok should I just get my view I already have in camera instead
        view = cgmath::Matrix4::from_cols(
            view[0],
            view[1],
            view[2],
            cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0)
        );
        
        shader_program.set_matrix4fv_uniform("view", &view);
        shader_program.set_matrix4fv_uniform("projection", projection_matrix);

        unsafe {
            // Set depth function to allow skybox to be rendered at the far plane
            gl::DepthFunc(gl::LEQUAL);  
            
            // Bind skybox VAO and cubemap texture
            self.vao.bind();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture_id);
            
            // Draw the skybox (36 vertices representing a cube)
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            
            // Reset depth function
            gl::DepthFunc(gl::LESS);  
        }
    }

    pub fn get_texture_id(&self) -> GLuint{
        self.texture_id
    }

}