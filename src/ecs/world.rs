use std::collections::HashMap;
use std::any::{Any};
use cgmath::{InnerSpace, Vector2, Vector3, Vector4};

use crate::ecs::physics::{PhysicsEntityData, PhysicsSystem, PhysicsType, PhysicsEntity};
use crate::graphics::camera::Camera;
use crate::graphics::gl_wrapper::ForwardPlusRenderer;
use crate::graphics::texture_manager::TextureManager;
// Importing your existing types (adjust paths as needed)
use crate::model::transform::WorldCoords;
use crate::model::objload::{ModelTrait};
use crate::ecs::components::Renderable;
// use crate::ecs::player::Player;
use crate::user_interface::text_render::TextRenderer;
use super::collision_system::{CollisionSystem, Collider, CollisionShape, CollisionEvent};
// use glfw::RenderContext;

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
impl<T: Any + Send + Sync> Component for T {} //this is actaully really cool you can do this

// Generic component storage like wow so intresting keep on reading buddy
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

// Registry for all entities bro again who cares about this shit
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

    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }

    //todo this is rather inefficent, values creates an iterator over all the values in the hashmap
    //but that includes empty ones
    //find just looks through an iterator until it finds the value that is is looking for
    //also value is an iterator of only the values btw so i am not getting the hashmaps id
    //I could if I wanted this to be faster have a separate hashmap that maps names to entity ids
    //and i dont think that that is a terrible idea simply because like it doesnt take up that much space
    //my real reservation is the fact that you would have to have unique names yk... or wait no im stupid you wouldnt nvm wait you would nvm the nvm bc the name would be the key not the value if used
    //like this
    pub fn get_entity_by_name(&self, name: &str) -> Option<&Entity> {
        self.entities.values().find(|e| e.name == name)
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

    pub fn get_velocity(&self, entity_id: u32) -> Option<&Velocity> {
        self.velocities.get(entity_id)
    }

    pub fn get_velocity_mut(&mut self, entity_id: u32) -> Option<&mut Velocity> {
        self.velocities.get_mut(entity_id)
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

//this one will eventually give me cancer i can tell, mainly because if we think about it with all scincerity we need multiple render types multiple like fucking render modes, its a hoot and hollah
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
        //but also this should just be changed bc the only reason that the models themselves maintain a rendering system is because of situations in which
        //you are purpousfully avoiding the ecs system...
        //todo change this
        //like all it does is it clones position to the fucking models worldpersonal worldcoords
        //thing 
        for (entity_id, renderable) in self.renderables.iter_mut() {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                renderable.model.set_position(coords.position);
                renderable.model.set_rotation_from_quaternion(coords.rotation);
            }
        }
    }

    //TODO redo this so its not so much a lot of this just could be stored in the render system ecs system thingy i thinky
    //TODO i dont know what i was yapping about above, it got twisted uuuh yeah maybe uuh looking at this why is forwardplusrender the only render context like what if we had other render contexts guys... just a thought
    pub fn render(
        &self, 
        render_context: &mut ForwardPlusRenderer,
        camera: &Camera,
        width: u32,
        height: u32,
        texture_manager: &TextureManager,
    ) {

        //todo lol i like... i know that this is incredibly absurd.... i will change i will 
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
    pub collision: CollisionSystem,
    pub physics: PhysicsSystem,
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

    // pub fn new_with_ui(screen_width: f32, screen_height: f32, text_renderer: TextRenderer) -> Self {
    //     Self {
    //         entities: EntityRegistry::new(),
    //         movement: MovementSystem::new(),
    //         render: RenderSystem::new(),
    //         ui: UISystem::new(screen_width, screen_height, text_renderer),
    //     }
    // }

    pub fn new_with_ui_and_collision(screen_width: f32, screen_height: f32, text_renderer: TextRenderer) -> Self {
        Self {
            entities: EntityRegistry::new(),
            movement: MovementSystem::new(),
            render: RenderSystem::new(),
            ui: UISystem::new(screen_width, screen_height, text_renderer),
            collision: CollisionSystem::new(),
            physics: PhysicsSystem::new(),
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

    pub fn spawn_player_with_collision(&mut self, name: &str, x: f32, y: f32, z: f32, rotation: f32, collider: Collider) -> Entity {
        let entity = self.create_entity(name);
        let coords = WorldCoords::new(x, y, z, rotation);
        
        self.movement.add_coords(entity.id, coords);
        self.movement.add_velocity(entity.id, Velocity { 
            direction: Vector3::new(0.0, 0.0, 0.0), 
            speed: 0.1 
        });
        
        // Add collision
        self.collision.add_collider(entity.id, collider);
        
        entity
    }

    //lol the naming needs to be better
    pub fn create_static(&mut self, name: &str, x: f32, y: f32, z: f32, collider: Collider) -> Entity {
        let entity = self.create_entity(name);
        let coords = WorldCoords::new(x, y, z, 0.0);
        
        self.movement.add_coords(entity.id, coords);
        // No velocity - it's static
        
        self.collision.add_collider(entity.id, collider);
        
        entity
    }

    pub fn create_moving_entity(&mut self, name: &str, x: f32, y: f32, z: f32, velocity: Velocity, collider: Collider) -> Entity {
        let entity = self.create_entity(name);
        let coords = WorldCoords::new(x, y, z, 0.0);
        
        self.movement.add_coords(entity.id, coords);
        self.movement.add_velocity(entity.id, velocity);
        self.collision.add_collider(entity.id, collider);
        
        entity
    }
    
    pub fn create_trigger_zone(&mut self, name: &str, x: f32, y: f32, z: f32, collider: Collider) -> Entity {
        let entity = self.create_entity(name);
        let coords = WorldCoords::new(x, y, z, 0.0);
        
        self.movement.add_coords(entity.id, coords);
        self.collision.add_collider(entity.id, collider.as_trigger());
        
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
        
        // Create button style with dark blue background and white text
        let button_style = UIStyle::button_style(
            Vector4::new(0.1, 0.3, 0.4, 1.0), // Background color
            Vector4::new(1.0, 1.0, 1.0, 1.0)  // Text color (white)
        );
        self.ui.add_style(entity.id, button_style);
        
        self.ui.add_button(entity.id);
        self.ui.add_text(entity.id, text, 16.0);
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }

    pub fn create_ui_button_colored(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, text: String, bg_color: Vector4<f32>, text_color: Vector4<f32>) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        
        let button_style = UIStyle::button_style(bg_color, text_color);
        self.ui.add_style(entity.id, button_style);
        
        self.ui.add_button(entity.id);
        self.ui.add_text(entity.id, text, 16.0);
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }


    //literally just a container without a layout
    //for just splatting a shape down wherever
    pub fn create_ui_panel(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, color: Vector4<f32>) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        self.ui.add_style(entity.id, UIStyle::new().with_color(color));
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }
    
    pub fn create_ui_container(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, layout: UILayout) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        self.ui.add_style(entity.id, UIStyle::new().with_color(Vector4::new(0.2, 0.2, 0.2, 0.8)));//make this dynamic and optional later
        self.ui.add_layout(entity.id, layout);

        self.ui.add_z_index(entity.id, -1);
        
        entity
    }
    
    pub fn create_ui_text(&mut self, name: &str, position: Vector2<f32>, text: String, font_size: f32, color: Vector4<f32>) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, Vector2::new(0.0, 0.0))); // Size will be auto-calculated
        
        // Create text-only style (no background rendering)
        let text_style = UIStyle::new()
            .with_text_color(color)
            .text_only(); // This sets render_background = false
        self.ui.add_style(entity.id, text_style);
        
        self.ui.add_text(entity.id, text, font_size);
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }

    //idk this might be a waste of space
    pub fn create_ui_label(&mut self, name: &str, position: Vector2<f32>, text: String) -> Entity {
        self.create_ui_text(name, position, text, 16.0, Vector4::new(0.0, 0.0, 0.0, 1.0)) // Black text
    }
        
    // lol they are the same now so this is like no longer needed
    pub fn create_ui_button_with_text(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, text: String, font_size: f32) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        self.ui.add_style(entity.id, UIStyle::new().with_color(Vector4::new(0.7, 0.7, 0.7, 1.0)));
        self.ui.add_button(entity.id);
        self.ui.add_text(entity.id, text, font_size);

        self.ui.add_z_index(entity.id, 0);
        
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

    // pub fn update_ui_with_collision(&mut self, delta_time: f32, mouse_pos: (f64, f64), mouse_down: bool, mouse_clicked: bool) {
    //     // Update movement
    //     self.movement.update(delta_time);
        
    //     // Update collisions
    //     self.collision.update(&mut self.movement, delta_time);
        
    //     // Update renderables
    //     self.render.update_transforms(&self.movement);
        
    //     // Update UI
    //     self.ui.update_input(mouse_pos, mouse_down, mouse_clicked);
    //     self.ui.update_layout();
    // }

    pub fn update_with_physics(&mut self, delta_time: f32) {
        // 1. Apply forces and integrate physics
        self.physics.update(&mut self.movement, delta_time);
        
        // 2. Update movement (velocity -> position)
        self.movement.update(delta_time);
        
        // 3. Check and resolve collisions
        self.collision.update(&mut self.movement, &mut self.physics, delta_time);
        
        // 4. Update renderables
        self.render.update_transforms(&self.movement);
    }

    pub fn update_with_physics_and_ui(&mut self, delta_time: f32, input_system: &mut crate::input::input::InputSystem) {
        // Physics and collision
        self.physics.update(&mut self.movement, delta_time);
        self.movement.update(delta_time);
        self.collision.update(&mut self.movement, &mut self.physics, delta_time);
        self.render.update_transforms(&self.movement);
        
        // UI
        let mouse_pos = input_system.get_mouse_position();
        let mouse_down = input_system.is_mouse_button_held(&crate::input::input::CLICKS::Left);
        let mouse_clicked = input_system.is_mouse_button_just_pressed(&crate::input::input::CLICKS::Left);
        
        self.ui.update_input(mouse_pos, mouse_down, mouse_clicked);
        self.ui.update_text_input(input_system);
        self.ui.update_text_inputs(delta_time);
        self.ui.update_layout();
    }

    pub fn update_ui_with_text_input_and_collision(&mut self, delta_time: f32, input_system: &mut crate::input::input::InputSystem) {
        // Update movement and collision
        self.movement.update(delta_time);
        self.physics.update(&mut self.movement, delta_time);
        self.collision.update(&mut self.movement,  &mut self.physics, delta_time);
        //self.collision.update_no_physics(&mut self.movement, delta_time);
        self.render.update_transforms(&self.movement);
        
        // Update regular UI input
        let mouse_pos = input_system.get_mouse_position();
        let mouse_down = input_system.is_mouse_button_held(&crate::input::input::CLICKS::Left);
        let mouse_clicked = input_system.is_mouse_button_just_pressed(&crate::input::input::CLICKS::Left);
        
        self.ui.update_input(mouse_pos, mouse_down, mouse_clicked);
        self.ui.update_text_input(input_system);
        self.ui.update_text_inputs(delta_time);
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
            style.background_color = color;//todo just change background color to color later
        }
    }

    pub fn create_ui_text_input(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, placeholder: String) -> Entity {
        let entity = self.create_entity(name);
        
        // Set up transform
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        
        // Create style with white background and border-like appearance
        let input_style = UIStyle::new()
            .with_color(Vector4::new(1.0, 1.0, 1.0, 1.0)) // White background
            .with_text_color(Vector4::new(0.0, 0.0, 0.0, 1.0)); // Black text
        self.ui.add_style(entity.id, input_style);
        
        // Add text input component
        self.ui.add_text_input(entity.id, UITextInput::new(placeholder));
        
        // Set z-index so it renders properly
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }
    
    // Create a text input with custom styling
    pub fn create_ui_text_input_styled(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, 
                                     placeholder: String, bg_color: Vector4<f32>, text_color: Vector4<f32>) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        
        let input_style = UIStyle::new()
            .with_color(bg_color)
            .with_text_color(text_color);
        self.ui.add_style(entity.id, input_style);
        
        self.ui.add_text_input(entity.id, UITextInput::new(placeholder));
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }
    
    // Create a text input with maximum length
    pub fn create_ui_text_input_with_limit(&mut self, name: &str, position: Vector2<f32>, size: Vector2<f32>, 
                                         placeholder: String, max_length: usize) -> Entity {
        let entity = self.create_entity(name);
        
        self.ui.add_transform(entity.id, UITransform::new(position, size));
        
        let input_style = UIStyle::new()
            .with_color(Vector4::new(1.0, 1.0, 1.0, 1.0))
            .with_text_color(Vector4::new(0.0, 0.0, 0.0, 1.0));
        self.ui.add_style(entity.id, input_style);
        
        let text_input = UITextInput::new(placeholder).with_max_length(max_length);
        self.ui.add_text_input(entity.id, text_input);
        self.ui.add_z_index(entity.id, 0);
        
        entity
    }
    
    // Update method that includes text input handling with your InputSystem
    pub fn update_ui_with_text_input(&mut self, delta_time: f32, input_system: &mut crate::input::input::InputSystem) {//dude maybe i should just like import and not do this shit
        // Update movement and render transforms
        self.movement.update(delta_time);
        self.render.update_transforms(&self.movement);
        
        // Update regular UI input
        let mouse_pos = input_system.get_mouse_position();
        let mouse_down = input_system.is_mouse_button_held(&crate::input::input::CLICKS::Left);
        let mouse_clicked = input_system.is_mouse_button_just_pressed(&crate::input::input::CLICKS::Left);
        
        self.ui.update_input(mouse_pos, mouse_down, mouse_clicked);
        
        // Update text inputs - this will consume relevant events from the input system
        self.ui.update_text_input(input_system);
        self.ui.update_text_inputs(delta_time);
        
        // Update layout
        self.ui.update_layout();
    }

    // Get the current text from a text input
    pub fn get_text_input_value(&self, entity_id: u32) -> Option<String> {
        self.ui.text_inputs.get(entity_id).map(|input| input.text.clone())
    }
    
    // Set the text in a text input
    pub fn set_text_input_value(&mut self, entity_id: u32, text: String) {
        if let Some(input) = self.ui.text_inputs.get_mut(entity_id) {
            input.text = text;
            input.cursor_position = input.text.len();
            input.reset_cursor_blink();
        }
    }
    
    // Clear a text input
    pub fn clear_text_input(&mut self, entity_id: u32) {
        if let Some(input) = self.ui.text_inputs.get_mut(entity_id) {
            input.text.clear();
            input.cursor_position = 0;
            input.reset_cursor_blink();
        }
    }
    
    // Check if a text input is focused
    pub fn is_text_input_focused(&self, entity_id: u32) -> bool {
        self.ui.text_inputs.get(entity_id)
            .map(|input| input.is_focused)
            .unwrap_or(false)
    }
    
    // Set focus on a text input (and remove focus from others)
    pub fn focus_text_input(&mut self, entity_id: u32) {
        // First, remove focus from all text inputs
        for (_, input) in self.ui.text_inputs.iter_mut() {
            input.is_focused = false;
        }
        
        // Then focus the specified one
        if let Some(input) = self.ui.text_inputs.get_mut(entity_id) {
            input.is_focused = true;
            input.reset_cursor_blink();
        }
    }
    
    // Remove focus from all text inputs
    pub fn clear_text_input_focus(&mut self) {
        for (_, input) in self.ui.text_inputs.iter_mut() {
            input.is_focused = false;
        }
    }

    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        self.collision.get_collision_events()
    }
    
    pub fn entity_collided_with(&self, entity_id: u32) -> Vec<u32> {
        self.collision.entity_collided_with(entity_id)
    }
    
    pub fn entities_collided(&self, entity_a: u32, entity_b: u32) -> bool {
        self.collision.entities_collided(entity_a, entity_b)
    }
    
    // Collision management methods
    pub fn add_entity_collider(&mut self, entity_id: u32, collider: Collider) {
        self.collision.add_collider(entity_id, collider);
    }
    
    pub fn remove_entity_collider(&mut self, entity_id: u32) {
        self.collision.remove_collider(entity_id);
    }
    
    pub fn get_entity_collider(&self, entity_id: u32) -> Option<&Collider> {
        self.collision.get_collider(entity_id)
    }
    
    pub fn set_collision_layers(&mut self, layer_a: u32, layer_b: u32, can_collide: bool) {
        self.collision.set_collision_layers(layer_a, layer_b, can_collide);
    }
    
    // Move an entity by a specific amount and handle collisions
    //todo add back later
    // pub fn move_entity(&mut self, entity_id: u32, delta: Vector3<f32>) {
    //     if let Some(coords) = self.movement.get_coords_mut(entity_id) {
    //         coords.position += delta;
    //     }
        
    //     // might want to check for collisions after manual movement
    //     // This is a simplified approach - in practice might want predictive collision
    //     self.collision.update(&mut self.movement, 0.0);
    // }
    
    // Set entity velocity
    pub fn set_entity_velocity(&mut self, entity_id: u32, velocity: Velocity) {
        self.movement.add_velocity(entity_id, velocity);
    }
    
    pub fn get_entity_velocity(&mut self, entity_id: u32) -> Option<&Velocity> {
        self.movement.velocities.get(entity_id)
    }

    pub fn spawn_physics_entity(&mut self, data: PhysicsEntityData) -> Entity {
        let entity = self.create_entity(&data.name);
        
        let coords = WorldCoords::new(
            data.position.x,
            data.position.y,
            data.position.z,
            data.rotation
        );
        self.movement.add_coords(entity.id, coords);
        
        let collider = data.collider;
        self.collision.add_collider(entity.id, collider);
        
        match data.phys_type {
            PhysicsType::Static => {
                // Static bodies don't move
                let rb = PhysicsEntity::static_body()
                    .with_restitution(data.restitution.unwrap_or(0.0))
                    .with_friction(data.friction.unwrap_or(0.5));
                self.physics.add_rigidbody(entity.id, rb);
            }
            
            PhysicsType::Dynamic => {
                let velocity = Velocity {
                    direction: data.velocity.unwrap_or(Vector3::new(0.0, 0.0, 0.0)).normalize(),
                    speed: data.velocity.unwrap_or(Vector3::new(0.0, 0.0, 0.0)).magnitude(),
                };
                self.movement.add_velocity(entity.id, velocity);
                
                let rb = PhysicsEntity::new(data.mass.unwrap_or(1.0))
                    .with_restitution(data.restitution.unwrap_or(0.3))
                    .with_friction(data.friction.unwrap_or(0.5));
                self.physics.add_rigidbody(entity.id, rb);
            }
            
            PhysicsType::Kinematic => {
                let velocity = Velocity {
                    direction: data.velocity.unwrap_or(Vector3::new(0.0, 0.0, 0.0)).normalize(),
                    speed: data.velocity.unwrap_or(Vector3::new(0.0, 0.0, 0.0)).magnitude(),
                };
                self.movement.add_velocity(entity.id, velocity);
                
                let rb = PhysicsEntity::kinematic();
                self.physics.add_rigidbody(entity.id, rb);
            }
            
            PhysicsType::Trigger => {
                // Triggers don't need physics
            }
        }
        
        entity
    }

    pub fn spawn_static_box(&mut self, name: &str, position: Vector3<f32>, size: Vector3<f32>) -> Entity {
            let entity = self.create_entity(name);
            
            self.movement.add_coords(entity.id, WorldCoords::new(position.x, position.y, position.z, 0.0));
            
            self.collision.add_collider(entity.id, Collider {
                shape: CollisionShape::Box {
                    width: size.x,
                    height: size.y,
                    depth: size.z,
                },
                is_trigger: false,
                layer: 0,
                offset: Vector3::new(0.0, 0.0, 0.0),
            });
            
            self.physics.add_rigidbody(entity.id, PhysicsEntity::static_body());
            
            entity
        }
    
    /// Spawn a dynamic sphere (ball, etc.)
    pub fn spawn_dynamic_sphere(&mut self, name: &str, position: Vector3<f32>, radius: f32, mass: f32) -> Entity {
        let entity = self.create_entity(name);
        
        self.movement.add_coords(entity.id, WorldCoords::new(position.x, position.y, position.z, 0.0));
        
        self.movement.add_velocity(entity.id, Velocity {
            direction: Vector3::new(0.0, 0.0, 0.0),
            speed: 0.0,
        });
        
        self.collision.add_collider(entity.id, Collider {
            shape: CollisionShape::Sphere { radius },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        });
        
        self.physics.add_rigidbody(entity.id, PhysicsEntity::new(mass));
        
        entity
    }
    
    /// Spawn a dynamic box
    pub fn spawn_dynamic_box(&mut self, name: &str, position: Vector3<f32>, size: Vector3<f32>, mass: f32) -> Entity {
        let entity = self.create_entity(name);
        
        self.movement.add_coords(entity.id, WorldCoords::new(position.x, position.y, position.z, 0.0));
        
        self.movement.add_velocity(entity.id, Velocity {
            direction: Vector3::new(0.0, 0.0, 0.0),
            speed: 0.0,
        });
        
        self.collision.add_collider(entity.id, Collider {
            shape: CollisionShape::Box {
                width: size.x,
                height: size.y,
                depth: size.z,
            },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        });
        
        self.physics.add_rigidbody(entity.id, PhysicsEntity::new(mass));
        
        entity
    }
    
    /// Spawn a bouncy ball (high restitution)
    pub fn spawn_bouncy_ball(&mut self, name: &str, position: Vector3<f32>, radius: f32, mass: f32) -> Entity {
        let entity = self.create_entity(name);
        
        self.movement.add_coords(entity.id, WorldCoords::new(position.x, position.y, position.z, 0.0));
        
        self.movement.add_velocity(entity.id, Velocity {
            direction: Vector3::new(0.0, 0.0, 0.0),
            speed: 0.0,
        });
        
        self.collision.add_collider(entity.id, Collider {
            shape: CollisionShape::Sphere { radius },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        });
        
        self.physics.add_rigidbody(entity.id, 
            PhysicsEntity::new(mass)
                .with_restitution(0.9)  // Very bouncy
                .with_friction(0.1)     // Low friction
        );
        
        entity
    }
    
    /// Spawn a kinematic platform (moves but not affected by physics)
    pub fn spawn_kinematic_platform(&mut self, name: &str, position: Vector3<f32>, size: Vector3<f32>, velocity: Vector3<f32>) -> Entity {
        let entity = self.create_entity(name);
        
        self.movement.add_coords(entity.id, WorldCoords::new(position.x, position.y, position.z, 0.0));
        
        let speed = velocity.magnitude();
        let direction = if speed > 0.001 {
            velocity.normalize()
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        };
        
        self.movement.add_velocity(entity.id, Velocity { direction, speed });
        
        self.collision.add_collider(entity.id, Collider {
            shape: CollisionShape::Box {
                width: size.x,
                height: size.y,
                depth: size.z,
            },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        });
        
        self.physics.add_rigidbody(entity.id, PhysicsEntity::kinematic());
        
        entity
    }
    
    // ========================================================================
    // HELPER METHODS - Apply forces, get rigidbodies, etc.
    // ========================================================================
    
    /// Apply a force to an entity (affects acceleration)
    pub fn apply_force_to_entity(&mut self, entity_id: u32, force: Vector3<f32>) {
        if let Some(rb) = self.physics.get_rigidbody_mut(entity_id) {
            rb.apply_force(force);
        }
    }
    
    /// Apply an impulse to an entity (instant velocity change)
    pub fn apply_impulse_to_entity(&mut self, entity_id: u32, impulse: Vector3<f32>) {
        if let Some(rb) = self.physics.get_rigidbody_mut(entity_id) {
            rb.apply_impulse(impulse);
        }
    }
    
    /// Set gravity for the entire world
    pub fn set_gravity(&mut self, gravity: Vector3<f32>) {
        self.physics.gravity = gravity;
    }

    pub fn set_position_directly(&mut self, entity_id: u32, new_position: Vector3<f32>) {
        if let Some(coords) = self.movement.get_coords_mut(entity_id) {
            coords.position = new_position;
        }
    }

    pub fn set_entity_velocity_directly(&mut self, entity_id: u32, new_velocity: Vector3<f32>) {
        if let Some(velocity) = self.movement.get_velocity_mut(entity_id) {
            velocity.direction = new_velocity.normalize();
            velocity.speed = new_velocity.magnitude();
        }
    }

    pub fn get_entity_id_by_name(&self, name: &str) -> Option<u32> {
        self.entities.get_entity_by_name(name).map(|e| e.id)
    }

    pub fn init_collision_shader(&mut self) {
        self.collision.init_collision_debug();
    }
    // Remove entity completely (from all systems)
    // pub fn remove_entity_completely(&mut self, entity_id: u32) {
    //     self.entities.remove_entity(entity_id);
    //     self.movement.coords.remove(entity_id);
    //     self.movement.velocities.remove(entity_id);
    //     self.render.renderables.remove(entity_id);
    //     self.collision.remove_collider(entity_id);
    //     // Remove from UI systems as needed
    //     self.ui.transforms.remove(entity_id);
    //     self.ui.styles.remove(entity_id);
    //     self.ui.buttons.remove(entity_id);
    //     self.ui.texts.remove(entity_id);
    //     self.ui.text_inputs.remove(entity_id);
    //     self.ui.z_indices.remove(entity_id);
    //     self.ui.layouts.remove(entity_id);
    //     self.ui.parents.remove(entity_id);
    // }
}