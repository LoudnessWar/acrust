use std::collections::HashMap;

use cgmath::Vector2;

use crate::graphics::gl_wrapper::Vao;
use crate::graphics::gl_wrapper::VertexAttribute;
use crate::graphics::gl_wrapper::BufferObject;
use crate::graphics::gl_wrapper::ShaderProgram;

use gl::types::GLuint;

//ok so Im not really sure what the best way to go about this would be... is like textures like faster then splines? da computers be making
//da fonts with them fr right?




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

