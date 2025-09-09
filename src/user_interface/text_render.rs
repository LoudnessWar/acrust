use std::collections::HashMap;
use std::mem;
use std::ptr;

use cgmath::Matrix4;
use cgmath::Vector2;
use cgmath::Vector3;

use crate::graphics::gl_wrapper::Vao;
use crate::graphics::gl_wrapper::VertexAttribute;
use crate::graphics::gl_wrapper::BufferObject;
use crate::graphics::gl_wrapper::ShaderProgram;

use gl::types::GLuint;

use rusttype::{Font, Scale, point};

pub struct Character {
    pub texture_id: GLuint,
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

    pub fn load_font(&mut self, font_path: &str, font_size: f32) {
        // Load font data
        let font_data = std::fs::read(font_path).expect("Failed to read font file");
        let font = Font::try_from_vec(font_data).expect("Failed to load font");

        let scale = Scale::uniform(font_size);

        for c in 0u8..128u8 {
            let ch = c as char;
            let glyph = font.glyph(ch).scaled(scale).positioned(point(0.0, 0.0));
            let h_metrics = glyph.unpositioned().h_metrics();

            // Rasterize glyph to a bitmap
            if let Some(bb) = glyph.pixel_bounding_box() {
                let width = bb.width();
                let height = bb.height();

                let mut pixel_data = vec![0u8; (width * height) as usize];
                glyph.draw(|x, y, v| {
                    let idx = (y as usize * width as usize) + x as usize;
                    pixel_data[idx] = (v * 255.0) as u8;
                });

                // Upload to OpenGL as a single-channel texture
                let mut texture_id: GLuint = 0;
                unsafe {
                    gl::GenTextures(1, &mut texture_id);
                    gl::BindTexture(gl::TEXTURE_2D, texture_id);
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RED as i32,
                        width,
                        height,
                        0,
                        gl::RED,
                        gl::UNSIGNED_BYTE,
                        pixel_data.as_ptr() as *const _,
                    );
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }

                self.characters.insert(
                    ch,
                    Character {
                        texture_id,
                        size: Vector2::new(width, height),
                        bearing: Vector2::new(bb.min.x, bb.min.y),
                        advance: h_metrics.advance_width as u32,
                    },
                );
            } else {
                // Insert a dummy character for non-renderable glyphs
                self.characters.insert(
                    ch,
                    Character {
                        texture_id: 0,
                        size: Vector2::new(0, 0),
                        bearing: Vector2::new(0, 0),
                        advance: h_metrics.advance_width as u32,
                    },
                );
            }
        }
    }

    pub fn render_text(
        &self,
        text: &str,
        mut x: f32,
        y: f32,
        scale: f32,
        color: Vector3<f32>,
        get_projection: &Matrix4<f32>,
    ) {
        self.shader.bind();
        self.vao.bind();

        // Set text color uniform (assuming your shader uses "textColor")
        self.shader.set_uniform3f("textColor", &color);
        self.shader.set_matrix4fv_uniform("projection", get_projection);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
        }

        for c in text.chars() {
            if let Some(ch) = self.characters.get(&c) {
                if ch.texture_id == 0 {
                    x += ch.advance as f32 * scale;
                    continue;
                }
                let xpos = x + ch.bearing.x as f32 * scale;
                let ypos = y - ch.bearing.y as f32 * scale;

                let w = ch.size.x as f32 * scale;
                let h = ch.size.y as f32 * scale;

                // Vertices: x, y, tex_x, tex_y
                let vertices: [f32; 16] = [
                    xpos,     ypos + h, 0.0, 1.0,
                    xpos + w, ypos + h, 1.0, 1.0,
                    xpos + w, ypos,     1.0, 0.0,
                    xpos,     ypos,     0.0, 0.0,
                ];

                // Upload vertex data
                self.vbo.bind();
                self.vbo.store_f32_data(&vertices);

                unsafe {
                    gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
                    gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
                }

                // Advance cursor for next glyph
                x += ch.advance as f32 * scale;
            }
        }

        self.vao.unbind();
    }
}