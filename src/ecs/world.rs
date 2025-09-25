use std::collections::HashMap;
use std::any::{Any, TypeId};
use cgmath::{Vector2, Vector3, Vector4};

use crate::graphics::camera::Camera;
use crate::graphics::gl_wrapper::ForwardPlusRenderer;
use crate::graphics::texture_manager::TextureManager;
// Importing your existing types (adjust paths as needed)
use crate::model::transform::WorldCoords;
use crate::model::objload::{Model, ModelTrait};
use crate::ecs::components::Renderable;
use crate::ecs::player::Player;
use crate::user_interface::text_render::TextRenderer;
use glfw::RenderContext;

use super::components::Velocity;

use super::UI_components::*;

// Entity remains simple
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub name: String,
}

// Component systems are separated
pub trait Component: Any + Send + Sync {}
impl<T: Any + Send + Sync> Component for T {}

// Generic component storage
pub struct ComponentStorage<T: Component> {
    components: HashMap<u32, T>,
}

impl<T: Component> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity_id: u32, component: T) {
        self.components.insert(entity_id, component);
    }

    pub fn get(&self, entity_id: u32) -> Option<&T> {
        self.components.get(&entity_id)
    }

    pub fn get_mut(&mut self, entity_id: u32) -> Option<&mut T> {
        self.components.get_mut(&entity_id)
    }

    pub fn contains(&self, entity_id: u32) -> bool {
        self.components.contains_key(&entity_id)
    }

    pub fn remove(&mut self, entity_id: u32) {
        self.components.remove(&entity_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u32, &T)> {
        self.components.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u32, &mut T)> {
        self.components.iter_mut()
    }
}

// Registry for all entities
pub struct EntityRegistry {
    entities: HashMap<u32, Entity>,
    next_id: u32,
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_entity(&mut self, name: &str) -> Entity {
        let entity = Entity {
            id: self.next_id,
            name: name.to_string(),
        };
        self.next_id += 1;
        self.entities.insert(entity.id, entity.clone());
        entity
    }

    pub fn get_entity(&self, entity_id: u32) -> Option<&Entity> {
        self.entities.get(&entity_id)
    }

    pub fn remove_entity(&mut self, entity_id: u32) {
        self.entities.remove(&entity_id);
    }

    pub fn all_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }
}

// OK SO LIKE... PLAYER... coolio needs to be like implinebted into this better...
//same with the camera... TODO
pub struct MovementSystem {
    coords: ComponentStorage<WorldCoords>,
    velocities: ComponentStorage<Velocity>,
}

impl MovementSystem {
    pub fn new() -> Self {
        Self {
            coords: ComponentStorage::new(),
            velocities: ComponentStorage::new(),
        }
    }

    pub fn add_coords(&mut self, entity_id: u32, coords: WorldCoords) {
        self.coords.insert(entity_id, coords);
    }

    pub fn add_velocity(&mut self, entity_id: u32, velocity: Velocity) {
        self.velocities.insert(entity_id, velocity);
    }

    pub fn get_coords(&self, entity_id: u32) -> Option<&WorldCoords> {
        self.coords.get(entity_id)
    }

    pub fn get_coords_mut(&mut self, entity_id: u32) -> Option<&mut WorldCoords> {
        self.coords.get_mut(entity_id)
    }

    pub fn update(&mut self, delta_time: f32) {
        // Safe to iterate and modify because we own all the data
        for (entity_id, velocity) in self.velocities.iter() {
            //println!("first: {}", entity_id);
            if let Some(coords) = self.coords.get_mut(*entity_id) {
                //println!("second: {}", entity_id);
                coords.position += velocity.direction * velocity.speed * delta_time;
            }
        }
    }
}

pub struct RenderSystem {
    renderables: ComponentStorage<Renderable>,
}

impl RenderSystem {
    pub fn new() -> Self {
        Self {
            renderables: ComponentStorage::new(),
        }
    }

    pub fn add_renderable(&mut self, entity_id: u32, renderable: Renderable) {
        self.renderables.insert(entity_id, renderable);
    }

    pub fn get_renderable(&self, entity_id: u32) -> Option<&Renderable> {
        self.renderables.get(entity_id)
    }

    pub fn get_renderable_mut(&mut self, entity_id: u32) -> Option<&mut Renderable> {
        self.renderables.get_mut(entity_id)
    }

    pub fn update_transforms(&mut self, movement_system: &MovementSystem) {
        // No borrow conflicts because we borrow from different systems
        for (entity_id, renderable) in self.renderables.iter_mut() {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                renderable.model.set_position(coords.position);
            }
        }
    }

    //TODO redo this so its not so much a lot of this just could be stored in the render system ecs system thingy i thinky
    pub fn render(
        &self, 
        render_context: &mut ForwardPlusRenderer,
        camera: &Camera,
        width: u32,
        height: u32,
        texture_manager: &TextureManager,
    ) {
        let models: Vec<&Box<dyn ModelTrait>> = self.renderables
            .iter()
            .map(|(_, r)| &r.model)
            .collect();

        render_context.render(models, camera, width, height, texture_manager);
    }
}

// World just coordinates between systems
pub struct World {
    pub entities: EntityRegistry,
    pub movement: MovementSystem,
    pub render: RenderSystem,
    pub ui: UISystem,
}

impl World {
    // pub fn new() -> Self {
    //     Self {
    //         entities: EntityRegistry::new(),
    //         movement: MovementSystem::new(),
    //         render: RenderSystem::new(),
    //         ui: UISystem::new(0.0, 0.0),//TODO fix this later because I know a lot of this is going to be really bad at the start because Im kinda just forcing my old UI system into the space of a new ECS one
    //     }
    // }

    pub fn new_with_ui(screen_width: f32, screen_height: f32, text_renderer: TextRenderer) -> Self {
        Self {
            entities: EntityRegistry::new(),
            movement: MovementSystem::new(),
            render: RenderSystem::new(),
            ui: UISystem::new(screen_width, screen_height, text_renderer),
        }
    }

    pub fn create_entity(&mut self, name: &str) -> Entity {
        self.entities.create_entity(name)
    }

    pub fn spawn_player(&mut self, name: &str, x: f32, y: f32, z: f32, rotation: f32) -> Entity {
        let entity = self.create_entity(name);
        let coords = WorldCoords::new(x, y, z, rotation);
        
        self.movement.add_coords(entity.id, coords);
        self.movement.add_velocity(entity.id, Velocity { 
            direction: Vector3::new(0.0, 0.0, 0.0), 
            speed: 0.1 
        });
        
        entity
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update physics first
        self.movement.update(delta_time);
        
        // Then update renderables
        self.render.update_transforms(&self.movement);
    }

    pub fn render(
        &self,
        render_context: &mut ForwardPlusRenderer,
        camera: &Camera,
        width: u32,
        height: u32,
        texture_manager: &TextureManager,
    ) {
        self.render.render(render_context, camera, width, height, texture_manager);
    }

    pub fn create_ui_button(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, text: String) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        self.ui.add_style(entity.id, UIStyle::new().with_color(Vector4::new(0.7, 0.7, 0.7, 1.0)));
        self.ui.add_button(entity.id);
        self.ui.add_text(entity.id, text, 16.0);
        
        entity
    }
    
    pub fn create_ui_container(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, layout: UILayout) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        self.ui.add_style(entity.id, UIStyle::new().with_color(Vector4::new(0.2, 0.2, 0.2, 0.8)));//make this dynamic and optional later
        self.ui.add_layout(entity.id, layout);
        
        entity
    }
    
    pub fn create_ui_text(&mut self, name: &str, position: Vector2<f32>, text: String, font_size: f32) -> Entity {
        let entity = self.create_entity(name);
        
        // Create text component first
        self.ui.add_text(entity.id, text.clone(), font_size);
        
        // Calculate dimensions automatically
        let scale = font_size / 24.0;
        let (width, height) = self.ui.text_renderer.measure_text(&text, scale);
        
        // Create transform with calculated size
        self.ui.add_transform(entity.id, UITransform::new(position, Vector2::new(width, height)));
        self.ui.add_style(entity.id, UIStyle::new());
        
        entity
    }
    
    // Enhanced button creation with text
    pub fn create_ui_button_with_text(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, text: String, font_size: f32) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        self.ui.add_style(entity.id, UIStyle::new().with_color(Vector4::new(0.7, 0.7, 0.7, 1.0)));
        self.ui.add_button(entity.id);
        self.ui.add_text(entity.id, text, font_size);
        
        entity
    }
    
    // Method to update text content
    pub fn update_ui_text(&mut self, entity_id: u32, new_text: String) {
        self.ui.update_text(entity_id, new_text);
        
        // Auto-resize if needed
        if let Some((width, height)) = self.ui.get_text_dimensions(entity_id) {
            if let Some(transform) = self.ui.get_transform_mut(entity_id) {
                transform.size = Vector2::new(width, height);
                self.ui.layout_dirty = true;
            }
        }
    }
    
    // Get text content
    pub fn get_ui_text(&self, entity_id: u32) -> Option<&str> {
        self.ui.texts.get(entity_id).map(|t| t.text.as_str())
    }
    
    // Helper to add child to parent
    pub fn add_ui_child(&mut self, parent_id: u32, child_id: u32) {
        self.ui.add_parent(child_id, Some(parent_id));
    }

    pub fn update_ui(&mut self, delta_time: f32, mouse_pos: (f64, f64), mouse_down: bool, mouse_clicked: bool) {
        // Update physics first
        self.movement.update(delta_time);
        
        // Update renderables
        self.render.update_transforms(&self.movement);
        
        // Update UI
        self.ui.update_input(mouse_pos, mouse_down, mouse_clicked);
        self.ui.update_layout();
    }   

    pub fn render_all(
        &self,
        render_context: &mut ForwardPlusRenderer,
        ui_shader: &crate::graphics::gl_wrapper::ShaderProgram,
        camera: &Camera,
        width: u32,
        height: u32,
        texture_manager: &TextureManager,
    ) {
        // Render 3D world
        self.render.render(render_context, camera, width, height, texture_manager);
        
        // Render UI on top
        self.ui.render(ui_shader);
    }
    
    // UI query methods
    pub fn is_ui_button_clicked(&self, entity_id: u32) -> bool {
        self.ui.is_button_clicked(entity_id)
    }
    
    pub fn is_ui_button_hovered(&self, entity_id: u32) -> bool {
        self.ui.is_button_hovered(entity_id)
    }
    
    // Safe UI element updates
    pub fn update_ui_element_position(&mut self, entity_id: u32, position: Vector2<f32>) {
        if let Some(transform) = self.ui.get_transform_mut(entity_id) {
            transform.position = position;
            // Layout will be recalculated automatically
        }
    }
    
    pub fn update_ui_element_color(&mut self, entity_id: u32, color: Vector4<f32>) {
        if let Some(style) = self.ui.styles.get_mut(entity_id) {
            style.color = color;
        }
    }
}