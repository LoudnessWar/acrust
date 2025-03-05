use std::fmt;
use std::collections::HashMap;
use std::mem;
use std::os::raw::*;
use std::ffi::CString;
use std::fs::File;
use std::ptr;
use std::io::Read;
// use std::sync::PoisonError;
use cgmath::*;
use gl::types::*;
use std::sync::Mutex;
use std::sync::Arc;

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

    pub fn load_shader(&self, name: &str, vert_path: &str, frag_path: &str) -> Arc<Mutex<ShaderProgram>> {
        let mut shaders = self.shaders.lock().unwrap();
        shaders.entry(name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(ShaderProgram::new(vert_path, frag_path))))
            .clone()
    }

    //bro whats the point even in using rust if imma do this like fr smh do better
    pub fn get_shader(&self, name: &str) -> Option<Arc<Mutex<ShaderProgram>>> {
        let shaders = self.shaders.lock().unwrap();
        shaders.get(name).cloned()
    }

    pub fn enable_backface_culling(&mut self, name: &str){
        self.get_shader(name).expect("CANNOT FIND SHADER").lock().unwrap().enable_backface_culling();
    }

    pub fn enable_depth(&mut self, name: &str){
        self.get_shader(name).expect("CANNOT FIND SHADER").lock().unwrap().enable_depth();
    }
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
