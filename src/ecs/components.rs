use cgmath::Vector3;

use crate::model::objload::ModelTrait;

pub struct Renderable {
    pub model: Box<dyn ModelTrait>,  // Assuming this contains mesh data
}

pub struct Velocity {
    pub direction: Vector3<f32>,
    pub speed: f32,
}