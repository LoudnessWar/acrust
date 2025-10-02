
use std::ffi::c_void;
// use std::mem;
use std::fmt;
use std::sync::{Arc, Mutex};//when the arc and mutex come out

use super::gl_wrapper::UniformValue;
use super::gl_wrapper::{BufferObject, ShaderManager, ShaderProgram, VertexAttribute, Vao};

//this is either genius or really fucking stupid

//also like damn i need to make this slightly bigger just to store vertexArray are we serious
// pub trait GraphicsBackend {
//     type VertexArray;
//     type Buffer: GpuBuffer;
//     type Shader;

//     fn create_vertex_array(&mut self) -> Self::VertexArray;
//     fn create_buffer(&mut self, target: u32, usage: u32) -> Self::Buffer;
//     fn create_shader(&mut self, vert_src: &str, frag_src: &str) -> Self::Shader;

//     fn set_uniform(&mut self, shader: &Self::Shader, name: &str, value: &UniformValue);
//     fn draw(&mut self, vao: &Self::VertexArray, shader: &Self::Shader, count: i32);
// }




//open gl_wrapper wrappers first, this is just going to be like trait functions that call stuff

//First off im going to start at the top of the Open GL file..... so like buffer or fucking whaterver bitch
//uuuh liker erm ok I had to create a gpubuffer object

//this is because VAO VBO vertexattributes like this is very good for open gl... uuuuhhhh kinda buns lowkey for vulkan so uuuh we are going to put this wrapper around it
//BTW the reason I am doing it all like this is simple ok.

//LIKE here is the philosophical just ification behind it. I want it to be possible so switch from open gl to vulkan by just like swapping out some shaders and like 
//jjerking off a monkey while still maintiaing all the complexity of something deeper like
//if you want after this nothing will stop you from calling to gl_wrapper and creating vbo vao ect in fact I want some thinks like material manager and some other things to maintian this sepertaion
//behind the scenes of course, like i dont want anyone calling to this unless they are just like
//hey im messing around with shaders and i want it to be really simple and just fun

//also you might notice that this is kinda reminiscent of like an ecs like kinda not really.....idk im stupfi 

pub trait GpuBuffer {
    fn bind(&self);
    fn unbind(&self);
    fn upload_f32(&self, data: &[f32]);
    fn upload_i32(&self, data: &[i32]);
    fn id(&self) -> u32;
}

/// Trait representing a vertex-array-like wrapper (VAO for GL; may be no-op for Vulkan)
pub trait VertexArray {
    fn new() -> Self where Self: Sized;
    fn bind(&self);
    fn unbind(&self);
}

/// Minimal shader manager API your existing shader manager must expose (adapter pattern).
/// Implement this for your existing shader manager to let OpenGLBackend use it directly.
pub trait ShaderManagerApi {
    type ShaderHandle;

    /// Create / compile / link shader program from GLSL source. Return an opaque handle.
    fn load_shader(&mut self, name: &str, vertex_src: &str, fragment_src: &str) -> Self::ShaderHandle;

    /// Bind the shader for use.
    fn bind(&mut self, shader: &Self::ShaderHandle);

    /// Unbind current shader (if needed).
    fn unbind(&mut self);

    /// Set a named uniform using your UniformValue type.
    fn set_uniform(&mut self, shader: &Self::ShaderHandle, name: &str, value: &UniformValue);

    /// Optional: fetch backend shader id if needed (e.g., GLuint)
    fn shader_id(&self, shader: &Self::ShaderHandle) -> Option<u32> { let _ = shader; None }
}

impl ShaderManagerApi for ShaderManager {
    type ShaderHandle = Arc<Mutex<ShaderProgram>>;

    fn load_shader(&mut self, name: &str, vertex_src: &str, fragment_src: &str) -> Self::ShaderHandle {
        ShaderManager::load_shader(self, name, vertex_src, fragment_src)//get it twisted compiler there are two functions in the same trait that are both called load_shader... they just take in different types so they are actually differnet though
        //but you can call them like kinda the same lowkey
        //also like tbh names for shaders, who needs them?
        //is what a fool would say but nah people get names just easier to work with
    }

    fn bind(&mut self, shader: &Self::ShaderHandle) {
        shader.lock().unwrap().bind();
    }

    fn unbind(&mut self) {
        ShaderProgram::unbind();
    }

    fn set_uniform(&mut self, shader: &Self::ShaderHandle, name: &str, value: &UniformValue) {
        let program = shader.lock().unwrap();
        match value {
            UniformValue::Float(f) => program.set_uniform1f(name, *f),
            UniformValue::Int(i)   => program.set_uniform1i(name, i),
            UniformValue::Vector4(v) => program.set_uniform4f(name, v),
            UniformValue::Vector3(v) => program.set_uniform3f(name, v),
            UniformValue::Matrix4(m) => program.set_matrix4fv_uniform(name, m),
            UniformValue::Texture(id) => {
                // Right now, your ShaderProgram doesn’t directly bind textures.
                // Typically you’d call glActiveTexture + glBindTexture + program.set_uniform1i
                // For now we just push it as integer sampler index.
                let tid = *id as i32;
                program.set_uniform1i(name, &tid);
            }
            UniformValue::Empty() => {
                // no-op or panic
                eprintln!("Tried to set empty uniform {}!", name);
            }
        }
    }

    fn shader_id(&self, shader: &Self::ShaderHandle) -> Option<u32> {
        Some(*shader.lock().unwrap().get_program_handle())
    }
}

// ---------------- Implement GpuBuffer + VertexArray for your GL types ----------------

// Implement GpuBuffer for your existing BufferObject
impl GpuBuffer for BufferObject {
    fn bind(&self) { self.bind(); }
    fn unbind(&self) { self.unbind(); }
    fn upload_f32(&self, data: &[f32]) { self.store_f32_data(data); }
    fn upload_i32(&self, data: &[i32]) { self.store_i32_data(data); }
    fn id(&self) -> u32 { self.get_id() }
}

// Implement VertexArray for your Vao
impl VertexArray for Vao {
    fn new() -> Self { Vao::new() }
    fn bind(&self) { self.bind(); }
    fn unbind(&self) { self.unbind(); }
}

// ---------------- The main GraphicsBackend trait ----------------

/// Top-level backend trait. Engine code depends on this and not on GL/VK directly.
pub trait GraphicsBackend {
    type VA: VertexArray;
    type Buffer: GpuBuffer;
    type Shader;

    /// Create resources
    fn create_vertex_array(&mut self) -> Self::VA;
    fn create_buffer(&mut self, target: u32, usage: u32) -> Self::Buffer;
    fn create_vertex_attribute(&mut self,
        index: u32,
        size: i32,
        r#type: u32,
        normalized: u8,
        stride: i32,
        pointer: *const c_void
    ) -> VertexAttribute;

    fn create_shader_from_src(&mut self, name: &str, vert_src: &str, frag_src: &str) -> Self::Shader;

    /// High-level operations
    fn set_uniform(&mut self, shader: &Self::Shader, name: &str, value: &UniformValue);
    fn draw_arrays(&mut self, vao: &Self::VA, shader: &Self::Shader, count: i32);
    fn draw_elements(&mut self, vao: &Self::VA, shader: &Self::Shader, count: i32);
}

// ---------------- OpenGL backend that delegates to your existing GL wrapper ----------------

/// OpenGLBackend holds a shader manager (your implementation) and delegates buffer/vao work
pub struct OpenGLBackend<SM: ShaderManagerApi> {
    pub shader_manager: SM,
}

impl<SM: ShaderManagerApi> OpenGLBackend<SM> {
    pub fn new(shader_manager: SM) -> Self {
        Self { shader_manager }
    }
}

impl<SM> GraphicsBackend for OpenGLBackend<SM>
where
    SM: ShaderManagerApi,
    SM::ShaderHandle: fmt::Debug,
{
    type VA = Vao;
    type Buffer = BufferObject;
    type Shader = SM::ShaderHandle;

    fn create_vertex_array(&mut self) -> Self::VA {
        Vao::new()
    }

    fn create_buffer(&mut self, target: u32, usage: u32) -> Self::Buffer {
        BufferObject::new(target, usage)
    }

    fn create_vertex_attribute(&mut self,
        index: u32,
        size: i32,
        r#type: u32,
        normalized: u8,
        stride: i32,
        pointer: *const c_void
    ) -> VertexAttribute {
        // note: cast types to GL types; your VertexAttribute::new expects GL types
        let norm = if normalized == 0 { gl::FALSE } else { gl::TRUE };
        VertexAttribute::new(index, size, r#type, norm, stride, pointer)
    }

    fn create_shader_from_src(&mut self, name: &str, vert_src: &str, frag_src: &str) -> Self::Shader {//long ass convoluted ass name rumple stiltskin ass fuckin rip van winkle ass nominal determinist when this takes a long time to compile
        self.shader_manager.load_shader(name ,vert_src, frag_src)
    }

    fn set_uniform(&mut self, shader: &Self::Shader, name: &str, value: &UniformValue) {
        self.shader_manager.set_uniform(shader, name, value)
    }

    fn draw_arrays(&mut self, vao: &Self::VA, shader: &Self::Shader, count: i32) {
        // Bind shader -> bind VAO -> issue draw -> unbind (if desired)
        self.shader_manager.bind(shader);
        vao.bind();
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, count);
        }
        vao.unbind();
        self.shader_manager.unbind();
    }

    fn draw_elements(&mut self, vao: &Self::VA, shader: &Self::Shader, count: i32) {
        self.shader_manager.bind(shader);
        vao.bind();
        unsafe {
            gl::DrawElements(gl::TRIANGLES, count, gl::UNSIGNED_INT, std::ptr::null());
        }
        vao.unbind();
        self.shader_manager.unbind();
    }
}

// ---------------- Backend enum — easy runtime switching ----------------

pub enum Backend<SM: ShaderManagerApi> {
    OpenGL(OpenGLBackend<SM>),
    // Vulkan(VulkanBackend) -> implement later
}

impl<SM: ShaderManagerApi> Backend<SM> {
    pub fn into_opengl(mgr: SM) -> Self { Backend::OpenGL(OpenGLBackend::new(mgr)) }
}

impl<SM> GraphicsBackend for Backend<SM>
where
    SM: ShaderManagerApi,
    SM::ShaderHandle: fmt::Debug,
{
    type VA = Vao;
    type Buffer = BufferObject;
    type Shader = SM::ShaderHandle;

    fn create_vertex_array(&mut self) -> Self::VA {
        match self {
            Backend::OpenGL(gl) => gl.create_vertex_array(),
        }
    }

    fn create_buffer(&mut self, target: u32, usage: u32) -> Self::Buffer {
        match self {
            Backend::OpenGL(gl) => gl.create_buffer(target, usage),
        }
    }

    fn create_vertex_attribute(&mut self,
        index: u32,
        size: i32,
        r#type: u32,
        normalized: u8,
        stride: i32,
        pointer: *const c_void
    ) -> VertexAttribute {
        match self {
            Backend::OpenGL(gl) => gl.create_vertex_attribute(index, size, r#type, normalized, stride, pointer)
        }
    }

    fn create_shader_from_src(&mut self, name: &str, vert_src: &str, frag_src: &str) -> Self::Shader {
        match self {
            Backend::OpenGL(gl) => gl.create_shader_from_src(name, vert_src, frag_src),
        }
    }

    fn set_uniform(&mut self, shader: &Self::Shader, name: &str, value: &UniformValue) {
        match self {
            Backend::OpenGL(gl) => gl.set_uniform(shader, name, value),
        }
    }

    fn draw_arrays(&mut self, vao: &Self::VA, shader: &Self::Shader, count: i32) {
        match self {
            Backend::OpenGL(gl) => gl.draw_arrays(vao, shader, count),
        }
    }

    fn draw_elements(&mut self, vao: &Self::VA, shader: &Self::Shader, count: i32) {
        match self {
            Backend::OpenGL(gl) => gl.draw_elements(vao, shader, count),
        }
    }
}