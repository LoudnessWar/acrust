use std::collections::HashMap;

use cgmath::Vector2;

use crate::graphics::gl_wrapper::Vao;
use crate::graphics::gl_wrapper::VertexAttribute;
use crate::graphics::gl_wrapper::BufferObject;
use crate::graphics::gl_wrapper::ShaderProgram;

use gl::types::GLuint;

//ok so Im not really sure what the best way to go about this would be... is like textures like faster then splines? da computers be making
//da fonts with them fr right?

//I think ineviably this is someting I am just going to use an external library for bc like bffr like that its just 2 much bs
pub struct Character {
    pub texture_id: GLuint,// I am hesistant to use the texture manager here only beacuse Im afraod it will be rather iffy to say da least
    pub size: Vector2<i32>,
    pub bearing: Vector2<i32>,
    pub advance: u32,
}

pub struct TextRenderer {
    characters: HashMap<char, Character>,
    vao: Vao,
    vbo: BufferObject,
    shader: ShaderProgram,
}

impl TextRenderer {
    //le setting up vbos and such
    pub fn new(text_shader: ShaderProgram) -> Self {
        let vao = Vao::new();
        vao.bind();
        
        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW);
        vbo.bind();
        
        let stride = 4 * mem::size_of::<f32>() as gl::types::GLsizei;
        VertexAttribute::new(0, 4, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();
        
        vao.unbind();
        
        Self {
            characters: HashMap::new(),
            vao,
            vbo,
            shader: text_shader,
        }
    }


}