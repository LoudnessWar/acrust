use std::collections::HashMap;
use std::any::{Any, TypeId};
use cgmath::Vector3;

use crate::graphics::camera::Camera;
use crate::graphics::gl_wrapper::ForwardPlusRenderer;
use crate::graphics::texture_manager::TextureManager;
// Importing your existing types (adjust paths as needed)
use crate::model::transform::WorldCoords;
use crate::model::objload::{Model, ModelTrait};
use crate::ecs::components::Renderable;
use crate::ecs::player::Player;
use glfw::RenderContext;

use super::components::Velocity;

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
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityRegistry::new(),
            movement: MovementSystem::new(),
            render: RenderSystem::new(),
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
}