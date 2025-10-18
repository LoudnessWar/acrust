use std::ptr;
use std::ffi::CString;
use cgmath::{Vector3, Vector4};
use gl;
use gl::types::GLuint;
use std::rc::Rc;

use super::gl_wrapper::{Framebuffer, ShaderProgram, depthTexture};//lol depth texture like being just used bc i am 2 lazy
//todo fix depth texture and text manager to be all one love or make frame buffer take in texture not just depth texture
use super::camera::Camera;
use crate::graphics::gl_wrapper::LightManager;
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
    // pub accum_tex: RenderTexture,
    // pub reveal_tex: RenderTexture,
    pub resolve_shader: ShaderProgram,
    pub transparent_shader: ShaderProgram,
    width: u32,
    height: u32,
    fs_quad_vao: GLuint,
    fs_quad_vbo: GLuint,
    depth_attached: bool,
}

impl WeightedOIT {
    /// Create new OIT resources. Must be called when GL context is valid.
    pub fn new(width: u32, height: u32) -> Self {
        // Create textures
        // let accum = RenderTexture::new(width, height, gl::RGBA16F);
        // let reveal = RenderTexture::new(width, height, gl::R16F);  // R16F or R8; float is safer

        
        let fbo = OITFramebuffer::new(width, height);
        fbo.bind();
        fbo.clear();
        OITFramebuffer::unbind();

        // Compile shaders (see strings below)
        let mut oit_transparent_shader = ShaderProgram::new(
            "shaders/oit_transparent.vert",
            "shaders/oit_transparent.frag"
        );
        
        oit_transparent_shader.bind();//lol just create the uniforms here
        oit_transparent_shader.create_uniforms(vec!["view", "projection", "model", "u_diffuseColor", "u_alpha", "u_lightCount", "u_tileCountX"]);

        oit_transparent_shader.create_uniforms(vec!["u_ior", "u_roughness"]);

        let mut oit_resolve_shader = ShaderProgram::new(
            "shaders/oit_resolve.vert",
            "shaders/oit_resolve.frag"
        );

        println!("Resolve shader program ID: {:?}", oit_resolve_shader.get_program_handle());

        oit_resolve_shader.bind();//lol just create the uniforms here
        oit_resolve_shader.create_uniforms(vec!["u_accumTex", "u_revealTex"]);

        ShaderProgram::unbind();

        let (fs_quad_vao, fs_quad_vbo) = Self::create_fullscreen_quad();

        Self {
            fbo,
            // accum_tex: accum,
            // reveal_tex: reveal,
            resolve_shader: oit_resolve_shader,
            transparent_shader: oit_transparent_shader,
            width,
            height,
            fs_quad_vao,
            fs_quad_vbo,
            depth_attached: false,
        }
    }

    pub fn attach_depth_texture(&mut self, depth_texture_id: GLuint) {
        if self.depth_attached {
            return;  // Already attached
        }
        
        self.fbo.bind();
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_texture_id,
                0
            );
            
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("OIT Framebuffer incomplete after depth attach! Status: 0x{:X}", status);
            }
        }
        OITFramebuffer::unbind();
        
        self.depth_attached = true;
        println!("Attached depth texture {} to OIT FBO", depth_texture_id);
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
        light_manager: &LightManager,
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
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Make sure both buffers are the draw buffers again
            let bufs = [gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1];
            gl::DrawBuffers(bufs.len() as i32, bufs.as_ptr());
        }

        // 2) Setup blending for weighted blended OIT
        unsafe {
            gl::Enable(gl::BLEND);
            
            // For MRT 0 (accum): additive
            gl::BlendFuncSeparatei(0, gl::ONE, gl::ONE, gl::ONE, gl::ONE);
            
            // For MRT 1 (reveal): multiplicative  
            gl::BlendFuncSeparatei(1, gl::ZERO, gl::ONE_MINUS_SRC_COLOR, gl::ZERO, gl::ONE_MINUS_SRC_COLOR);
            
            gl::DepthMask(gl::FALSE);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        // 3) Render all transparent geometry
        let t_shader = &self.transparent_shader;
        t_shader.bind();
        
        t_shader.set_matrix4fv_uniform("view", camera.get_view());
        t_shader.set_matrix4fv_uniform("projection", camera.get_p_matrix());
        t_shader.set_uniform4f("u_diffuseColor", &Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 });
        t_shader.set_uniform1f("u_alpha", 0.5);
        //t_shader.set_uniform1f("u_specularPower", 32.0);
        t_shader.set_uniform1i("u_lightCount", &(light_manager.lights.len() as i32));

                t_shader.set_uniform1f("u_ior", 1.52);
                t_shader.set_uniform1f("u_roughness",  0.1);
                //t_shader.set_uniform3f("u_tintColor", &Vector3::new(1.0, 1.0, 1.0));

        if let Some(culling_buffers) = &light_manager.culling_buffers {
            let (tile_count_x, _) = culling_buffers.get_tile_counts();
            t_shader.set_uniform1i("u_tileCountX", &(tile_count_x as i32));
        }

        for model in transparent_models.into_iter() {
            t_shader.set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());
            model.get_mesh().draw();
        }

        // 4) Restore GL state
        unsafe {
            gl::Disable(gl::DEPTH_TEST);//TODO i removed this as a test but this might have been a mistake
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
        }

        // Debug output
        unsafe {
            let mut pixel: [f32; 4] = [0.0; 4];
            gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
            gl::ReadPixels(360, 360, 1, 1, gl::RGBA, gl::FLOAT, pixel.as_mut_ptr() as *mut _);
            println!("Accum pixel at center: {:?}", pixel);
            
            let mut reveal_pixel: [f32; 1] = [0.0];
            gl::ReadBuffer(gl::COLOR_ATTACHMENT1);
            gl::ReadPixels(360, 360, 1, 1, gl::RED, gl::FLOAT, reveal_pixel.as_mut_ptr() as *mut _);
            println!("Reveal pixel at center: {:?}", reveal_pixel);
        }

        Framebuffer::unbind();

        // 5) Resolve pass - composite onto screen
        self.resolve_shader.bind();

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.fbo.accum_tex.id);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.fbo.reveal_tex.id);
        }

        self.resolve_shader.set_uniform1i("u_accumTex", &0);
        self.resolve_shader.set_uniform1i("u_revealTex", &1);

        // Enable blending for compositing transparent result over opaque scene
        // unsafe {
        //     gl::Disable(gl::DEPTH_TEST);
        //     gl::Disable(gl::BLEND);
        //     //gl::BlendFunc(gl::ONE_MINUS_SRC_ALPHA, gl::SRC_ALPHA);
        //     //gl::ClearColor(1.0, 0.0, 0.0, 1.0);

        //     gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        //     gl::Clear(gl::COLOR_BUFFER_BIT);
        // }

        unsafe {
            //gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA); // SWAPPED!
            // This means: final = src * src.a + dst * (1 - src.a)
        }

        println!("About to draw fullscreen quad");
        unsafe {
            let mut current_fbo: i32 = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut current_fbo);
            println!("Current FBO bound: {}", current_fbo); // Should be 0 (default framebuffer)
        }

        unsafe {
            // Verify textures are valid
            println!("Accum texture ID: {}", self.fbo.accum_tex.id);
            println!("Reveal texture ID: {}", self.fbo.reveal_tex.id);
            
            // Check if textures are actually bound
            let mut bound_tex_0: i32 = 0;
            gl::ActiveTexture(gl::TEXTURE0);
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut bound_tex_0);
            println!("Texture bound to unit 0: {}", bound_tex_0);
            
            let mut bound_tex_1: i32 = 0;
            gl::ActiveTexture(gl::TEXTURE1);
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut bound_tex_1);
            println!("Texture bound to unit 1: {}", bound_tex_1);
        }

        unsafe {
            let mut viewport: [i32; 4] = [0; 4];
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            println!("Viewport: x={}, y={}, w={}, h={}", 
                    viewport[0], viewport[1], viewport[2], viewport[3]);
            
            // Make sure viewport is set correctly
            gl::Viewport(0, 0, 720 as i32, 720 as i32);
        }

        unsafe {
            let mut error = gl::GetError();
            while error != gl::NO_ERROR {
                println!("GL Error before quad: 0x{:X}", error);
                error = gl::GetError();
            }
        }

        // unsafe {
        //     // Check and disable face culling
        //     let mut cull_enabled: u8 = 0;
        //     gl::GetBooleanv(gl::CULL_FACE, &mut cull_enabled);
        //     println!("Face culling enabled: {}", cull_enabled);
            
        //     if cull_enabled != 0 {
        //         let mut cull_mode: i32 = 0;
        //         gl::GetIntegerv(gl::CULL_FACE_MODE, &mut cull_mode);
        //         println!("Cull face mode: 0x{:X}", cull_mode);
        //     }
            
        //     // Disable it just in case
        //     gl::Disable(gl::CULL_FACE);
            
        //     // Also check polygon mode
        //     let mut poly_mode: [i32; 2] = [0; 2];
        //     gl::GetIntegerv(gl::POLYGON_MODE, poly_mode.as_mut_ptr());
        //     println!("Polygon mode: 0x{:X}", poly_mode[0]);
        // }


        self.draw_fullscreen_quad();


        println!("Finished drawing fullscreen quad");

        unsafe {
            let mut error = gl::GetError();
            while error != gl::NO_ERROR {
                println!("GL Error after quad: 0x{:X}", error);
                error = gl::GetError();
            }
        }

        unsafe {
            //gl::Disable(gl::BLEND);
            // gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }

        ShaderProgram::unbind();

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
            println!("Drawing quad with VAO: {}", self.fs_quad_vao);
            
            // Check if VAO is valid
            let is_vao = gl::IsVertexArray(self.fs_quad_vao);
            println!("Is valid VAO: {}", is_vao);
            
            gl::BindVertexArray(self.fs_quad_vao);
            
            // Check what's actually bound
            let mut bound_vao: i32 = 0;
            gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut bound_vao);
            println!("Actually bound VAO: {}", bound_vao);
            
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
