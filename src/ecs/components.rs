use crate::model::objload::ModelTrait;

pub struct Renderable {
    pub model: Box<dyn ModelTrait>,  // Assuming this contains mesh data
}