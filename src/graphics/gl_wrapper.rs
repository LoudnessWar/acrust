use std::cmp::min;
use std::fmt;
use std::collections::HashMap;
use std::mem;
use std::os::raw::*;
use std::ffi::CString;
use std::fs::File;
use std::ptr;
use std::io::Read;
use std::rc::Rc;
// use std::thread::panicking;
use std::vec;
// use std::sync::PoisonError;
use cgmath::*;
use gl::types::*;
// use gl::TextureParameterfv;
// use gl::SHADER;
use std::sync::Mutex;
use std::sync::Arc;

use crate::graphics::weighted_oit::WeightedOIT;
// use crate::model::mesh;
use crate::model::mesh::Mesh;
use crate::model::objload::ModelTrait;

// use super::camera;
use super::camera::Camera;
use super::texture_manager::TextureManager;


macro_rules! gl_check {
    ($call:expr) => {{
        $call;
        #[allow(unused_unsafe)]
        let err = unsafe { gl::GetError() };
        if err != gl::NO_ERROR {
            panic!("GL Error {:#x} in `{}` at line {}", err, stringify!($call), line!());
        }
    }}
}

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
        println!("new VAO made");
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
            let err = gl::GetError();
            if err != gl::NO_ERROR {
                panic!("GL Error during GenBuffers: 0x{:x}", err);
            }
        }
        println!("new buffer made");
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
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, 0); // unbind
        }
        self.bind();
        let size = (data.len() * mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr;
        //println!("Buffer ID: {}, Size: {}", self.id, size);
        //println!("Binding buffer {} to target {:#X}", self.id, self.r#type);
        //print!("data {:#?}", data.len());

        unsafe {
            let ctx_err = gl::GetError();
            if ctx_err != gl::NO_ERROR {
                println!("GL context error before BufferData: 0x{:X}", ctx_err);
            }

            gl_check!(gl::BufferData(self.r#type, size, &data[0] as *const _ as *const c_void, self.usage));
            let err = gl::GetError();
            if err != gl::NO_ERROR {
                panic!("GL ERROR in BufferData: 0x{:X} for ID: {}", err, self.id);
            }
        }
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.id);
        }
    }

    pub fn store_i32_data(&self, data: &[i32]) {
        let size = (data.len() * mem::size_of::<gl::types::GLint>()) as gl::types::GLsizeiptr;
        //ln!("Buffer I32 ID: {}, Size: {} ", self.id, size);
        //print!("data {:#?}", data.len());
        unsafe {
            gl_check!(gl::BufferData(
                self.r#type,
                size,
                &data[0] as *const i32 as *const c_void,
                self.usage,
            ));
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
    index: GLuint,//lol very bare bones ik
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
        println!("new VertexAttribute made");
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
    uniform_ids: HashMap<String, GLint>,//lol
}

// #[allow(temporary_cstring_as_ptr)]
impl ShaderProgram {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Self {
        println!("created a new shader");
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
        //println!("Source: {}, Type: {}", source, shader_type);
        let shader = unsafe { gl::CreateShader(shader_type) };
        let c_str = CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);
        }
        let mut status = gl::FALSE as GLint;
        unsafe {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            if status != gl::TRUE as GLint {
                let mut len: GLint = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "Shader compilation failed: {}",
                    String::from_utf8_lossy(&buffer)
                );
            }
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
        //print!("name {}", uniform_name);
        let uniform_location = unsafe {
            gl::GetUniformLocation(
                self.program_handle,
                CString::new(uniform_name).unwrap().as_ptr(),
            )
        };
        if uniform_location < 0 {
            panic!("Cannot locate uniform: {} \n    or issue with frament shader", uniform_name);
        } else {
            //println!("sucessfully created unifrom {}", uniform_name);
            self.uniform_ids.insert(uniform_name.to_string(), uniform_location);
        }
    }

    pub fn create_uniforms(&mut self, keys_vector: Vec<&str>) {//all this really does is like init a uniform and check if your shader actually like need it
        for key in keys_vector.into_iter() {
            if !self.uniform_ids.contains_key(key) {
                self.create_uniform(key);
            } else {
                println!("Key: {} Already Defined-So Skipping", key)
            }
        }
    }

    // pub fn get_uniform_type(&self, uniform_name: &str) -> GLenum {
    // let location = self.uniform_ids[uniform_name];
    // let mut uniform_type: GLint = 0;
    // unsafe {
    //     gl::GetActiveUniform(
    //         self.program_handle,
    //         location as GLuint,
    //         0, // nameLength parameter not used
    //         std::ptr::null_mut(), // don't need length
    //         std::ptr::null_mut(), // don't need size
    //         &mut uniform_type, // this is what we want
    //         std::ptr::null_mut(), // don't need name
    //     );
    // }
    // uniform_type as GLenum
    // }



    //rules for new set... add to mat add to mat man add to enum add to shader man 

    //intrestng things these are they are not mut
    pub fn set_matrix4fv_uniform(&self, uniform_name: &str, matrix: &Matrix4<f32>) {
        //println!("{}", uniform_name);
        unsafe {
            gl::UniformMatrix4fv(
                self.uniform_ids[uniform_name],
                1,
                gl::FALSE,
                matrix.as_ptr(),
            );
            let err: u32 = gl::GetError();
            if err != gl::NO_ERROR {
                panic!("GL ERROR in BufferData: 0x{:X} for Name: {} with Value: {:?}", err, uniform_name, matrix);
            }
        }
    }


    //idk what the diff between these two is... we need to add thsi one to material and stuff as well onjfod
    pub fn set_uniform1iv(&self, uniform_name: &str, value: &i32) {
        //println!("try Uniform1iv :{}", uniform_name);
        unsafe {
            gl::Uniform1iv(
                self.uniform_ids[uniform_name],
                1,
                value,
            );
            let err: u32 = gl::GetError();
            if err != gl::NO_ERROR {
                panic!("GL ERROR in BufferData: 0x{:X} for Name: {} with Value: {}", err, uniform_name, value);
            }
        }
        //println!("set Uniform1iv :{}", uniform_name);
    }

    pub fn set_uniform1i(&self, uniform_name: &str, value: &i32) {
        //println!("try Uniform1i :{}", uniform_name);
        unsafe {
            // Change from Uniform1iv to Uniform1i for a single integer
            gl::Uniform1i(
                self.uniform_ids[uniform_name],
                *value,
            );
            let err: u32 = gl::GetError();
            if err != gl::NO_ERROR {
                panic!("GL ERROR in BufferData: 0x{:X} for Name: {} with Value: {}", err, uniform_name, value);
            }
        }
        //println!("set Uniform1i :{}", uniform_name);
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
            unsafe { 
                gl::Uniform1f(location, value);
                let err: u32 = gl::GetError();
                if err != gl::NO_ERROR {
                    panic!("GL ERROR in BufferData: 0x{:X} for Name: {} with Value: {}", err, name, value);
                }
            };
        }
    }

    pub fn set_uniform3f(&self, name: &str, value: &Vector3<f32>) {
        unsafe {
            gl::Uniform3fv(
                self.uniform_ids[name],
                1,
                value.as_ptr(),
            )
        }
    }

    pub fn enable_depth() {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
        }
    }

    pub fn enable_backface_culling() {
        unsafe {
            gl::Enable(gl::CULL_FACE);     // Enable face culling
            gl::CullFace(gl::BACK);        // Cull back faces
            gl::FrontFace(gl::CCW);        // Use counter-clockwise vertex winding for front faces
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

        // let mut status = gl::FALSE as GLint;
        // unsafe {
        //     gl::GetShaderiv(program_handle, gl::COMPILE_STATUS, &mut status);
        //     if status != gl::TRUE as GLint {
        //         let mut len: GLint = 0;
        //         gl::GetShaderiv(program_handle, gl::INFO_LOG_LENGTH, &mut len);
        //         let mut buffer = vec![0u8; len as usize];
        //         gl::GetShaderInfoLog(
        //             program_handle,
        //             len,
        //             std::ptr::null_mut(),
        //             buffer.as_mut_ptr() as *mut GLchar,
        //         );
        //         panic!(
        //             "Shader compilation failed: {}",
        //             String::from_utf8_lossy(&buffer)
        //         );
        //     }
        // }

        //println!("successfully made computer shader");
        ShaderProgram {
            program_handle,
            uniform_ids: HashMap::new(),
        }
    }
    
    pub fn dispatch_compute(&self, x: u32, y: u32, z: u32) {
        //print!("run compute");
        unsafe {
            gl_check!(gl::DispatchCompute(x, y, z));
                    // Insert fence after compute dispatch
            let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0); //TODO lol we are creating a new fence every time here which is fine for now becasue we are deleteing it but later on will not be chill

            // Wait for the GPU to complete compute
            let wait_result = gl::ClientWaitSync(fence, gl::SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000); // 1 sec
            if wait_result == gl::TIMEOUT_EXPIRED {
                println!("⚠️ Compute shader did not complete in time. 😭😭😭😭😭😭😭😭😭😭");
            }

            gl::DeleteSync(fence);
            //gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);
            //gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            //gl_check!(gl::Finish());//before here
        }
    }

    pub fn debug_print_uniforms(&self) {
        let mut count: GLint = 0;
        unsafe {
            gl::GetProgramiv(self.program_handle, gl::ACTIVE_UNIFORMS, &mut count);
        }
        println!("Shader program ID: {}: ", self.get_program_handle());
        for i in 0..count {
            let mut name_buf = vec![0u8; 256];
            let mut length: GLsizei = 0;
            let mut size: GLint = 0;
            let mut utype: GLenum = 0;
    
            unsafe {
                gl::GetActiveUniform(
                    self.program_handle,
                    i as GLuint,
                    256,
                    &mut length,
                    &mut size,
                    &mut utype,
                    name_buf.as_mut_ptr() as *mut GLchar,
                );
            }
            
    
            let name = String::from_utf8_lossy(&name_buf[..length as usize]);
            let loc = unsafe {
                gl::GetUniformLocation(self.program_handle, CString::new(name.as_bytes()).unwrap().as_ptr())
            };
            println!("Uniform {} (type 0x{:x}) at location {}", name, utype, loc);
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

    //this is all like initializing debug stuff
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());
        }
        
        println!("3:1");

        extern "system" fn debug_callback(
            source: GLenum,
            type_: GLenum,
            id: GLuint,
            severity: GLenum,
            _length: GLsizei,
            message: *const GLchar,
            _user_param: *mut c_void,
        ) {
            unsafe {
                let string = std::ffi::CStr::from_ptr(message).to_string_lossy();
                println!("GL CALLBACK: source = {}, type = {}, id = {}, severity = {}, message = {}",
                         source, type_, id, severity, string);
            }
        }

        println!("3:2");
        self.add_shader("depth", initialize_depth_shader());
        println!("3:3");
        self.add_shader("light", initialize_light_shader());//EDIT EDIT EDIT

        println!("3:4");
    }

    pub fn init_forward_plus_light_debug(&mut self){
        //this is all like initializing debug stuff
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());
        }
        
        extern "system" fn debug_callback(
            source: GLenum,
            type_: GLenum,
            id: GLuint,
            severity: GLenum,
            _length: GLsizei,
            message: *const GLchar,
            _user_param: *mut c_void,
        ) {
            unsafe {
                let string = std::ffi::CStr::from_ptr(message).to_string_lossy();
                println!("GL CALLBACK: source = {}, type = {}, id = {}, severity = {}, message = {}",
                            source, type_, id, severity, string);
            }
        }


        self.add_shader("depth", initialize_depth_shader());
        self.add_shader("light", initialize_light_shader_debug());
    }

    // pub fn enable_backface_culling(&mut self, name: &str){
    //     self.get_shader(name).expect("CANNOT FIND SHADER").lock().unwrap().enable_backface_culling();
    // }

    // pub fn enable_depth(&mut self, name: &str){
    //     self.get_shader(name).expect("CANNOT FIND SHADER").lock().unwrap().enable_depth();
    // }

    pub fn enable_backface_culling(){
        ShaderProgram::enable_backface_culling();
    }

    pub fn enable_depth(){
        ShaderProgram::enable_depth();
    }

    pub fn enable_z_depth(){//todo add this later
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::GREATER);      // Reverse Z!
            gl::ClearDepth(0.0);             // Reverse Z clears to 0 (max far plane)
        }
    }
}

pub struct Framebuffer { id: GLuint, depth_texture: Rc<depthTexture> }
#[allow(non_camel_case_types)]
pub struct depthTexture { id: GLuint, width: u32, height: u32 }//todo we finna have to deal with the two textures later bro

impl Drop for depthTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}

impl Framebuffer {
    pub fn new_depth_only(width: u32, height: u32) -> Self {
        let mut fbo: GLuint = 0;
        let mut depth_tex: GLuint = 0;

        unsafe {
            gl_check!(gl::GenFramebuffers(1, &mut fbo));
            //gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            gl::GenTextures(1, &mut depth_tex);
            gl::BindTexture(gl::TEXTURE_2D, depth_tex);
            gl_check!(gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as GLint,//maybe32f here or whatever
                width as GLsizei,
                height as GLsizei,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            ));

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);//idk if tgese do much
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            //let borderColor: [f32; 4]= [1.0, 1.0, 1.0, 1.0];

            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, [1.0, 1.0, 1.0, 1.0].as_ptr());//giving error for some reason// what is the point of this even is it like to make it so that 
            //it knowns where to cull based off the white border?

            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            
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

        unsafe {
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer incomplete: {:x}", status);
            }
        }

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
    models_iter: &Vec<&&Box<dyn ModelTrait>>,
    framebuffer: Rc<depthTexture>,
    //scene_objects: &Vec<&Mesh>,
    light_manager: &mut LightManager,//idk why this isnt just a return tbh
    width: u32,
    height: u32,
) {
    
    unsafe {
        gl_check!(gl::Viewport(0, 0, width as i32, height as i32));
        gl::Clear(gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);//need this for it to work even thought this should already be enabled?
    }

    // unsafe {
    //     gl::ClearDepthf(1.0); // Clear to the "far" value
    //     gl::Clear(gl::DEPTH_BUFFER_BIT);
    // }

    depth_shader.bind();

    unsafe {
        gl::DepthFunc(gl::LEQUAL); // Use LESS or LEQUAL based on your needs
        gl::DepthMask(gl::TRUE); // Ensure depth writing is enabled
        //gl_check!(gl::Enable(gl::MULTISAMPLE));//idk if this is needed looking at it it doesnt change much

    }

    for model in models_iter {
        depth_shader.set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());
        let mesh = model.get_mesh();
        mesh.draw();//like this is better it might be a little diff thought we will see
    }



    light_manager.set_depth_texture(framebuffer);
}

// Replace run_depth_prepass temporarily
pub fn run_depth_debug_pass(
    debug_shader: &ShaderProgram,
    scene_objects: &Vec<&Mesh>,
    view_proj: &Camera,
    model_matrices: &[Matrix4<f32>],
) {
    // unsafe {
    //     gl::DepthFunc(gl::ALWAYS);
    //     gl::Disable(gl::CULL_FACE);
    //     gl::FrontFace(gl::CW);
    //     gl::BindFramebuffer(gl::FRAMEBUFFER, 0); // draw to screen
    //     gl::Viewport(0, 0, 720, 720);
    //     gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    //     gl::Enable(gl::DEPTH_TEST);
    // }

    // unsafe {
    //     gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    //     gl::Viewport(0, 0, 720, 720);
    //     gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    //     gl::Disable(gl::DEPTH_TEST);
    //     gl::Disable(gl::CULL_FACE);
    // }

    debug_shader.bind();
    debug_shader.set_matrix4fv_uniform("view", view_proj.get_view());
    debug_shader.set_matrix4fv_uniform("projection", view_proj.get_p_matrix());

    debug_shader.set_uniform1f("near", 4.0);
    debug_shader.set_uniform1f("far", 1000.0);

    for (mesh, model) in scene_objects.iter().zip(model_matrices.iter()) {
        debug_shader.set_matrix4fv_uniform("model", model);
        mesh.draw();
    }

    ShaderProgram::unbind();
}


// pub fn run_light_pass(
//     light_shader: &ShaderProgram,
//     scene_objects: &[Mesh],
//     light_manager: &LightManager,
//     width: u32,
//     height: u32,
// ) {
//     Framebuffer::unbind();

//     unsafe {
//         gl::Viewport(0, 0, width as i32, height as i32);
//         gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
//         gl::Enable(gl::DEPTH_TEST);
//     }

//     light_shader.bind();

//     // Bind depth texture
//     let depth_tex = light_manager.get_depth_texture();
//     unsafe {
//         gl::ActiveTexture(gl::TEXTURE0);
//         gl::BindTexture(gl::TEXTURE_2D, depth_tex.id);
//     }
    
//     light_shader.set_uniform1i("u_depthTex", &(depth_tex.id as i32));//idk if I or IV is right also... what why is it 0
    
//     // Bind light culling buffers to their respective binding points
//     if let Some(culling_buffers) = &light_manager.culling_buffers {
//         unsafe {
//             // These should match the binding points used in the compute shader
//             gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, culling_buffers.light_buffer.get_id());
//             gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, culling_buffers.light_grid_buffer.get_id());
//             gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, culling_buffers.light_index_buffer.get_id());
//         }
        
//         // Set the tile size for the fragment shader
//         let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();
//         light_shader.set_uniform1f("u_tileCountX", tile_count_x as f32);
//         //light_shader.set_uniform1f("u_tileCountY", tile_count_y as f32);
//     }
    
//     // Set light count
//     light_shader.set_uniform1i("u_lightCount", &(light_manager.lights.len() as i32));

//     // Draw scene objects
//     for mesh in scene_objects {
//         mesh.draw();
//     }

//     ShaderProgram::unbind();
// }

#[derive(Debug)]
pub enum UniformValue {//i need one for vec3 but im 2 lazy to add rn literally then need to add to materials shader shadermanager make trys for it and also materials maganager its ass 2 lazy
    Float(f32),
    Int(i32),//erm should it be something else?
    Vector4(Vector4<f32>),
    Vector3(Vector3<f32>),
    Matrix4(Matrix4<f32>),
    Texture(u32),
    Empty(),
}

//this is like so stupid and useless i know but like yooooooo maybe it will be hype guys
//i mean not really but also it looks better for the user
impl TryFrom<f32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Float(value))
    }
}

impl TryFrom<i32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Int(value))
    }
}

impl TryFrom<Vector4<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: Vector4<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Vector4(value))
    }
}

impl TryFrom<Vector3<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: Vector3<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Vector3(value))
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

impl TryFrom<&i32> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &i32) -> Result<Self, Self::Error> {
        Ok(UniformValue::Int(*value))//ok bro like this is def sus
    }
}

impl TryFrom<&Vector4<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &Vector4<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Vector4(*value))
    }
}

impl TryFrom<&Vector3<f32>> for UniformValue {
    type Error = &'static str;

    fn try_from(value: &Vector3<f32>) -> Result<Self, Self::Error> {
        Ok(UniformValue::Vector3(*value))
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
    pub color: [f32; 3],
    pub intensity: f32,
}


//TODO  I want this to be a little more robust eventully because like... Its only capable of like helping fpr rn tbh
//fpr should have an option to take in a light manager and not just makes its own if we are being fr
pub struct LightManager {
    pub lights: Vec<Light>,
    pub depth_texture: Option<Rc<depthTexture>>,
    pub tile_light_indices: Vec<Vec<usize>>, // per-tile light indices (for CPU culling)
    pub culling_buffers: Option<LightCullingBuffers>, // GPU culling buffers
    pub compute_shader: Option<Arc<Mutex<ShaderProgram>>>, // Compute shader for light culling
    pub debug_texture: Option<GLuint>,//remove this later TODO
}

impl LightManager {
    pub fn new() -> Self {
        Self {
            lights: vec![],//bruh im stupid
            depth_texture: None,
            tile_light_indices: vec![],
            culling_buffers: None,
            compute_shader: None,
            debug_texture: None,
        }
    }

    pub fn set_depth_texture(&mut self, texture: Rc<depthTexture>) {
        self.depth_texture = Some(texture);//this is ideally from FBO
    }

    pub fn get_depth_texture(&self) -> Rc<depthTexture>{
        self.depth_texture.clone().expect("cant get depth texture")//dude like I HATE that i have to use clone here but optimization gotta come after I get his forward+ bruteforced
    }

    // pub fn cpu_tile_light_culling(&mut self, screen_width: u32, screen_height: u32) {
    //     let tile_size = 16;
    //     let tiles_x = (screen_width + tile_size - 1) / tile_size;
    //     let tiles_y = (screen_height + tile_size - 1) / tile_size;
    //     let num_tiles = (tiles_x * tiles_y) as usize;

    //     self.tile_light_indices = vec![vec![]; num_tiles];

    //     for (light_index, light) in self.lights.iter().enumerate() {
    //         let light_screen_x = (light.position[0] / screen_width as f32 * tiles_x as f32) as u32;
    //         let light_screen_y = (light.position[1] / screen_height as f32 * tiles_y as f32) as u32;

    //         for ty in 0..tiles_y {
    //             for tx in 0..tiles_x {
    //                 let tile_index = (ty * tiles_x + tx) as usize;
    //                 // Fake AABB check (placeholder)
    //                 let tile_center_x = (tx * tile_size + tile_size / 2) as f32;
    //                 let tile_center_y = (ty * tile_size + tile_size / 2) as f32;

    //                 let dx = light.position[0] - tile_center_x;
    //                 let dy = light.position[1] - tile_center_y;
    //                 let dist2 = dx * dx + dy * dy;

    //                 if dist2 < light.radius * light.radius {
    //                     self.tile_light_indices[tile_index].push(light_index);
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn initialize_gpu_culling(&mut self, width: u32, height: u32, shader_manager: &ShaderManager) {
        // Create compute shader for light culling
        //println!("cum poop shader");
        let compute_shader = shader_manager.load_shader_compute(//yoooooooo this shit does not work with the shader is the acrust src only looks at the engine buttttt... who cares bro fix later
            "light_culling", 
            "shaders/comp_debug.comp" // Path to compute shader that should prolly change
        );
        
        // Initialize culling buffers
        let culling_buffers = LightCullingBuffers::new(width, height, self.lights.len() as u32);//TODO only have one light initializtion we got 2 rn for fpr and lightmangaer
        
        compute_shader.lock().expect("Failed to lock computer shader").create_uniform("view");//TODO as to why I create the uniforms here... idk im dumb I will move later in fact TODO move later to comp
        compute_shader.lock().expect("Failed to lock computer shader").create_uniform("projection");

        compute_shader.lock().expect("Failed to lock computer shader").create_uniform("u_lightCount");
        //compute_shader.lock().expect("Failed to lock computer shader").create_uniform("u_viewProjection");
        compute_shader.lock().expect("Failed to lock computer shader").create_uniform("u_screenWidth");
        compute_shader.lock().expect("Failed to lock computer shader").create_uniform("u_screenHeight");
        compute_shader.lock().expect("Failed to lock computer shader").create_uniform("u_depthTexture");

        // These onl;e need to happen on init
        compute_shader.lock().expect("Failed to lock computer shader").bind();
        compute_shader.lock().expect("Failed to lock computer shader").set_uniform1f("u_screenWidth", width as f32);
        compute_shader.lock().expect("Failed to lock computer shader").set_uniform1f("u_screenHeight", height as f32);

        ShaderProgram::unbind();
        self.compute_shader = Some(compute_shader);//ok like it doesnt really need to be like a option
        self.culling_buffers = Some(culling_buffers);
    }
    

    //there was width and height here but now its gone omg
    pub fn perform_gpu_light_culling(&mut self, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        if let (Some(culling_buffers), Some(compute_shader)) = (&self.culling_buffers, &self.compute_shader) {
            // Bind the light data buffers
            culling_buffers.bind(&self.lights);

            let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();
            
            // Bind and set up the compute shader
            #[allow(unused_mut)]//yeah lol this should be mut because i am actually mutating it just in unsafe
            let mut shader = compute_shader.lock().unwrap();
            shader.bind();
            //shader.create_uniform("u_viewProjection");
            // Set uniforms for the compute shader
            //shader.set_matrix4fv_uniform("u_viewProjection", view_projection);
            shader.set_matrix4fv_uniform("view", view);
            shader.set_matrix4fv_uniform("projection", projection);
            shader.set_uniform1i("u_lightCount", &(self.lights.len() as i32));
            
            if let Some(depth_tex) = &self.depth_texture {
                unsafe {
                    gl_check!(gl::ActiveTexture(gl::TEXTURE0));//after here
                    shader.set_uniform1iv("u_depthTexture", &0);//do i need this? idk but crying emoji this was the error i spend 2 days on needed to be 0 😭😭😭😭😭😭😭😭😭😭😭😭😭😭😭😭
                    gl_check!(gl::BindTexture(gl::TEXTURE_2D, depth_tex.id));
                }
            }
            
            shader.dispatch_compute(tile_count_x, tile_count_y, 1);
            // Set screen size uniforms
            // shader.set_uniform1f("u_screenWidth", width as f32);
            // shader.set_uniform1f("u_screenHeight", height as f32);
            
            // Get tile counts for dispatch size
            //let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();
            //self.debug_comp(tile_count_x, tile_count_y);
            // Dispatch compute shader (1 work group per tile)
            
            //println!("Compute dispatched for tiles: {} x {}", tile_count_x, tile_count_y);

            //self.debug_read_buffers();
            // unsafe {
            //     gl::BindTexture(gl::TEXTURE_2D, self.debug_texture.unwrap());
            //     let mut data = vec![0.0f32; (tile_count_x * tile_count_y * 4) as usize];
            //     gl::GetTexImage(
            //         gl::TEXTURE_2D,
            //         0,
            //         gl::RGBA,
            //         gl::FLOAT,
            //         data.as_mut_ptr() as *mut c_void,
            //     );
            
            //     for i in 0..10 {
            //         println!(
            //             "Pixel {}: {:?}",
            //             i,
            //             &data[(i * 4)..(i * 4 + 4)]
            //         );
            //     }
            // }

            unsafe {
                gl_check!(gl::ActiveTexture(gl::TEXTURE0));
                gl::BindTexture(gl::TEXTURE_2D, 0);
                //gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                //todo lol got rid of this because it was clearing the skybox when we replaced in the in the transparent pass
                //this might be needed in fact idk
                //i forgot why it was here but im pretty sure it was just a saftey mesure
                //TODO yeah look at this
            }

            
            // Unbind shader
            ShaderProgram::unbind();
        }
    }

    pub fn debug_perform_gpu_light_culling(&mut self, view: &Matrix4<f32>, projection: &Matrix4<f32>) -> Option<GLuint> {
        let mut debug_tex = None;
        
        if let (Some(culling_buffers), Some(compute_shader)) = (&self.culling_buffers, &self.compute_shader) {
            // Bind the light data buffers
            culling_buffers.bind(&self.lights);
            
            // Get tile counts for dispatch size
            let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();
            
            // Create debug texture
            let tex_id = self.create_debug_texture(tile_count_x, tile_count_y);
            debug_tex = Some(tex_id);
            self.debug_texture = Some(tex_id); // I have a bomb if you are reading this

            
            // Bind and set up the compute shader
            #[allow(unused_mut)]//yeah lol this should be mut because i am actually mutating it just in unsafe
            let mut shader = compute_shader.lock().unwrap();
            shader.bind();
            
            // Set uniforms for the compute shader
            shader.set_matrix4fv_uniform("view", view);
            shader.set_matrix4fv_uniform("projection", projection);
            shader.set_uniform1i("u_lightCount", &(self.lights.len() as i32));
            
            if let Some(depth_tex) = &self.depth_texture {
                unsafe {
                    gl_check!(gl::ActiveTexture(gl::TEXTURE0));
                    shader.set_uniform1iv("u_depthTexture", &0);
                    gl_check!(gl::BindTexture(gl::TEXTURE_2D, depth_tex.id));
                }
            }
        
            
            // Dispatch compute shader (1 work group per tile)
            shader.dispatch_compute(tile_count_x, tile_count_y, 1);
            //self.debug_read_buffers();
            
            // Memory barrier to ensure writes are completed idk if this is doing much TODO or the unsafe below
            unsafe {
                gl_check!(gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT));
            }

            unsafe {
                gl_check!(gl::ActiveTexture(gl::TEXTURE0));
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            
            // Unbind shader
            ShaderProgram::unbind();
        }
        
        debug_tex
    }

    // pub fn debug_comp(&mut self, tile_count_x: u32, tile_count_y: u32) {
    //     let mut debug_tex: GLuint = 0;
    //     unsafe {
    //         gl_check!(gl::GenTextures(1, &mut debug_tex));
    //         gl::BindTexture(gl::TEXTURE_2D, debug_tex);
    //         gl::TexImage2D(
    //             gl::TEXTURE_2D,
    //             0,
    //             gl::RGBA32F as GLint,
    //             tile_count_x as GLsizei,
    //             tile_count_y as GLsizei,
    //             0,
    //             gl::RGBA,
    //             gl::FLOAT,
    //             std::ptr::null(),
    //         );
    //         gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
    //         gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
    //         gl_check!(gl::BindImageTexture(3, debug_tex, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F));
    //     }
        
    //     self.debug_texture = Some(debug_tex);
    // }

    // pub fn render_debug_visualization(&self, texture_id: GLuint, width: u32, height: u32, debug_shader: &mut ShaderProgram) {
    //     let debug_width = width / 4;  // 1/4 of screen width
    //     let debug_height = height / 4; // 1/4 of screen height
        
    //     unsafe {
    //         gl::Viewport(
    //             (width - debug_width) as i32, 
    //             0, 
    //             debug_width as i32, 
    //             debug_height as i32
    //         );
    //     }

        
    //     if let Some(debug_tex) = self.debug_texture {
    //         // Create a simple fullscreen quad shader if you don't have one
    //         //let debug_shader = LightManager::create_debug_display_shader();
    //         debug_shader.bind();

    //         let vertices: [f32; 8] = [
    //             -1.0, -1.0,
    //              1.0, -1.0,
    //             -1.0,  1.0,
    //              1.0,  1.0,
    //         ];

    //         let mut vao: GLuint = 0;
    //         let mut vbo: GLuint = 0;

    //         unsafe {
    //             gl::GenVertexArrays(1, &mut vao);
    //             gl::GenBuffers(1, &mut vbo);

    //             gl::BindVertexArray(vao);
    //             gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    //             gl::BufferData(
    //                 gl::ARRAY_BUFFER,
    //                 (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
    //                 vertices.as_ptr() as *const c_void,
    //                 gl::STATIC_DRAW,
    //             );

    //             gl::EnableVertexAttribArray(0);
    //             gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<f32>() as GLsizei, ptr::null());

    //             gl::Disable(gl::DEPTH_TEST);

    //             gl::UseProgram(debug_shader.get_program_handle().clone());

    //             gl::ActiveTexture(gl::TEXTURE0);
    //             gl::BindTexture(gl::TEXTURE_2D, debug_tex);
    //             //debug_shader.create_uniform("debugTexture");
    //             debug_shader.set_uniform1i("debugTexture", &0);

    //             gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    //             gl::BindVertexArray(0);
    //             gl::DeleteBuffers(1, &vbo);
    //             gl::DeleteVertexArrays(1, &vao);

    //             gl::Enable(gl::DEPTH_TEST);
    //         }
                        
    //         // Create a VAO for the fullscreen quad (can be cached and reused)
    //         // let mut vao: GLuint = 0;
    //         // let mut vbo: GLuint = 0;
            
    //         // // Define fullscreen quad vertices (position and texture coordinates)
    //         // let vertices: [f32; 20] = [
    //         //     // Position (3) and TexCoord (2)
    //         //     -1.0, -1.0, 0.0,   0.0, 0.0,
    //         //      1.0, -1.0, 0.0,   1.0, 0.0,
    //         //     -1.0,  1.0, 0.0,   0.0, 1.0,
    //         //      1.0,  1.0, 0.0,   1.0, 1.0,
    //         // ];
            
    //         // unsafe {
    //         //     // Disable depth testing for this debug render
    //         //     gl::Disable(gl::DEPTH_TEST);
                
    //         //     // Create and set up VAO and VBO
    //         //     gl::GenVertexArrays(1, &mut vao);
    //         //     gl::GenBuffers(1, &mut vbo);
                
    //         //     gl::BindVertexArray(vao);
    //         //     gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    //         //     gl::BufferData(
    //         //         gl::ARRAY_BUFFER,
    //         //         (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
    //         //         vertices.as_ptr() as *const std::ffi::c_void,
    //         //         gl::STATIC_DRAW
    //         //     );
                
    //         //     // Position attribute
    //         //     gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 5 * std::mem::size_of::<f32>() as GLsizei, std::ptr::null());
    //         //     gl::EnableVertexAttribArray(0);
                
    //         //     // Texture coordinate attribute
    //         //     gl::VertexAttribPointer(
    //         //         1, 
    //         //         2, 
    //         //         gl::FLOAT, 
    //         //         gl::FALSE, 
    //         //         5 * std::mem::size_of::<f32>() as GLsizei, 
    //         //         (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void
    //         //     );
    //         //     gl::EnableVertexAttribArray(1);
                
    //         //     // Bind the debug texture
    //         //     gl::ActiveTexture(gl::TEXTURE0);
    //         //     gl::BindTexture(gl::TEXTURE_2D, debug_tex);
    //         //     gl::GetError();
                
    //         //     // Set the shader uniform
    //         //     debug_shader.set_uniform1i("debugTexture", &0);
                
    //         //     // Draw the quad
    //         //     gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                
    //         //     // Clean up
    //         //     gl::BindVertexArray(0);
    //         //     gl::DeleteVertexArrays(1, &vao);
    //         //     gl::DeleteBuffers(1, &vbo);
                
    //         //     // Re-enable depth testing
    //         //     //gl::Enable(gl::DEPTH_TEST);add back
    //         // }
            
    //         ShaderProgram::unbind();
    //     }

    //     unsafe {
    //         gl::Viewport(0, 0, width as i32, height as i32);
    //     }
    // }

    pub fn create_debug_display_shader() -> ShaderProgram {//lol this is just me being lazy
        let vertex_src = "shaders/debug_comp.vert";
        
        let fragment_src = "shaders/debug_comp.frag";
        
        // Create shader program
        let mut shader = ShaderProgram::new(vertex_src, fragment_src);
        shader.create_uniform("debugTexture");
        shader
    }

    pub fn create_debug_texture(&self, tile_count_x: u32, tile_count_y: u32) -> GLuint {
        let mut debug_tex: GLuint = 0;
        unsafe {
            gl_check!(gl::GenTextures(1, &mut debug_tex));
            gl::BindTexture(gl::TEXTURE_2D, debug_tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as GLint,
                tile_count_x as GLsizei,
                tile_count_y as GLsizei,
                0,
                gl::RGBA,
                gl::FLOAT,
                std::ptr::null(),
            );
            gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
            gl_check!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
            gl_check!(gl::BindImageTexture(3, debug_tex, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F));
            let err = gl::GetError();
            assert_eq!(err, gl::NO_ERROR, "BindImageTexture failed: 0x{:x}", err);
        }
        
        debug_tex
    }

    pub fn debug_read_buffers(&self) {
        if let Some(culling_buffers) = &self.culling_buffers {
            // Read back light grid buffer
            unsafe {
                let buffer_size = (culling_buffers.tile_count_x * culling_buffers.tile_count_y * 2) as isize;
                let mut data = vec![0i32; buffer_size as usize];
                
                //gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, culling_buffers.light_grid_buffer.get_id());
                gl::GetBufferSubData(
                    gl::SHADER_STORAGE_BUFFER, 
                    0,
                    (buffer_size * std::mem::size_of::<i32>() as isize) as isize,
                    data.as_mut_ptr() as *mut std::os::raw::c_void
                );
                
                // Print some sample data
                println!("Light grid buffer sample:");
                for i in 0..min(10, data.len()/2) {
                    println!("Tile {}: offset={}, count={}", i, data[i*2], data[i*2 + 1]);
                }
            }
        }
    }

}

//rare public functions... do I just add these as like their own things in shadermanager... YES BRO OMG IM SO DUMB
fn initialize_depth_shader() -> ShaderProgram {//i could make this dynamic but like bruh
    //print!("erm couldnt do depth");
    let mut depth = ShaderProgram::new("shaders/depth_prepass.vert","shaders/depth_prepass.frag");
    depth.create_uniform("model");
    depth.create_uniform("view");
    depth.create_uniform("projection");
    depth
}

fn initialize_depth_debug_shader() -> ShaderProgram {//i could make this dynamic but like bruh
    //print!("erm couldnt do depth");
    let mut depth = ShaderProgram::new("shaders/depth_debug.vert","shaders/depth_debug.frag");
    depth.create_uniform("model");
    depth.create_uniform("view");
    depth.create_uniform("projection");
    depth.create_uniform("near");
    depth.create_uniform("far");
    depth
}

fn initialize_light_shader() -> ShaderProgram {//i could make this dynamic but like bruh
    let mut light = ShaderProgram::new("shaders/forward_plus.vert","shaders/forward_plus.frag");
    light.create_uniform("model");
    light.create_uniform("view");
    light.create_uniform("projection");
    light.create_uniform("u_specularPower");
    light.create_uniform("u_tileCountX");
    //light.create_uniforms(vec!["u_tileCountY", "u_screenWidth", "u_screenHeight"]);
    //light.create_uniform("u_depthTex");
    light.create_uniform("u_lightCount");
    light.create_uniform("u_diffuseColor");//why was this ok that it was like not there
    light
}

fn initialize_light_shader_plus() -> ShaderProgram {//i could make this dynamic but like bruh
    println!(":1");
    let mut light = ShaderProgram::new("shaderfp/plus.vert","shaderfp/plus.frag");
    light.create_uniform("model");
    light.create_uniform("view");
    light.create_uniform("projection");
    light.create_uniform("u_specularPower");
    light.create_uniform("u_tileCountX");
    //light.create_uniforms(vec!["u_tileCountY", "u_screenWidth", "u_screenHeight"]);
    //light.create_uniform("u_depthTex");
    light.create_uniform("u_lightCount");
    light.create_uniform("u_diffuseColor");//why was this ok that it was like not there
    print!(":2");
    light
}


fn initialize_light_test_shader() -> ShaderProgram {//i could make this dynamic but like bruh
    let mut light = ShaderProgram::new("shaders/forward_plus.vert","shaders/fp_new.frag");
    light.create_uniform("model");
    light.create_uniform("view");
    light.create_uniform("projection");
    light.create_uniform("u_specularPower");
    light.create_uniform("u_tileCountX");
    light.create_uniform("u_useNormalSmoothing");
    light.create_uniform("u_smoothingFactor");
    //light.create_uniforms(vec!["u_tileCountY", "u_screenWidth", "u_screenHeight"]);
    //light.create_uniform("u_depthTex");
    light.create_uniform("u_lightCount");
    light.create_uniform("u_diffuseColor");//why was this ok that it was like not there
    light
}

fn initialize_light_shader_debug() -> ShaderProgram {//i could make this dynamic but like bruh
    let mut light = ShaderProgram::new("shaders/debug_forward.vert","shaders/debug_forward.frag");
    light.create_uniform("model");
    light.create_uniform("view");
    light.create_uniform("projection");
    light.create_uniform("numberOfTilesX");
    light.create_uniform("totalLightCount");
    light
}


//i should just... make this... a function so it can just store all this stuff as references or just store it all for me that will prolly be like
//TODO first hing is do above prolly maybe just put it in light_manager or something man idk

//ok i did above but now I need to add this to a sepereate file i am like pretty sure
//2 many imports that were not previously needed by gl_wrapper
pub struct ForwardPlusRenderer {
    depth_shader: Arc<Mutex<ShaderProgram>>,//I might just make these not in shader_manager tbh
    light_shader: Arc<Mutex<ShaderProgram>>,
    light_manager: LightManager,
    framebuffer: Framebuffer,
    pub weighted_oit: WeightedOIT,
}

impl ForwardPlusRenderer {
    pub fn new(shader_manager: &ShaderManager) -> Self {
        let depth_shader = shader_manager.get_shader("depth")
            .expect("Depth shader not found");

        let light_shader = shader_manager.get_shader("light")
            .expect("Light shader not found");

        let light_manager = LightManager::new();

        let framebuffer = Framebuffer::new_depth_only(720, 720);//TODO add a method to update this later
        let weighted_oit = WeightedOIT::new(720, 720);
        
        Self {
            depth_shader,
            light_shader,
            light_manager,
            framebuffer,
            weighted_oit,
        }
    }

    pub fn new_debug(shader_manager: &ShaderManager) -> Self {
        let depth_shader = shader_manager.get_shader("depth")
            .expect("Depth shader not found");

        let light_shader = shader_manager.get_shader("light")
            .expect("Light shader not found");

        let light_manager = LightManager::new();

        let framebuffer = Framebuffer::new_depth_only(720, 720);
        let weighted_oit = WeightedOIT::new(720, 720);
        
        Self {
            depth_shader,
            light_shader,
            light_manager,
            framebuffer,
            weighted_oit
        }
    }
    
    pub fn render<'a>(&mut self, 
        models: impl IntoIterator<Item = &'a Box<dyn ModelTrait>>,
        camera: &Camera,
        width: u32, 
        height: u32,
        texture_manager: &TextureManager
    ) {
        //let models_iter = models.into_iter().collect::<Vec<_>>();//TODO feel like this adds overhead
        let all_models_iter = models.into_iter().collect::<Vec<_>>();

        // unsafe{
        //     gl::Enable(gl::DEPTH_TEST);
        //     gl::Disable(gl::BLEND);
        // }

        let (models_iter, transparent_models): (Vec<&&Box<dyn ModelTrait>>, Vec<&&Box<dyn ModelTrait>>) = all_models_iter
            .iter()
            .partition(|model| {
                // Check if material has alpha/transparency enabled
                !model.get_material().read().unwrap().is_transparent()
                // Or if you have a direct alpha bool:
                // !model.get_material().read().unwrap().alpha
            });

            //lol maybe switch to this for readability
            // let mut opaque_models: Vec<&Box<dyn ModelTrait>> = Vec::new();
            // let mut transparent_models: Vec<&Box<dyn ModelTrait>> = Vec::new();

            // for model in &models_iter {
            //     if model.get_material().read().unwrap().is_transparent() {
            //         // Or check your alpha bool
            //         transparent_models.push(model);
            //     } else {
            //         opaque_models.push(model);
            //     }
            // }

        // let framebuffer = Framebuffer::new_depth_only(width, height);
        //let meshes: Vec<&Mesh> = models_iter.iter().map(|model| model.get_mesh()).collect();
        
        let depth_shader_guard = self.depth_shader.lock().expect("failed to bind depth shader");
        depth_shader_guard.bind();
        //self.depth_shader.lock().expect("failed to bind depth").bind();//hmm its werid enabling this it just stops it from running the expect doesnt hit
        depth_shader_guard.set_matrix4fv_uniform("view", camera.get_view());
        depth_shader_guard.set_matrix4fv_uniform("projection", camera.get_p_matrix());
        //println!("here");

        self.framebuffer.bind();
        
        //for model in &models_iter {
            //TODO see how this is running the whole prepass twice... literally not what we want
            //its creating a super intresting but i know what i did wrong but i dont know why the output that i am getting
            //is being caused
            //mainly because its in consistennt.
            //i think it figured it out
            //its likely because right it is based off what comes first in the iterorator the moving object or th static one
            //i need to move model into the run depth prepass so it will not use the some model matrix for all the models and will
            //instead use them on a per model basis
            //println!("model Matrix {:#?}", &model.get_world_coords().get_model_matrix());
            //depth_shader_guard.set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());

            run_depth_prepass(
                &depth_shader_guard,
                &models_iter,
                self.framebuffer.get_depth_texture(),
                //&meshes,
                &mut self.light_manager,
                width,
                height,
            );
        //}

        ShaderProgram::unbind();
        Framebuffer::unbind();

        if let Some(culling_buffers) = &self.light_manager.culling_buffers {
            unsafe {
                gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, culling_buffers.light_buffer.get_id());
                gl_check!(gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, culling_buffers.light_index_buffer.get_id()));
            }
        }
        
        // Light culling
        self.light_manager.perform_gpu_light_culling(camera.get_view(),camera.get_p_matrix());//vp should prolly be just done on gpu later
        
        // Light pass
        // Framebuffer::unbind();//TODO CHECK WHAT THIS DOES THESE TWO LINES OR I GUESS 4
        // unsafe{
        //     gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        // }
        // unsafe {
        //     gl::Viewport(0, 0, width as i32, height as i32);
        //     gl::ClearDepth(0.0);
        //     gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        //     gl::DepthFunc(gl::GREATER);//this is erm maybe finiky
        //     gl::Enable(gl::DEPTH_TEST);
        // }
        
        self.light_shader.lock().expect("failed bind").bind();//if this was mutable would this work
        //if im getting errrors later look into this

        // Bind depth texture
        // let depth_tex = self.light_manager.get_depth_texture();
        // unsafe {
        //     gl_check!(gl::ActiveTexture(gl::TEXTURE0));
        //     gl::BindTexture(gl::TEXTURE_2D, depth_tex.id);
        // }

        //self.light_shader.lock().expect("failed bind").bind();

        //TODO make this more dynamic and ofc this will take time as i add more things and such. For example though what if I wanted to handle like ice with a different shader then glass because they would need different shaders what would I do there. Well I always have the matiral
        //shader that is attached to the model. That could be the basis for a first run or something idk.

        //idk gonna do like vertex stuff here
        self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("view", &camera.get_view());
        self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("projection", &camera.get_p_matrix());

        //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("totalLightCount", &(self.light_manager.lights.len() as i32));
        //lol move this create uniform somewhere else
        //self.light_shader.lock().expect("temp_light_shader failed to set uniform").create_uniform("u_depthTex");
        //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_depthTex", &0);
        
        //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_depthTex", &0);

        if let Some(culling_buffers) = &self.light_manager.culling_buffers {//If i get rid of if this doesnt work and I forget why... I think its something like if if we know its there so no need to account for like refstuff
            let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();//TODO move this and num of tiles to where shader is set up
            self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_tileCountX", &(tile_count_x as i32));
        }
        

        //self.light_shader.lock().expect("temp_light_shader failed to set uniform").create_uniform("u_lightCount");
        self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_lightCount", &(self.light_manager.lights.len() as i32));


        //the two below I will add to like materials or something... PROBABLY PROBABLY IN FACT TODO, impliment a Forward_Plus Material Trait for materials that can be used here in this forward plus...
        //maybe they just like dont have a shader and like have these material properties instead... maybe I make it completely different from the materials we have hmmm yeah
        //so what I should do is have a materials trait that has like 2 sub catagories wich is like shader material and forward plus material and they both have like material properties
        //one just had a shader and some other stuff. This would probably greatly simplify it and prevent it from needing to use arc and stuff making it like smaller or whatever
        self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform4f("u_diffuseColor", &Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 });
        self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1f("u_specularPower", 0.7);
        
        //self.light_shader.lock().expect("temp_light_shader failed to set uniform").debug_print_uniforms();
        // Render each model with its material
        for model in &models_iter {
            println!("model Matrix {:#?}", &model.get_world_coords().get_model_matrix());
            self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());//lol model matrix , low key needa be finna more accessible
            //model.get_material().write().unwrap().apply(texture_manager, &model.get_world_coords().get_model_matrix());
            model.get_mesh().draw();
        }


        
        ShaderProgram::unbind();

        if !transparent_models.is_empty() {
            // Resize OIT buffers if needed
            //self.weighted_oit.resize(width, height);

            println!("Rendering {} transparent models", transparent_models.len());
            for model in &transparent_models {
                println!("  - Transparent model at: {:?}", model.get_world_coords().get_model_matrix());
            }

            self.weighted_oit.attach_depth_texture(self.framebuffer.get_depth_texture().id);
            
            // Render transparent objects using weighted OIT
            self.weighted_oit.render_transparency(
                transparent_models.into_iter().map(|&model| model),
                camera,
                texture_manager,
                &self.light_manager
            );
        }

        unsafe{//just a precaution
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, 0);
		    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, 0);
        }
    }

    // pub fn render_test<T: ModelTrait>(&mut self, 
    //     models: &[T], 
    //     camera: &Camera,
    //     width: u32, 
    //     height: u32,
    //     texture_manager: &TextureManager
    // ) {
    //     let framebuffer = Framebuffer::new_depth_only(width, height);
    //     let meshes: Vec<&Mesh> = models.iter().map(|model| model.get_mesh()).collect();
        
    //     self.depth_shader.lock().expect("failed to bind depth").bind();
        
    //     for model in models {
    //         self.depth_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());
    //         self.depth_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("view", camera.get_view());
    //         self.depth_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("projection", camera.get_p_matrix());
    //         // Depth pre-pass
    //         run_depth_prepass(
    //             &self.depth_shader.lock().expect("failed to get depth Shader during prepass"),
    //             &framebuffer,
    //             &meshes,
    //             &mut self.light_manager,
    //             width,
    //             height,
    //         );
    //     }

    //     if let Some(culling_buffers) = &self.light_manager.culling_buffers {
    //         unsafe {
    //             gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, culling_buffers.light_buffer.get_id());
    //             gl_check!(gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, culling_buffers.light_index_buffer.get_id()));
    //         }
    //     }
        
    //     // Light culling
    //     self.light_manager.perform_gpu_light_culling(camera.get_view(),camera.get_p_matrix());//vp should prolly be just done on gpu later
        
       
        
    //     self.light_shader.lock().expect("failed bind").bind();//if this was mutable would this work
        
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("view", &camera.get_view());
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("projection", &camera.get_p_matrix());

    //     //print!("heeeeeeeheeeee");
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1f("u_useNormalSmoothing", 1.0);
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1f("u_smoothingFactor",1.0);

    //     if let Some(culling_buffers) = &self.light_manager.culling_buffers {//If i get rid of if this doesnt work and I forget why... I think its something like if if we know its there so no need to account for like refstuff
    //         let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();//TODO move this and num of tiles to where shader is set up
    //         self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_tileCountX", &(tile_count_x as i32));
    //     }
        
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_lightCount", &(self.light_manager.lights.len() as i32));


    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform4f("u_diffuseColor", &Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 });
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1f("u_specularPower", 1.0);
        
    
    //     for model in models {
    //         self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());//lol model matrix , low key needa be finna more accessible
    //         model.get_mesh().draw();
    //     }

    //     unsafe{//just a precaution
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, 0);
	// 	    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, 0);
    //     }
        
    //     ShaderProgram::unbind();
    // }

    // pub fn render_debug<T: ModelTrait>(&mut self, 
    //     models: &[T], 
    //     camera: &Camera,
    //     width: u32, 
    //     height: u32,
    //     texture_manager: &TextureManager,
    //     debug_comp_shader: &mut ShaderProgram,
    // ) {
    //     let framebuffer = Framebuffer::new_depth_only(width, height);
    //     let meshes: Vec<&Mesh> = models.iter().map(|model| model.get_mesh()).collect();
        
    //     self.depth_shader.lock().expect("failed to bind depth").bind();

    //     // let debugshader = initialize_depth_debug_shader();//lol recompiling this is not optimal btw
    //     // debugshader.bind();

    //     // let model_matrices: Vec<Matrix4<f32>> = models.iter()
    //     //     .map(|model| model.get_world_coords().get_model_matrix())
    //     //     .collect();

    //     // run_depth_debug_pass(
    //     //     &debugshader,
    //     //     &meshes,
    //     //     camera,
    //     //     &model_matrices,
    //     // );//this works really well now
        
    //     for model in models {
    //         self.depth_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());
    //         self.depth_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("view", camera.get_view());
    //         self.depth_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("projection", camera.get_p_matrix());
    //         // Depth pre-pass
    //         run_depth_prepass(
    //             &self.depth_shader.lock().expect("failed to get depth Shader during prepass"),
    //             &framebuffer,
    //             &meshes,
    //             &mut self.light_manager,
    //             width,
    //             height,
    //         );
    //     }

    //     if let Some(culling_buffers) = &self.light_manager.culling_buffers {
    //         unsafe {
    //             gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, culling_buffers.light_buffer.get_id());
    //             //gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, culling_buffers.light_grid_buffer.get_id());
    //             gl_check!(gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, culling_buffers.light_index_buffer.get_id()));
    //         }
    //     }
        
        
    //     // Light culling
    //     //self.light_manager.debug_perform_gpu_light_culling(camera.get_view(),camera.get_p_matrix(), width, height);//vp should prolly be just done on gpu later
    //     let debug_texture = self.light_manager.debug_perform_gpu_light_culling(
    //         camera.get_view(),
    //         camera.get_p_matrix(),
    //     );
    
        
    //     // Light pass
    //     //Framebuffer::unbind();
        
    //     // unsafe {
    //     //     gl::Viewport(0, 0, width as i32, height as i32);
    //     //     gl::ClearDepth(0.0);
    //     //     gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    //     //     gl::DepthFunc(gl::GREATER);//this is erm maybe finiky
    //     //     gl::Enable(gl::DEPTH_TEST);
    //     // }
        
    //     self.light_shader.lock().expect("failed bind").bind();//if this was mutable would this work
    //     //if im getting errrors later look into this

    //     // Bind depth texture
    //     //let depth_tex = self.light_manager.get_depth_texture();
    //     // unsafe {
    //     //     gl_check!(gl::ActiveTexture(gl::TEXTURE0));
    //     //     gl::BindTexture(gl::TEXTURE_2D, depth_tex.id);
    //     // }

    //     //self.light_shader.lock().expect("failed bind").bind();

    //     //idk gonna do like vertex stuff here
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("view", &camera.get_view());
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("projection", &camera.get_p_matrix());

    //     //lol move this create uniform somewhere else
    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").create_uniform("u_depthTex");
    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_depthTex", &0);
        
    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("u_depthTex", &0);
    //     // Bind light culling buffers

    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").create_uniform("u_lightCount");
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("totalLightCount", &(self.light_manager.lights.len() as i32));
    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform4f("u_diffuseColor", &Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 });
        
    //     if let Some(culling_buffers) = &self.light_manager.culling_buffers {//If i get rid of if this doesnt work and I forget why... I think its something like if if we know its there so no need to account for like refstuff
    //         let (tile_count_x, tile_count_y) = culling_buffers.get_tile_counts();//TODO move this and num of tiles to where shader is set up
    //         self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1i("numberOfTilesX", &(tile_count_x as i32));
    //     }
        
    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1f("u_tileCountY", tile_count_y as f32);
    //     //this all needs to move later
    //     //self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_uniform1f("u_specularPower", 1.0);
        
    //     self.light_shader.lock().expect("temp_light_shader failed to set uniform").debug_print_uniforms();
    //     // Render each model with its material
    //     for model in models {
    //         self.light_shader.lock().expect("temp_light_shader failed to set uniform").set_matrix4fv_uniform("model", &model.get_world_coords().get_model_matrix());//lol model matrix , low key needa be finna more accessible
    //         //model.get_material().write().unwrap().apply(texture_manager, &model.get_world_coords().get_model_matrix());
    //         model.get_mesh().draw();
    //     }

    //     if let Some(debug_tex) = debug_texture {
    //         // Now render the debug texture (can be in corner of screen)
    //         self.light_manager.render_debug_visualization(debug_tex, width, height, debug_comp_shader);
            
    //         // Clean up the debug texture when done
    //         unsafe {
    //             gl_check!(gl::DeleteTextures(1, &debug_tex));
    //         }
    //     }

    //     unsafe{
    //         gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, 0);
	// 	    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, 0);
    //     }
        
    //     ShaderProgram::unbind();
    // }
    

    //a bunch of function mess with lights and stuff
    pub fn add_light(&mut self, position: [f32; 3], radius: f32, color: [f32; 3], intensity: f32) {
        self.light_manager.lights.push(Light { 
            position, 
            radius, 
            color, 
            intensity 
        });
    }

    pub fn clear_lights(&mut self) {
        self.light_manager.lights.clear();
    }

    pub fn get_light_count(&self) -> usize {
        self.light_manager.lights.len()
    }

    pub fn get_lights(&self) -> &Vec<Light> {
        &self.light_manager.lights
    }

    pub fn update_light_position(&mut self, index: usize, new_position: [f32; 3]) {
        if let Some(light) = self.light_manager.lights.get_mut(index) {
            light.position = new_position;
        }
    }
    
    pub fn initialize_light_culling(&mut self, width: u32, height: u32, shader_manager: &ShaderManager) {
        self.light_manager.initialize_gpu_culling(width, height, shader_manager);
    }

    pub fn debug_light_info(&self) {
        for (i, light) in self.light_manager.lights.iter().enumerate() {
            println!("Light {}: pos={:?}, radius={}, color={:?}, intensity={}", 
                     i, light.position, light.radius, light.color, light.intensity);
        }
    }
}

pub struct LightCullingBuffers {
    light_buffer: BufferObject,          // SSBO for light data
    light_index_buffer: BufferObject,    // SSBO for light indices per tile
    //light_grid_buffer: BufferObject,     // SSBO for light grid data
    tile_count_x: u32,
    tile_count_y: u32,
    max_lights_per_tile: u32,
}

#[allow(unused_parens)]
impl LightCullingBuffers {
    pub fn new(width: u32, height: u32, max_lights: u32) -> Self {
        let tile_size = 16; // Tile size, same as in your CPU implementation
        let tile_count_x = (width + (tile_size % tile_size) / tile_size);//TODO cahnged these look at this as well
        let tile_count_y = (height + (tile_size % tile_size)) / tile_size;
        let max_lights_per_tile = 256; // HERHEHERHEHERE look at this is this it YEHA THIS WAS IT
        
        // Create SSBO for light data
        let light_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        
        // Create SSBO for light indices
        let light_index_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        
        // Create SSBO for light grid
        //let light_grid_buffer = BufferObject::new(gl::SHADER_STORAGE_BUFFER, gl::DYNAMIC_DRAW);
        
        Self {
            light_buffer,
            light_index_buffer,
            //light_grid_buffer,
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

            light_data.push(light.color[0]);
            light_data.push(light.color[1]);
            light_data.push(light.color[2]);
            light_data.push(light.intensity);
        }
        
        // Bind and upload light data
        self.light_buffer.bind();

        //println!("Buffer ID before store: {}", self.light_buffer.id);

        //print!("bf liught");
        //print!("light data: {:#?}", light_data);
        //self.light_buffer.bind();
        self.light_buffer.store_f32_data(&light_data);
        //print!("af liught");
        // Prepare light grid buffer (will be filled by compute shader)TODO LOOK HEREH HERHEHREHR EHRHE
        //TODO
        let total_tiles = (self.tile_count_x * self.tile_count_y) as usize;
        //let grid_size = total_tiles * 2; // Each tile has (offset, count)
        //let mut grid_data = vec![0i32; grid_size];
        
        // self.light_grid_buffer.bind();//TODO ifeel bad for killing him
        // self.light_grid_buffer.store_i32_data(&grid_data);
        
        // Prepare light index buffer (will be filled by compute shader)
        let index_buffer_size = total_tiles * self.max_lights_per_tile as usize;
        let mut index_data = vec![0i32; index_buffer_size];

        self.light_index_buffer.bind();
        self.light_index_buffer.store_i32_data(&index_data);
        
        // Bind the buffers to their respective binding points
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.light_buffer.get_id());
            //gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 1, self.light_grid_buffer.get_id());
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 2, self.light_index_buffer.get_id());
        }
    }
    
    pub fn get_tile_counts(&self) -> (u32, u32) {
        (self.tile_count_x, self.tile_count_y)
    }
}

