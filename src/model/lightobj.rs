use cgmath::Vector3;

use crate::graphics::lightmanager::*;

pub struct LightSource {
    emission_color: Vector3<f32>,
    emission_intensity: f32,//maybe just u8
    ambient_color: Vector3<f32>,
    specular_color: Vector3<f32>,
    position: Vector3<f32>,
}

impl LightSource {
    pub fn new(emission_color: Vector3<f32>,
        emission_intensity: f32,
        ambient_color: Vector3<f32>,
        specular_color: Vector3<f32>,
        position: Vector3<f32>,) -> Self{
            Self { emission_color: emission_color, 
                emission_intensity: emission_intensity, 
                ambient_color: ambient_color, 
                specular_color: specular_color, 
                position: position 
            }
    }
}

impl LightTrait for LightSource{
    fn get_emission_color(&self) -> &Vector3<f32> {
        &self.emission_color
    }

    fn get_emission_intensity(&self) -> &f32 {
        &self.emission_intensity
    }

    fn get_ambient_color(&self) -> &Vector3<f32> {
        &self.ambient_color
    }

    fn get_specular_color(&self) -> &Vector3<f32> {
        &self.specular_color
    }

    fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    //im always never sure with setters if I should like
    //have it take a reference or not. Refrence more readable but less flexible so i guess
    //it depends both prolly have their ups and downs
    fn set_emission_color(&mut self, color: Vector3<f32>) {
        self.emission_color = color;
    }

    fn set_emission_intensity(&mut self, intensity: f32) {
        self.emission_intensity = intensity;
    }

    fn set_ambient_color(&mut self, color: Vector3<f32>) {
        self.ambient_color = color;
    }

    fn set_specular_color(&mut self, color: Vector3<f32>) {
        self.specular_color = color;
    }
}

//erm ok so like... whatta they gonna do ayy
//basically. I need these hecking light sources to like
//be in a list yo