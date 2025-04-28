use std::collections::HashMap;
use std::any::{Any, TypeId};
use cgmath::{Vector3, Quaternion, Rad};
use crate::model::transform::WorldCoords; // reusing original WorldCoords
use crate::model::objload::Model;
use super::player::Player;
use super::components::Renderable;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub name: String,
}

pub trait Component: Any + Send + Sync {}

impl<T: Any + Send + Sync> Component for T {}

pub struct World {
    entities: Vec<Entity>,
    component_storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
    next_id: u32,
}

type EntityId = u32;

trait ComponentStorage: Send + Sync {
    fn remove(&mut self, entity: &Entity);
    fn contains(&self, entity: &Entity) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct Storage<T: Component> {
    components: HashMap<u32, T>,
}

impl<T: Component> ComponentStorage for Storage<T> {
    fn remove(&mut self, entity: &Entity) {
        self.components.remove(&entity.id);
    }

    fn contains(&self, entity: &Entity) -> bool {
        self.components.contains_key(&entity.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            component_storages: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn create_entity(&mut self, name: &str) -> Entity {
        let entity = Entity {
            id: self.next_id,
            name: name.to_string(),
        };
        self.next_id += 1;
        self.entities.push(entity.clone());
        entity
    }

    pub fn add_component<T: Component>(&mut self, entity: &Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .component_storages
            .entry(type_id)
            .or_insert_with(|| Box::new(Storage::<T> { components: HashMap::new() }));

        let storage = storage.as_any_mut().downcast_mut::<Storage<T>>().unwrap();
        storage.components.insert(entity.id, component);
    }

    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        self.component_storages
            .get(&TypeId::of::<T>())
            .and_then(|s| s.as_any().downcast_ref::<Storage<T>>())
            .and_then(|s| s.components.get(&entity.id))
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: &Entity) -> Option<&mut T> {
        self.component_storages
            .get_mut(&TypeId::of::<T>())
            .and_then(|s| s.as_any_mut().downcast_mut::<Storage<T>>())
            .and_then(|s| s.components.get_mut(&entity.id))
    }

    pub fn spawn_player(&mut self, name: &str, x: f32, y: f32, z: f32, rotation: f32) -> (Entity, Player) {
        let entity = self.create_entity(name);
        let coords = WorldCoords::new(x, y, z, rotation);
        self.add_component(&entity, coords.clone());
        let player = Player::new_coords(coords);
        (entity, player)
    }

    pub fn spawn_model(&mut self, name: &str, coords: WorldCoords, mut model: Model) -> (Entity, Model) {
        let entity = self.create_entity(name);
        self.add_component(&entity, coords.clone());//i dont like having this be clone TODO myabe find a way around
        model.set_position(coords.position);
        (entity, model)
    }

    pub fn update_renderable_transforms(&mut self) {
        for entity in &self.entities {
            if let (Some(coords), Some(renderable)) = (
                self.get_component::<WorldCoords>(entity),
                self.get_component_mut::<Renderable>(entity)
            ) {
                renderable.model.set_position(coords.position);
            }
        }
    }

    // Render system (immutable access only)
    pub fn render_entities(&self, render_context: &mut RenderContext) {
        for entity in &self.entities {
            if let Some(renderable) = self.get_component::<Renderable>(entity) {
                // Assuming your existing render function takes &ModelTrait
                render_context.render_model(&*renderable.model);
            }
        }
    }
}

// Extend existing WorldCoords to be ECS-compatible
// impl Component for WorldCoords {}
