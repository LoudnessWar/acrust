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
    line_height: f32,
    tab_width: f32,
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
            line_height: 0.0,     // <- Initialized here
            tab_width: 0.0,
        }
    }

    pub fn render_text(
        &self,
        text: &str,
        start_x: f32,
        start_y: f32,
        scale: f32,
        color: Vector3<f32>,
        projection: &Matrix4<f32>,
    ) {
        self.shader.bind();
        self.vao.bind();

        // Set text color uniform
        self.shader.set_uniform3f("textColor", &color);
        self.shader.set_matrix4fv_uniform("projection", &projection);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let mut x = start_x;
        let mut y = start_y;
        let line_height = self.line_height * scale;
        let tab_width = self.tab_width * scale;

        for c in text.chars() {
            match c {
                '\n' => {
                    // Move to next line
                    x = start_x;
                    y += line_height; // Move down (assuming y increases upward)
                    continue;
                }
                '\r' => {
                    // Carriage return - move to start of current line
                    x = start_x;
                    continue;
                }
                '\t' => {
                    // Tab character - advance to next tab stop
                    let tab_stops = (x - start_x) / tab_width;
                    let next_tab = ((tab_stops as i32) + 1) as f32;
                    x = start_x + (next_tab * tab_width);
                    continue;
                }
                '\0' => {
                    // Skip null characters
                    continue;
                }
                _ => {}
            }

            if let Some(ch) = self.characters.get(&c) {
                if ch.texture_id == 0 {
                    x += ch.advance * scale;
                    continue;
                }

                // Fixed positioning: align all characters to the same baseline
                let xpos = x + ch.bearing.x as f32 * scale;
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

                let indices: [i32; 6] = [0, 1, 2, 0, 2, 3];

                // Upload vertex data per-glyph
                self.vbo.bind();
                self.vbo.store_f32_data(&vertices);

                self.ebo.bind();
                self.ebo.store_i32_data(&indices);

                unsafe {
                    gl::BindTexture(gl::TEXTURE_2D, ch.texture_id);
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

        let total_height = (max_bearing_y - min_bearing_y) as f32;
        self.line_height = total_height * 1.2; // 1.2 is a common line spacing multiplier

        // Initialize tab width (4 spaces worth)
        let space_glyph = font.glyph(' ').scaled(scale);
        let space_advance = space_glyph.h_metrics().advance_width;
        self.tab_width = space_advance * 4.0;

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

    /// Get the calculated line height for this font
    pub fn get_line_height(&self) -> f32 {
        self.line_height
    }

    /// Set custom line height multiplier (default is 1.2)
    pub fn set_line_height_multiplier(&mut self, multiplier: f32) {
        // Recalculate line height with new multiplier
        let base_height = self.line_height / 1.2; // Get original base height
        self.line_height = base_height * multiplier;
    }

    /// Set custom tab width (in terms of space characters, default is 4)
    pub fn set_tab_width(&mut self, spaces: f32) {
        let space_glyph = match self.characters.get(&' ') {
            Some(ch) => ch.advance,
            None => self.line_height * 0.25, // Fallback
        };
        self.tab_width = space_glyph * spaces;
    }

    /// Calculate the dimensions of a text string (width, height)
    pub fn measure_text(&self, text: &str, scale: f32) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut max_width = 0.0f32;
        let mut height = self.line_height * scale;

        let line_height = self.line_height * scale;
        let tab_width = self.tab_width * scale;

        for c in text.chars() {
            match c {
                '\n' => {
                    max_width = max_width.max(width);
                    width = 0.0;
                    height += line_height;
                }
                '\r' => {
                    width = 0.0;
                }
                '\t' => {
                    let tab_stops = width / tab_width;
                    let next_tab = ((tab_stops as i32) + 1) as f32;
                    width = next_tab * tab_width;
                }
                ' ' => {
                    if let Some(ch) = self.characters.get(&' ') {
                        width += ch.advance * scale;
                    } else {
                        width += self.line_height * 0.25 * scale;
                    }
                }
                '\0' => {
                    // Skip null characters
                }
                _ => {
                    if let Some(ch) = self.characters.get(&c) {
                        width += ch.advance * scale;
                    } else {
                        width += self.line_height * 0.5 * scale;
                    }
                }
            }
        }

        max_width = max_width.max(width);
        (max_width, height)
    }
}