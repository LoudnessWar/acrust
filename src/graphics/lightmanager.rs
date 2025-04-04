use cgmath::Vector3;
use gl::types::GLsizeiptr;
use gl::types::GLuint;

use super::gl_wrapper::ShaderProgram;

//do I just make a Color Type?
//even though it's a little bit tedious im making it a trait because later there will likely
//be instances where I want to be able to run these functions on all light sources
//so that I can have a lighting manager passing these between each other and objects
pub trait LightTrait {
    fn is_on(&self) -> bool{
        true
    }
    fn get_emission_color(&self) -> &Vector3<f32>;
    fn get_emission_intensity(&self) -> &f32;//maybe just u8
    fn get_ambient_color(&self) -> &Vector3<f32>;
    fn get_specular_color(&self) -> &Vector3<f32>;
    fn get_position(&self) -> &Vector3<f32>;

    fn set_emission_color(&mut self, color: Vector3<f32>);
    fn set_emission_intensity(&mut self, intensity: f32);//maybe just u8
    fn set_ambient_color(&mut self, color: Vector3<f32>);
    fn set_specular_color(&mut self, color: Vector3<f32>);
    //maybe diffuse material color
    //i need a position but i am like 90% sure like 90%
    //that mesh or like worldcoords already has that.
    //so that will probably be the implimentation of it under most situations
}

//need light manager eventaully, which will prolly need mutex or sometype of shared
//like managment 😥😥😥... maybe not actaully
//it will be faster for the graphics card to process the combining of the lights
//all that it would need to do then is generate a matrix of lights and their elements
//this would be lit prolly


//ok after a lot of cumtimplation we finna use forward+ shading


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LightGPU {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub _pad: f32,
}

// file: lightmanager.rs

use crate::graphics::gl_wrapper::*;
use cgmath::Vector4;
use std::mem;

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub position: Vector3<f32>,
    pub radius: f32,
    pub color: Vector3<f32>,
    pub _pad: f32,
}

pub struct LightManager {
    pub lights: Vec<Light>,
    pub light_ssbo: GLuint,
    pub max_lights: usize,
}

impl LightManager {
    pub fn new(max_lights: usize) -> Self {
        let mut light_ssbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut light_ssbo);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, light_ssbo);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                (max_lights * mem::size_of::<Light>()) as GLsizeiptr,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }

        Self {
            lights: Vec::with_capacity(max_lights),
            light_ssbo,
            max_lights,
        }
    }

    pub fn update_ssbo(&self) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.light_ssbo);
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY) as *mut Light;
            if !ptr.is_null() {
                for (i, light) in self.lights.iter().enumerate() {
                    *ptr.add(i) = *light;
                }
            }
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }

    pub fn bind_ssbo(&self, binding_point: GLuint) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding_point, self.light_ssbo);
        }
    }

    pub fn add_light(&mut self, position: Vector3<f32>, radius: f32, color: Vector3<f32>) {
        if self.lights.len() < self.max_lights {
            self.lights.push(Light {
                position,
                radius,
                color,
                _pad: 0.0,
            });
        }
    }
}

pub enum RenderPassType {
    DepthPrepass,
    ForwardPlus,
}

pub trait SceneRenderable {
    fn draw(&self, pass: RenderPassType);
}

// Later we can extend this to support tiled light index buffers per tile for Forward+

pub struct LightCullingBuffers {
    pub counts_ssbo: u32,
    pub indices_ssbo: u32,
}

pub unsafe fn create_light_index_buffers(
    num_tiles: usize,
    max_lights_per_tile: usize,
) -> LightCullingBuffers {
    let mut counts_ssbo: u32 = 0;
    gl::GenBuffers(1, &mut counts_ssbo);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, counts_ssbo);
    gl::BufferData(
        gl::SHADER_STORAGE_BUFFER,
        (num_tiles * std::mem::size_of::<u32>()) as isize,
        std::ptr::null(),
        gl::DYNAMIC_DRAW,
    );
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, counts_ssbo);

    let mut indices_ssbo: u32 = 0;
    gl::GenBuffers(1, &mut indices_ssbo);
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, indices_ssbo);
    gl::BufferData(
        gl::SHADER_STORAGE_BUFFER,
        (num_tiles * max_lights_per_tile * std::mem::size_of::<u32>()) as isize,
        std::ptr::null(),
        gl::DYNAMIC_DRAW,
    );
    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 3, indices_ssbo);

    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);

    LightCullingBuffers {
        counts_ssbo,
        indices_ssbo,
    }
}

pub unsafe fn dispatch_light_culling(
    compute_shader_program: u32,
    num_tiles_x: u32,
    num_tiles_y: u32,
) {
    gl::UseProgram(compute_shader_program);
    gl::DispatchCompute(num_tiles_x, num_tiles_y, 1);
    gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
}

pub unsafe fn bind_depth_texture_for_compute(depth_texture: u32, unit: u32) {
    gl::ActiveTexture(gl::TEXTURE0 + unit);
    gl::BindTexture(gl::TEXTURE_2D, depth_texture);
}

pub unsafe fn render_depth_only_prepass(depth_shader_program: u32){
        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE); // Don't write color
        gl::DepthMask(gl::TRUE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    
        gl::UseProgram(depth_shader_program);
        // set uniforms...
    
        // bind VAO, issue draw calls...
    
        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE); // Re-enable color write after
}

pub unsafe fn compute_shader_lightculling(width: i32, height: i32){
    let mut fbo = 0;
    gl::GenFramebuffers(1, &mut fbo);
    gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

    let mut depth_tex = 0;
    gl::GenTextures(1, &mut depth_tex);
    gl::BindTexture(gl::TEXTURE_2D, depth_tex);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::DEPTH_COMPONENT32F as i32,
        width,
        height,
        0,
        gl::DEPTH_COMPONENT,
        gl::FLOAT,
        std::ptr::null(),
    );

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST.try_into().unwrap());
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST.try_into().unwrap());

    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_tex, 0);
    gl::DrawBuffer(gl::NONE);
    gl::ReadBuffer(gl::NONE);

    assert!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE);
    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

}