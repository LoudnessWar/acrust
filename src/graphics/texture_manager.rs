use std::collections::HashMap;
use std::path::Path;

use image::GenericImageView;

pub struct TextureManager {
    textures: HashMap<String, u32>, // I wonder if arc would be useful here
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(&mut self, file_path: &str) -> Result<u32, String> {
        if let Some(&texture_id) = self.textures.get(file_path) {
            // Return cached texture ID
            return Ok(texture_id);
        }

        let img = image::open(&Path::new(file_path))
            .map_err(|e| format!("Failed to load image: {}", e))?;
        let data = img.to_rgba8();//.as_rgba8().ok_or("Image format is not RGBA8")?;//ok just uuh make it rgb8 ig
        let (width, height) = img.dimensions();//a whole package fro one line come back to this

        let mut texture_id: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            // Upload texture data
            println!("uploaded texture data");
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        // Cache the texture ID
        self.textures.insert(file_path.to_string(), texture_id);

        Ok(texture_id)
    }

    pub fn get_texture(&self, file_path: &str) -> Option<u32> {
        self.textures.get(file_path).copied()
    }
}
