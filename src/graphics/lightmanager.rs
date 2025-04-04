use cgmath::Vector3;

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


pub struct LightManager{
    light_sources: Vec<Box<dyn LightTrait>>,//im wondering if I should have been using box more
}//erm intuitevly, I could just make this a hashmap and then just replace the value with a new one in an instance of modification...
//this might be unifficient

impl LightManager{
    pub fn new() -> Self{
        Self { light_sources: Vec::new() }
    }

    pub fn add_light(&mut self, light: Box<dyn LightTrait>){
        self.light_sources.push(light);
    }

    pub unsafe fn upload_lights_to_ssbo(lights: &[LightGPU]) -> u32 {
        let mut ssbo: u32 = 0;
        gl::GenBuffers(1, &mut ssbo);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
    
        let ptr = lights.as_ptr() as *const std::ffi::c_void;
        let size = (std::mem::size_of::<LightGPU>() * lights.len()) as isize;
    
        gl::BufferData(gl::SHADER_STORAGE_BUFFER, size, ptr, gl::DYNAMIC_DRAW);
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, ssbo); // binding = 1
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
    
        ssbo
    }
    

    //pub fn compile_lights(&self,)
}

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