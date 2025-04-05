use std::fmt;
use std::collections::HashMap;
use std::mem;
use std::os::raw::*;
use std::ffi::CString;
use std::fs::File;
use std::ptr;
use std::io::Read;
use std::rc::Rc;
// use std::sync::PoisonError;
use cgmath::*;
use gl::types::*;
use std::sync::Mutex;
use std::sync::Arc;

use crate::model::mesh;
use crate::model::mesh::Mesh;

/// # Vertex Array Object
///
/// ## Example
/// ```
/// let vao = Vao::new();
/// vao.bind();
/// ```
pub struct Vao {
    id: gl::types::GLuint,
}

impl Vao {
    pub fn new() -> Vao {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }

        Vao { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

/// # Buffer Object
/// An object for storing data
///
/// ## Example
/// ```
/// let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::STATIC_DRAW);
/// vbo.bind();
///
/// vbo.store_f32_data(&float32_array);
/// ```
pub struct BufferObject {
    id: gl::types::GLuint,
    r#type: gl::types::GLenum,
    usage: gl::types::GLenum,
}

impl BufferObject {
    pub fn new(r#type: gl::types::GLenum, usage: gl::types::GLenum) -> BufferObject {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        BufferObject { id, r#type, usage }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.r#type, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.r#type, 0);
        }
    }

    pub fn store_f32_data(&self, data: &[f32]) {
        unsafe {
            gl::BufferData(
                self.r#type,
                (data.len() * mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &data[0] as *const f32 as *const c_void,
                self.usage,
            );
        }
    }

    pub fn store_i32_data(&self, data: &[i32]) {
        unsafe {
            gl::BufferData(
                self.r#type,
                (data.len() * mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &data[0] as *const i32 as *const c_void,
                self.usage,
            );
        }
    }

    pub fn get_id(&self) -> gl::types::GLuint {//well well well... this function is added post sucking so it sucks
        self.id
    }
}

/// # Vertex Attribute
/// Discribes vertex data(the stuff from the buffer)
///
/// ## Example
/// ```
/// let position_attribute = VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
/// position_attribute.enable()
/// ```
pub struct VertexAttribute {
    index: GLuint,
}

impl VertexAttribute {
    pub fn new(
        index: u32,
        size: i32,
        r#type: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const c_void,
    ) -> VertexAttribute {
        unsafe {
            gl::VertexAttribPointer(index, size, r#type, normalized, stride, pointer);
        }

        VertexAttribute { index }
    }

    pub fn enable(&self) {
        unsafe {
            gl::EnableVertexAttribArray(self.index);
        }
    }

    pub fn disable(&self) {
        unsafe {
            gl::DisableVertexAttribArray(self.index);
        }
    }
}

pub struct ShaderProgram {
    program_handle: u32,
    uniform_ids: HashMap<String, GLint>,
}

#[allow(temporary_cstring_as_ptr)]
impl ShaderProgram {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
        let vertex_shader_source = Self::read_shader_source(vertex_shader_path);
        let fragment_shader_source = Self::read_shader_source(fragment_shader_path);
        let program_handle = unsafe {
            let vertex_shader = Self::compile_shader(&vertex_shader_source, gl::VERTEX_SHADER);
            let fragment_shader = Self::compile_shader(&fragment_shader_source, gl::FRAGMENT_SHADER);
            let handle = gl::CreateProgram();
            gl::AttachShader(handle, vertex_shader);
            gl::AttachShader(handle, fragment_shader);
            gl::LinkProgram(handle);
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            handle
        };

        ShaderProgram {
            program_handle,
            uniform_ids: HashMap::new(),
        }
    }

    fn read_shader_source(path: &str) -> String {
        let mut file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));
        let mut source = String::new();
        file.read_to_string(&mut source)
            .expect("Failed to read shader file");
        source
    }

    fn compile_shader(source: &str, shader_type: GLenum) -> GLuint {
        let shader = unsafe { gl::CreateShader(shader_type) };
        let c_str = CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);
        }
        shader
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_handle);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn create_uniform(&mut self, uniform_name: &str) {//all this really does is like init a uniform and check if your shader actually like need it
        let uniform_location = unsafe {
            gl::GetUniformLocation(
                self.program_handle,
                CString::new(uniform_name).unwrap().as_ptr(),
            )
        };
        if uniform_location < 0 {
            panic!("Cannot locate uniform: {} \n    or issue with frament shader", uniform_name);
        } else {
            println!("sucessfully created unifrom {}", uniform_name);
            self.uniform_ids.insert(uniform_name.to_string(), uniform_location);
        }
    }


    //intrestng things these are they are not mut
    pub fn set_matrix4fv_uniform(&self, uniform_name: &str, matrix: &Matrix4<f32>) {
        //println!("{}", uniform_name);
        unsafe {
            gl::UniformMatrix4fv(
                self.uniform_ids[uniform_name],
                1,
                gl::FALSE,
                matrix.as_ptr(),
            )
        }
    }

    pub fn set_uniform1i(&self, uniform_name: &str, value: &i32) {
        unsafe {
            gl::Uniform1iv(
                self.uniform_ids[uniform_name],
                1,
                value,
            )
        }
    }

    pub fn set_uniform4f(&self, uniform_name: &str, value: &Vector4<f32>) {
        unsafe {
            gl::Uniform4fv(
                self.uniform_ids[uniform_name],
                1,
                value.as_ptr(),
            )
        }
    }

    pub fn set_uniform1f(&self, name: &str, value: f32) {
        if let Some(&location) = self.uniform_ids.get(name) {
            unsafe { gl::Uniform1f(location, value) };
        }
    }

    pub fn enable_depth(&self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }
    }

    pub fn enable_backface_culling(&self) {
        unsafe {
            gl::Enable(gl::CULL_FACE);     // Enable face culling
            gl::CullFace(gl::BACK);        // Cull back faces
            //gl::FrontFace(gl::CCW);        // Use counter-clockwise vertex winding for front faces
        }
    }

    pub fn get_program_handle(&self) -> &u32 {
        &self.program_handle
    }

    pub fn to_string(&self) -> String{
        let mut output = self.program_handle.to_string();
        output = output + ": ";
        for (key, _value) in self.uniform_ids.iter(){
            output.push_str(&key);
            output = output + ", ";
        }
        output
    }

    //computer shader stuff 
    pub fn new_compute(compute_shader_path: &str) -> Self {
        println!("comp path {}", compute_shader_path );
        let compute_shader_source = Self::read_shader_source(compute_shader_path);
        let program_handle = unsafe {
            let compute_shader = Self::compile_shader(&compute_shader_source, gl::COMPUTE_SHADER);
            let handle = gl::CreateProgram();
            gl::AttachShader(handle, compute_shader);
            gl::LinkProgram(handle);
            gl::DeleteShader(compute_shader);
            handle
        };

        println!("successfully made computer shader");
        ShaderProgram {
            program_handle,
            uniform_ids: HashMap::new(),
        }
    }
    
    pub fn dispatch_compute(&self, x: u32, y: u32, z: u32) {
        unsafe {
            gl::DispatchCompute(x, y, z);
            gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
        }
    }
}


//this needs a lot

//uuuugghhhh like idk why I made it mutex here idk man bruh like basically I feel like this whole thing was a waste of time
//to add like
//the smallest amount of functionality
pub struct ShaderManager {
    shaders: Mutex<HashMap<String, Arc<Mutex<ShaderProgram>>>>,//i decided to just make it Arc
}

impl ShaderManager {
    pub fn new() -> Self {
        Self { shaders: Mutex::new(HashMap::new()) }
    }

    pub fn load_shader(&self, name: &str, vert_path: &str, frag_path: &str) -> Arc<Mutex<ShaderProgram>> {//why does this output this again dude idkeven remember prolly useful though
        let mut shaders = self.shaders.lock().unwrap();
        shaders.entry(name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(ShaderProgram::new(vert_path, frag_path))))
            .clone()
    }

    pub fn load_shader_compute(&self, name: &str, comp_path: &str) -> Arc<Mutex<ShaderProgram>> {//why does this output this again dude idkeven remember prolly useful though
        let mut shaders = self.shaders.lock().unwrap();
        shaders.entry(name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(ShaderProgram::new_compute(comp_path))))
            .clone()
    }


    //bro whats the point even in using rust if imma do this like fr smh do better
    pub fn get_shader(&self, name: &str) -> Option<Arc<Mutex<ShaderProgram>>> {
        let shaders = self.shaders.lock().unwrap();
        shaders.get(name).cloned()
    }

    pub fn add_shader(&mut self, name: &str, shader: ShaderProgram){
        let mut shaders = self.shaders.lock().unwrap();
        shaders.entry(name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(shader)));
    }

    pub fn init_forward_plus(&mut self){
        self.add_shader("depth", initialize_depth_shader());
        self.add_shader("light", initialize_light_shader());
    }

    pub fn enable_backface_culling(&mut self, name: &str){
        self.get_shader(name).expect("CANNOT FIND SHADER").lock().unwrap().enable_backface_culling();
    }

    pub fn enable_depth(&mut self, name: &str){
        self.get_shader(name).expect("CANNOT FIND SHADER").lock().unwrap().enable_depth();
    }
}

pub struct Framebuffer { id: GLuint, depth_texture: Rc<depthTexture> }
pub struct depthTexture { id: GLuint, width: u32, height: u32 }//we finna have to deal with the two textures  later bro

impl Framebuffer {
    pub fn new_depth_only(width: u32, height: u32) -> Self {
        let mut fbo: GLuint = 0;
        let mut depth_tex: GLuint = 0;

        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            gl::GenTextures(1, &mut depth_tex);
            gl::BindTexture(gl::TEXTURE_2D, depth_tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as GLint,
                width as GLsizei,
                height as GLsizei,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                depth_tex,
                0,
            );

            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Depth framebuffer is not complete");
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        let depth_texture = Rc::new(depthTexture {
            id: depth_tex,
            width,
            height,
        });

        Self {
            id: fbo,
            depth_texture,
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.id); }
    }

    pub fn unbind() {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    pub fn get_depth_texture(&self) -> Rc<depthTexture> {
        Rc::clone(&self.depth_texture)
    }
}

pub fn run_depth_prepass(
    depth_shader: &ShaderProgram,
    framebuffer: &Framebuffer,
    scene_objects: &[Mesh], // tuple of (VAO, index count)
    light_manager: &mut LightManager,
    width: u32,
    height: u32,
) {
    framebuffer.bind();

    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
    }

    depth_shader.bind();

    for mesh in scene_objects {
        mesh.draw();//like this is better it might be a little diff thought we will see
    }

    ShaderProgram::unbind();
    Framebuffer::unbind();

    light_manager.set_depth_texture(framebuffer.get_depth_texture());
}

fn run_light_pass(
    light_shader: &ShaderProgram,
    scene_objects: &[Mesh],
    light_manager: &LightManager,
    width: u32,
    height: u32,
) {
    Framebuffer::unbind();

    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
    }

    light_shader.bind();

    // Bind depth texture
    let depth_tex = light_manager.get_depth_texture();
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, depth_tex.id);
    }
    light_shader.set_uniform1i("u_depthTex", &0);
    
    // Bind light culling buffers to their respective binding points
    if let Some(culling_buffers) = &light_manager.culling_buffers {
        unsafe {
            // These should match the binding points used in the compute shader
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, culling_buffers.light_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, culling_buffers.light_grid_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, culling_buffers.light_index_buffer.get_id());
        }
        
        // Set the tile size for the fragment shader
        let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();
        light_shader.set_uniform1f("u_tileCountX", tile_count_x as f32);
        light_shader.set_uniform1f("u_tileCountY", tile_count_y as f32);
    }
    
    // Set light count
    light_shader.set_uniform1i("u_lightCount", &(light_manager.lights.len() as i32));

    // Draw scene objects
    for mesh in scene_objects {
        mesh.draw();
    }

    ShaderProgram::unbind();
}

#[derive(Debug)]
pub enum UniformValue {//i need one for vec3 but im 2 lazy to add rn literally then need to add to materials shader shadermanager make trys for it and also materials maganager its ass 2 lazy
    Float(f32),
    Vector4(Vector4<f32>),
    Matrix4(Matrix4<f32>),
    Texture(u32),
    Empty(),
}



//this is like so stupid and useless i know but like yooooooo maybe it will be hype guys
impl TryFrom<f32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Float(value))
    }
}

impl TryFrom<Vector4<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: Vector4<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Vector4(value))
    }
}

impl TryFrom<Matrix4<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: Matrix4<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Matrix4(value))
    }
}

impl TryFrom<u32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Texture(value))
    }
}

//I need... ones for like references to everything
impl TryFrom<&f32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &f32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Float(*value))//ok bro like this is def sus
    }
}

impl TryFrom<&Vector4<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &Vector4<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Vector4(*value))
    }
}

impl TryFrom<&Matrix4<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &Matrix4<f32>) -> Result<Self, Self::Error> {
        //print!("ok {:?}", value);
        Ok(UniformValue::Matrix4(*value))
    }
}

impl TryFrom<&u32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &u32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Texture(*value))
    }
}

impl fmt::Display for UniformValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}
// lightmanager.rs
pub struct Light {
    pub position: [f32; 3],
    pub radius: f32,
}

pub struct LightManager {
    pub lights: Vec<Light>,
    pub depth_texture: Option<Rc<depthTexture>>,
    pub tile_light_indices: Vec<Vec<usize>>, // per-tile light indices (for CPU culling)
    pub culling_buffers: Option<LightCullingBuffers>, // GPU culling buffers
    pub compute_shader: Option<Arc<Mutex<ShaderProgram>>>, // Compute shader for light culling
}

impl LightManager {
    pub fn new() -> Self {
        Self {
            lights: vec![],
            depth_texture: None,
            tile_light_indices: vec![],
            culling_buffers: None,
            compute_shader: None,
        }
    }

    pub fn set_depth_texture(&mut self, texture: Rc<depthTexture>) {
        self.depth_texture = Some(texture);
    }

    pub fn get_depth_texture(&self) -> Rc<depthTexture>{
        self.depth_texture.clone().expect("cant get depth texture")//dude like I HATE that i have to use clone here but optimization gotta come after I get his forward+ bruteforced
    }

    pub fn cpu_tile_light_culling(&mut self, screen_width: u32, screen_height: u32) {
        let tile_size = 16;
        let tiles_x = (screen_width + tile_size - 1) / tile_size;
        let tiles_y = (screen_height + tile_size - 1) / tile_size;
        let num_tiles = (tiles_x * tiles_y) as usize;

        self.tile_light_indices = vec![vec![]; num_tiles];

        for (light_index, light) in self.lights.iter().enumerate() {
            let light_screen_x = (light.position[0] / screen_width as f32 * tiles_x as f32) as u32;
            let light_screen_y = (light.position[1] / screen_height as f32 * tiles_y as f32) as u32;

            for ty in 0..tiles_y {
                for tx in 0..tiles_x {
                    let tile_index = (ty * tiles_x + tx) as usize;
                    // Fake AABB check (placeholder)
                    let tile_center_x = (tx * tile_size + tile_size / 2) as f32;
                    let tile_center_y = (ty * tile_size + tile_size / 2) as f32;

                    let dx = light.position[0] - tile_center_x;
                    let dy = light.position[1] - tile_center_y;
                    let dist2 = dx * dx + dy * dy;

                    if dist2 < light.radius * light.radius {
                        self.tile_light_indices[tile_index].push(light_index);
                    }
                }
            }
        }
    }

    pub fn initialize_gpu_culling(&mut self, width: u32, height: u32, shader_manager: &ShaderManager) {
        // Create compute shader for light culling
        println!("cum poop shader");
        let compute_shader = shader_manager.load_shader_compute(//yoooooooo this shit does not work with the shader is the acrust src only looks at the engine buttttt... who cares bro fix later
            "light_culling", 
            "shaders/light_culling.comp" // Path to your compute shader
        );
        
        // Initialize culling buffers
        let culling_buffers = LightCullingBuffers::new(width, height, self.lights.len() as u32);
        
        self.compute_shader = Some(compute_shader);
        self.culling_buffers = Some(culling_buffers);
    }
    
    pub fn perform_gpu_light_culling(&mut self, view_projection: &Matrix4<f32>, width: u32, height: u32) {
        if let (Some(culling_buffers), Some(compute_shader)) = (&self.culling_buffers, &self.compute_shader) {
            // Bind the light data buffers
            culling_buffers.bind(&self.lights);
            
            // Bind and set up the compute shader
            let shader = compute_shader.lock().unwrap();
            shader.bind();
            
            // Set uniforms for the compute shader
            shader.set_matrix4fv_uniform("u_viewProjection", view_projection);
            shader.set_uniform1i("u_lightCount", &(self.lights.len() as i32));
            
            if let Some(depth_tex) = &self.depth_texture {
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, depth_tex.id);
                }
                shader.set_uniform1i("u_depthTexture", &0);
            }
            
            // Set screen size uniforms
            shader.set_uniform1f("u_screenWidth", width as f32);
            shader.set_uniform1f("u_screenHeight", height as f32);
            
            // Get tile counts for dispatch size
            let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();
            
            // Dispatch compute shader (1 work group per tile)
            shader.dispatch_compute(tile_count_x, tile_count_y, 1);
            
            // Unbind shader
            ShaderProgram::unbind();
        }
    }
}

//rare public functions... do I just add these as like their own things in shadermanager... YES BRO OMG IM SO DUMB
fn initialize_depth_shader() -> ShaderProgram {//i could make this dynamic but like bruh
    print!("erm couldnt do depth");
    ShaderProgram::new("shaders/depth_prepass.vert","shaders/depth_prepass.frag")
}

fn initialize_light_shader() -> ShaderProgram {//i could make this dynamic but like bruh
    print!("erm couldnt do light");
    ShaderProgram::new("shaders/forward_plus.vert","shaders/forward_plus.frag")
}


//i should just... make this... a function so it can just store all this stuff as references or just store it all for me that will prolly be like
//TODO first hing is do above prolly maybe just put it in light_manager or something man idk
fn render_frame(
    scene: &[Mesh],
    depth_shader: &ShaderProgram,
    light_shader: &ShaderProgram,
    light_manager: &mut LightManager,
    view_projection: &Matrix4<f32>, // Added view_projection matrix
    width: u32,
    height: u32,
) {
    let framebuffer = Framebuffer::new_depth_only(width, height);

    // Depth pre-pass
    run_depth_prepass(
        depth_shader,
        &framebuffer,
        &scene,
        light_manager,
        width,
        height,
    );
    
    // Perform GPU-based light culling
    light_manager.perform_gpu_light_culling(view_projection, width, height);

    // Light pass (forward+ rendering)
    run_light_pass(
        light_shader,
        &scene,
        light_manager,
        width,
        height,
    );
}

pub struct LightCullingBuffers {
    light_buffer: BufferObject,          // SSBO for light data
    light_index_buffer: BufferObject,    // SSBO for light indices per tile
    light_grid_buffer: BufferObject,     // SSBO for light grid data
    tile_count_x: u32,
    tile_count_y: u32,
    max_lights_per_tile: u32,
}

impl LightCullingBuffers {
    pub fn new(width: u32, height: u32, max_lights: u32) -> Self {
        let tile_size = 16; // Tile size, same as in your CPU implementation
        let tile_count_x = (width + tile_size - 1) / tile_size;
        let tile_count_y = (height + tile_size - 1) / tile_size;
        let max_lights_per_tile = 64; // Can be adjusted based on your needs
        
        // Create SSBO for light data
        let light_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        
        // Create SSBO for light indices
        let light_index_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        
        // Create SSBO for light grid
        let light_grid_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        
        Self {
            light_buffer,
            light_index_buffer,
            light_grid_buffer,
            tile_count_x,
            tile_count_y,
            max_lights_per_tile,
        }
    }
    
    pub fn bind(&self, lights: &[Light]) {
        // Prepare light data for GPU
        let mut light_data: Vec<f32> = Vec::new();
        for light in lights {
            // Pack light data: position (xyz) + radius
            light_data.push(light.position[0]);
            light_data.push(light.position[1]);
            light_data.push(light.position[2]);
            light_data.push(light.radius);
            // Add more light properties as needed (color, intensity, etc.)
        }
        
        // Bind and upload light data
        self.light_buffer.bind();
        self.light_buffer.store_f32_data(&light_data);
        
        // Prepare light grid buffer (will be filled by compute shader)
        let total_tiles = (self.tile_count_x * self.tile_count_y) as usize;
        let grid_size = total_tiles * 2; // Each tile has (offset, count)
        let mut grid_data = vec![0i32; grid_size];
        
        self.light_grid_buffer.bind();
        self.light_grid_buffer.store_i32_data(&grid_data);
        
        // Prepare light index buffer (will be filled by compute shader)
        let index_buffer_size = total_tiles * self.max_lights_per_tile as usize;
        let mut index_data = vec![0i32; index_buffer_size];
        
        self.light_index_buffer.bind();
        self.light_index_buffer.store_i32_data(&index_data);
        
        // Bind the buffers to their respective binding points
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.light_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.light_grid_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, self.light_index_buffer.get_id());
        }
    }
    
    pub fn get_tile_counts(&self) -> (u32, u32) {
        (self.tile_count_x, self.tile_count_y)
    }
}