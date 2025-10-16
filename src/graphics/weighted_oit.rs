use std::ptr;
use std::ffi::CString;
use gl;
use gl::types::GLuint;
use std::rc::Rc;

use super::gl_wrapper::{Framebuffer, ShaderProgram, depthTexture};//lol depth texture like being just used bc i am 2 lazy
//todo fix depth texture and text manager to be all one love or make frame buffer take in texture not just depth texture
use super::camera::Camera;
use crate::graphics::texture_manager::RenderTexture;
use crate::model::mesh::Mesh;
use crate::model::objload::ModelTrait;
use super::texture_manager::TextureManager;
// use lazy_static::lazy_static;
//todo I would actually like to explain why I do not use lazy static anywhere in my code even thought it would make woing with certain things a lot easier.... the reason is is bc I am using rust and there is often a better or more efficent solution that often helps reduce code complexity
//and improve like ownership like readability

/// Weighted blended OIT renderer helper.
pub struct WeightedOIT {
    pub fbo: OITFramebuffer,
    pub accum_tex: RenderTexture,
    pub reveal_tex: RenderTexture,
    pub resolve_shader: ShaderProgram,
    pub transparent_shader: ShaderProgram,
    width: u32,
    height: u32,
    fs_quad_vao: GLuint,
    fs_quad_vbo: GLuint,
}

impl WeightedOIT {
    /// Create new OIT resources. Must be called when GL context is valid.
    pub fn new(width: u32, height: u32) -> Self {
        // Create textures
        let accum = RenderTexture::new(width, height, gl::RGBA16F);
        let reveal = RenderTexture::new(width, height, gl::R16F);  // R16F or R8; float is safer

    
        let fbo = OITFramebuffer::new(720, 720);
        fbo.bind();
        fbo.clear();
        OITFramebuffer::unbind();

        // Compile shaders (see strings below)
        let mut oit_transparent_shader = ShaderProgram::new(
            "shaders/oit_transparent.vert",
            "shaders/oit_transparent.frag"
        );
        
        oit_transparent_shader.bind();//lol just create the uniforms here
        oit_transparent_shader.create_uniforms(vec!["view", "projection", "model"]);

        let mut oit_resolve_shader = ShaderProgram::new(
            "shaders/oit_resolve.vert",
            "shaders/oit_resolve.frag"
        );

        oit_resolve_shader.bind();//lol just create the uniforms here
        oit_resolve_shader.create_uniforms(vec!["u_accumTex", "u_revealTex"]);

        ShaderProgram::unbind();

        let (fs_quad_vao, fs_quad_vbo) = Self::create_fullscreen_quad();

        Self {
            fbo,
            accum_tex: accum,
            reveal_tex: reveal,
            resolve_shader: oit_resolve_shader,
            transparent_shader: oit_transparent_shader,
            width,
            height,
            fs_quad_vao,
            fs_quad_vbo,
        }
    }

    // pub fn resize(&mut self, width: u32, height: u32) {
    //     if self.width == width && self.height == height { return; }
    //     self.width = width; self.height = height;
    //     self.accum_tex.resize(width, height);
    //     self.reveal_tex.resize(width, height);
    // }

    /// Run the transparent pass: render transparent_models using weighted blended OIT,
    /// then composite into the currently bound default framebuffer using resolve pass.
    /// - `transparent_models` : collection of models flagged transparent
    /// - `camera` : camera for view/proj
    /// - `texture_manager` : to bind textures for materials (if needed)
    pub fn render_transparency<'a>(
        &mut self,
        transparent_models: impl IntoIterator<Item = &'a Box<dyn ModelTrait>>,
        camera: &Camera,
        texture_manager: &TextureManager,
    ) {
        // 1) Bind OIT FBO and clear (accum=0, revealage=1)
        self.fbo.bind();
        unsafe {
            // Clear accum to zero
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Clear revealage to 1.0
            gl::DrawBuffer(gl::COLOR_ATTACHMENT1);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0); // stored as float in R
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Make sure both buffers are the draw buffers again
            let bufs = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(bufs.len() as i32, bufs.as_ptr());
        }

        // 2) Setup blending for weighted blended OIT
        unsafe {
            gl::Enable(gl::BLEND);
            // Additive for accum (RGB): ONE, ONE
            // Revealage (alpha channel) uses: ZERO, ONE_MINUS_SRC_ALPHA (see shader outputs)
            gl::BlendFuncSeparate(gl::ONE, gl::ONE, gl::ZERO, gl::ONE_MINUS_SRC_ALPHA);

            // We don't want transparent geometry to write to the depth buffer (we still want depth test)
            gl::DepthMask(gl::FALSE);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        // 3) Render all transparent geometry using the transparent shader to MRTs
        let mut t_shader = &self.transparent_shader; //mut here bc it is actually being mut gpu side so this helps organize things even though im pretty sure i shouldnt do this...
        t_shader.bind();
        t_shader.set_matrix4fv_uniform("view", camera.get_view());
        t_shader.set_matrix4fv_uniform("projection", camera.get_p_matrix());
        // Provide any uniforms your transparent shader expects (lights, etc.)
        // Optionally bind depth texture from opaque pass to discard fragments behind opaque geometry:
        // t_shader.set_uniform1i("u_sceneDepth", 0); bind depth texture to TEXTURE0 before draw.

        for model in transparent_models.into_iter() {
            t_shader.set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());
            // bind material maps via texture_manager if needed...
            // model.get_material().write().unwrap().apply(texture_manager, &model.get_world_coords().get_model_matrix());
            model.get_mesh().draw();
        }

        // 4) Restore GL state
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
        }

        Framebuffer::unbind();

        self.resolve_shader.bind();

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.accum_tex.id);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.reveal_tex.id);
        }
        self.resolve_shader.set_uniform1i("u_accumTex", &0);
        self.resolve_shader.set_uniform1i("u_revealTex", &1);

        self.draw_fullscreen_quad();

        ShaderProgram::unbind();

        // Unbind textures to keep state clean
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1); gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::ActiveTexture(gl::TEXTURE0); gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn create_fullscreen_quad() -> (GLuint, GLuint) {
        let mut vao = 0;
        let mut vbo = 0;
        unsafe {
            let vertices: [f32; 16] = [
                -1.0, -1.0, 0.0, 0.0, // pos, uv
                 1.0, -1.0, 1.0, 0.0,
                -1.0,  1.0, 0.0, 1.0,
                 1.0,  1.0, 1.0, 1.0,
            ];
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * 4, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * 4, (2 * 4) as *const _);
            gl::BindVertexArray(0);
        }
        (vao, vbo)
    }

    fn draw_fullscreen_quad(&self) {
        unsafe {
            gl::BindVertexArray(self.fs_quad_vao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for WeightedOIT {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.fs_quad_vao);
            gl::DeleteBuffers(1, &self.fs_quad_vbo);
        }
    }
}

pub struct OITFramebuffer {
    pub id: GLuint,
    pub accum_tex: Rc<super::texture_manager::RenderTexture>,
    pub reveal_tex: Rc<RenderTexture>,
}

impl OITFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let accum_tex = Rc::new(RenderTexture::new(width, height, gl::RGBA16F));
        let reveal_tex = Rc::new(RenderTexture::new(width, height, gl::R16F));

        let mut fbo: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            // Attach textures to color attachments
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER, 
                gl::COLOR_ATTACHMENT0, 
                gl::TEXTURE_2D, 
                accum_tex.id, 
                0
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER, 
                gl::COLOR_ATTACHMENT1, 
                gl::TEXTURE_2D, 
                reveal_tex.id, 
                0
            );

            // Set draw buffers for MRT (Multiple Render Targets)
            let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(2, attachments.as_ptr());

            // Check framebuffer completeness
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("OIT Framebuffer is not complete! Status: 0x{:X}", status);
            }

            // Unbind
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Self { id: fbo, accum_tex, reveal_tex }
    }

    pub fn bind(&self) {
        unsafe { 
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            
            // Ensure draw buffers are set (some drivers need this)
            let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(2, attachments.as_ptr());
        }
    }

    pub fn unbind() {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    pub fn clear(&self) {
        unsafe {
            // Clear accum to (0, 0, 0, 0)
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Clear reveal to 1.0
            gl::DrawBuffer(gl::COLOR_ATTACHMENT1);
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Restore draw buffers
            let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(2, attachments.as_ptr());
        }
    }
}

impl Drop for OITFramebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}

// pub struct OITFramebuffer {
//     pub id: GLuint,
//     pub accum_tex: Rc<super::texture_manager::RenderTexture>,
//     pub reveal_tex: Rc<RenderTexture>,
// }

// impl OITFramebuffer {
//     pub fn new(width: u32, height: u32) -> Self {
//         let accum_tex = Rc::new(RenderTexture::new(width, height, gl::RGBA16F));
//         let reveal_tex = Rc::new(RenderTexture::new(width, height, gl::R16F));

//         let mut fbo: GLuint = 0;
//         unsafe {
//             gl::GenFramebuffers(1, &mut fbo);
//             gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

//             gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, accum_tex.id, 0);
//             gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::TEXTURE_2D, reveal_tex.id, 0);

//             let attachments = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
//             gl::DrawBuffers(2, attachments.as_ptr());

//             //todo attach depth buffer later from the depth buffer bc will make faster
//         }

//         Self { id: fbo, accum_tex, reveal_tex }
//     }

//     pub fn bind(&self) {
//         unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.id); }
//     }

//     pub fn unbind() {
//         unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
//     }
// }
