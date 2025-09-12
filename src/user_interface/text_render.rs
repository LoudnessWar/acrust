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
    pub advance: f32,
}

pub struct TextRenderer {
    characters: HashMap<char, Character>,
    vao: Vao,
    vbo: BufferObject,
    ebo: BufferObject,
    shader: ShaderProgram,
    // Store font metrics for consistent baseline alignment
    baseline_offset: f32,
}

impl TextRenderer {
    pub fn new(text_shader: ShaderProgram) -> Self {
        let vao = Vao::new();
        vao.bind();

        let vbo = BufferObject::new(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW);
        vbo.bind();

        let ebo = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER, gl::DYNAMIC_DRAW);
        ebo.bind();

        // stride: 3 position floats + 2 uv floats
        let stride = (5 * mem::size_of::<f32>()) as gl::types::GLsizei;
        VertexAttribute::new(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null()).enable();
        VertexAttribute::new(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<f32>()) as *const _,
        )
        .enable();

        // Unbind to keep state clean
        vao.unbind();

        Self {
            characters: HashMap::new(),
            vao,
            vbo,
            ebo,
            shader: text_shader,
            baseline_offset: 0.0,
        }
    }

    pub fn render_text(
        &self,
        text: &str,
        mut x: f32,
        y: f32,
        scale: f32,
        color: Vector3<f32>,
        projection: &Matrix4<f32>,
    ) {
        self.shader.bind();
        self.vao.bind();

        // Set text color uniform
        self.shader.set_uniform3f("textColor", &color);

        // Some GL wrappers expect row-major, some column-major. To be resilient we
        // upload the transposed projection as well as the original from the caller
        // (common shader upload implementations take column-major by default). If
        // your shader expects a different layout, remove the .transpose() call.
        self.shader.set_matrix4fv_uniform("projection", &projection);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        for c in text.chars() {
            if let Some(ch) = self.characters.get(&c) {
                if ch.texture_id == 0 {
                    x += ch.advance * scale;
                    continue;
                }

                // Fixed positioning: align all characters to the same baseline
                let xpos = x + ch.bearing.x as f32 * scale;
                // Use the baseline_offset to ensure consistent alignment
                let ypos = y + (self.baseline_offset + ch.bearing.y as f32) * scale;

                let w = ch.size.x as f32 * scale;
                let h = ch.size.y as f32 * scale;

                println!(
                    "Rendering char '{}' at ({}, {}) with size ({}, {})",
                    c, xpos, ypos, w, h
                );

                // bottom-left origin for positions and matching UVs
                let vertices: [f32; 20] = [
                    xpos,     ypos + h, 0.0, 0.0, 1.0, // top-left
                    xpos + w, ypos + h, 0.0, 1.0, 1.0, // top-right
                    xpos + w, ypos,     0.0, 1.0, 0.0, // bottom-right
                    xpos,     ypos,     0.0, 0.0, 0.0, // bottom-left
                ];

                // indices as unsigned ints to match gl::UNSIGNED_INT
                let indices: [i32; 6] = [0, 1, 2, 0, 2, 3];

                // Upload vertex data per-glyph (overwrite VBO/ EBO)
                self.vbo.bind();
                self.vbo.store_f32_data(&vertices);

                self.ebo.bind();
                self.ebo.store_i32_data(&indices);

                unsafe {
                    // Bind the glyph texture
                    gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);

                    // Draw the quad
                    gl::DrawElements(
                        gl::TRIANGLES,
                        indices.len() as i32,
                        gl::UNSIGNED_INT,
                        ptr::null(),
                    );
                }

                // Advance cursor for next glyph
                x += ch.advance * scale;
            }
        }

        unsafe {
            gl::Disable(gl::BLEND);
        }

        self.vao.unbind();
    }

    pub fn load_font(&mut self, font_path: &str, font_size: f32) {
        // Load font data
        let font_data = std::fs::read(font_path).expect("Failed to read font file");
        let font = Font::try_from_vec(font_data).expect("Failed to load font");

        let scale = Scale::uniform(font_size);
        
        // Calculate baseline offset by finding the maximum bearing.y (how far above baseline)
        // This ensures all characters align to the same visual baseline
        let mut max_bearing_y = 0;
        let mut min_bearing_y = 0;
        
        // First pass: calculate font metrics
        for c in 0u8..128u8 {
            let ch = c as char;
            let glyph = font.glyph(ch).scaled(scale).positioned(point(0.0, 0.0));
            
            if let Some(bb) = glyph.pixel_bounding_box() {
                max_bearing_y = max_bearing_y.max(bb.max.y);
                min_bearing_y = min_bearing_y.min(bb.min.y);
            }
        }
        
        // Set baseline offset to the maximum ascent (distance above baseline)
        // This way, the 'y' parameter in render_text represents the bottom of the tallest character
        self.baseline_offset = -min_bearing_y as f32;

        // Second pass: generate textures
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

                    // IMPORTANT: single-channel bitmap rows may not be 4-byte aligned.
                    // Tell GL unpack alignment = 1 to avoid row stride padding which
                    // otherwise can cause strange distortions (appears like shear).
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

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

                    // Reset to default alignment after upload to be safe for other uploads
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                }

                // Store the character with its bounding box bearings
                self.characters.insert(
                    ch,
                    Character {
                        texture_id,
                        size: Vector2::new(width, height),
                        bearing: Vector2::new(bb.min.x, bb.min.y),
                        advance: h_metrics.advance_width,
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
                        advance: h_metrics.advance_width,
                    },
                );
            }
        }
    }
}