use cgmath::{Vector3, InnerSpace};
// use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::model::transform::WorldCoords;
use super::collision_system::{Collider, CollisionShape, CollisionEvent};
use super::components::Velocity;
use super::world::{MovementSystem, ComponentStorage};

//lol these are just to start
pub enum PhysicsType {
    Static,
    Dynamic,
    Kinematic,
    Trigger,
}

//now we dont actually need this but it helps with organization because all physics entities will have similar components
//this will eventaully be run through a funciton and just get turned into a not physics entity so our system can have its way with it

//good time to simplify the ecs

//we have the
//entity

//and a bunch of components are synced to the entitity

//entity <-- (render, movement, collision, ect)

//so now you have an entity but you dont really use the entity all its there for is to sync our components

//now we have the system

//the system acts on the components
//and has a component storage which stores the components of a certain type

//the system acts and calls on the components and such like rendering ect

//so then how do we not end up with a collision component that is attached to the wrong render component

//the component storage store the component and a entity id which is used to
//sync the components for the render so if we wanna move the render component we need the id and so if we update the system at the same time they will be in sync

//for our render we can actually see this because i had an error earlier,

//like the cube is just a rendered cube at a spot and a collider at the same spot, so i accidentally moved the cube but didnt move its collider and in fact they were not in the same spot
//basically they just happen to be in the same spot because i set them to be there and the components
//dont really know or care about each other
pub struct PhysicsEntityData { //this is more like physics entity data doe btw
    pub name: String,//probabl not needed
    pub phys_type: PhysicsType,

    //this is why i was not fond of this solution
    //TODO look into this and see if there is a better way to use world transforms or something here
    //though like... we dont really need to because it will be bigger then POD 
    pub position: Vector3<f32>,
    pub rotation: f32,

    pub collider: Collider,

    pub mass: Option<f32>,
    pub velocity: Option<Vector3<f32>>,
    pub angular_velocity: Option<f32>,

    pub restitution: Option<f32>,
    pub friction: Option<f32>,
    pub is_kinematic: bool,//lol this should maybe be in the collider nah i just need to remove the part in the collider that moves the stuff
    //todo 
}


//bascially just pattern mathcing
impl PhysicsEntityData {
    pub fn static_body(name: &str, position: Vector3<f32>, collider: Collider) -> Self {
        Self {
            name: name.to_string(),
            phys_type: PhysicsType::Static,
            position,
            rotation: 0.0,
            collider,
            mass: None,
            velocity: None,
            angular_velocity: None,
            restitution: Some(0.0),
            friction: Some(0.5),
            is_kinematic: false, 
        }
    }
    
    pub fn dynamic_body(name: &str, position: Vector3<f32>, collider: Collider, mass: f32) -> Self {
        Self {
            name: name.to_string(),
            phys_type: PhysicsType::Dynamic,
            position,
            rotation: 0.0,
            collider,
            mass: Some(mass),
            velocity: Some(Vector3::new(0.0, 0.0, 0.0)),
            angular_velocity: Some(0.0),
            restitution: Some(0.3),
            friction: Some(0.5),
            is_kinematic: false,
        }
    }
    
    pub fn kinematic_body(name: &str, position: Vector3<f32>, collider: Collider) -> Self {
        Self {
            name: name.to_string(),
            phys_type: PhysicsType::Kinematic,
            position,
            rotation: 0.0,
            collider,
            mass: None,
            velocity: Some(Vector3::new(0.0, 0.0, 0.0)),
            angular_velocity: None,
            restitution: None,
            friction: None,
            is_kinematic: true,
        }
    }
    
    pub fn trigger(name: &str, position: Vector3<f32>, shape: CollisionShape) -> Self {
        Self {
            name: name.to_string(),
            phys_type: PhysicsType::Trigger,
            position,
            rotation: 0.0,
            collider: Collider {
                shape,
                is_trigger: true,
                layer: 0,
                offset: Vector3::new(0.0, 0.0, 0.0),
            },
            mass: None,
            velocity: None,
            angular_velocity: None,
            restitution: None,
            friction: None,
            is_kinematic: false,
        }
    }
    
    pub fn with_velocity(mut self, velocity: Vector3<f32>) -> Self {
        self.velocity = Some(velocity);
        self
    }
    
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
    
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = Some(restitution);
        self
    }
    
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = Some(friction);
        self
    }
    
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.collider.layer = layer;
        self
    }
}

// pub struct RigidBody {
//     pub mass: f32,
//     pub restitution: f32,
//     pub friction: f32,
//     pub force_accumulator: Vector3<f32>,
// }


// pub struct PhysicsSystem {
//     rigidbodies: ComponentStorage<RigidBody>,
// }

//example of like a ridgid body but also want ridgid bodys anyway

#[derive(Debug, Clone)]
pub struct PhysicsEntity {
    pub mass: f32,
    pub inverse_mass: f32, // Cached for performance (0.0 = infinite mass/static)
    
    // Material properties
    pub restitution: f32,  // Bounciness (0.0 = no bounce, 1.0 = perfect bounce)
    pub friction: f32,     // Surface friction (0.0 = ice, 1.0 = rubber)
    
    // Force accumulator (cleared each frame)
    pub force: Vector3<f32>,
    pub impulse: Vector3<f32>, // For instant velocity changes
    
    // Linear damping (air resistance)
    pub linear_damping: f32, // 0.0 = no damping, 1.0 = full damping
    
    // Constraints
    pub is_kinematic: bool, // Moves but not affected by forces
    pub lock_rotation: bool,
    pub lock_axis: Vector3<bool>, // Lock movement on specific axes
}

impl PhysicsEntity {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            inverse_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            restitution: 0.3,
            friction: 0.5,
            force: Vector3::new(0.0, 0.0, 0.0),
            impulse: Vector3::new(0.0, 0.0, 0.0),
            linear_damping: 0.01,
            is_kinematic: false,
            lock_rotation: false,
            lock_axis: Vector3::new(false, false, false),
        }
    }
    
    pub fn static_body() -> Self {
        Self {
            mass: 0.0,
            inverse_mass: 0.0,
            restitution: 0.0,
            friction: 0.5,
            force: Vector3::new(0.0, 0.0, 0.0),
            impulse: Vector3::new(0.0, 0.0, 0.0),
            linear_damping: 0.0,
            is_kinematic: false,
            lock_rotation: true,
            lock_axis: Vector3::new(false, false, false),
        }
    }
    
    pub fn kinematic() -> Self {
        Self {
            mass: 0.0,
            inverse_mass: 0.0,
            restitution: 0.0,
            friction: 0.5,
            force: Vector3::new(0.0, 0.0, 0.0),
            impulse: Vector3::new(0.0, 0.0, 0.0),
            linear_damping: 0.0,
            is_kinematic: true,
            lock_rotation: true,
            lock_axis: Vector3::new(false, false, false),
        }
    }
    
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping.clamp(0.0, 1.0);
        self
    }

    pub fn with_kinematic(mut self, is_kinematic: bool) -> Self {
        self.is_kinematic = is_kinematic;
        self
    }

    pub fn lock_x_axis(mut self) -> Self {
        self.lock_axis.x = true;
        self
    }
    
    pub fn lock_y_axis(mut self) -> Self {
        self.lock_axis.y = true;
        self
    }
    
    pub fn apply_force(&mut self, force: Vector3<f32>) {
        if !self.is_kinematic && self.inverse_mass > 0.0 {
            self.force += force;
        }
    }
    
    pub fn apply_impulse(&mut self, impulse: Vector3<f32>) {
        if !self.is_kinematic && self.inverse_mass > 0.0 {
            self.impulse += impulse;
        }
    }
    
    pub fn is_static(&self) -> bool {
        !self.is_kinematic && self.inverse_mass == 0.0
    }
}


//todo this is erm rather scuffed amd simple and not what i really want like as you can see right now it only stores rigidbodys which is of course not really ideally what i want
pub struct PhysicsSystem {
    rigidbodies: ComponentStorage<PhysicsEntity>,
    pub gravity: Vector3<f32>,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            rigidbodies: ComponentStorage::new(),
            gravity: Vector3::new(0.0, -9.81, 0.0), // Default Earth gravity
        }
    }
    
    pub fn with_gravity(mut self, gravity: Vector3<f32>) -> Self {
        self.gravity = gravity;
        self
    }
    
    pub fn add_rigidbody(&mut self, entity_id: u32, rigidbody: PhysicsEntity) {
        self.rigidbodies.insert(entity_id, rigidbody);
    }
    
    pub fn get_rigidbody(&self, entity_id: u32) -> Option<&PhysicsEntity> {
        self.rigidbodies.get(entity_id)
    }
    
    pub fn get_rigidbody_mut(&mut self, entity_id: u32) -> Option<&mut PhysicsEntity> {
        self.rigidbodies.get_mut(entity_id)
    }
    
    pub fn remove_rigidbody(&mut self, entity_id: u32) {
        self.rigidbodies.remove(entity_id);
    }
    
    /// Main physics update - applies forces and integrates velocity
    pub fn update(&mut self, movement_system: &mut MovementSystem, delta_time: f32) {

        //print!("Physics Update Start\n");
        for (entity_id, rigidbody) in self.rigidbodies.iter_mut() {

            //println!("Updating rigidbody for entity {}: {:?}", entity_id, rigidbody);
            if rigidbody.is_static() {
                //print!("Skipping static rigidbody for entity {}\n", entity_id);
                continue;
            }
            
            // Get velocity component (create if doesn't exist for dynamic bodies)
            let velocity = movement_system.get_velocity_mut(*entity_id);//todo mmee eeh mee heeheh jh he like i dont like gettign mut here... 
            if velocity.is_none() {
                print!("No velocity component for entity {}, skipping physics update\n", entity_id);
                continue;
            }
            let velocity = velocity.unwrap();
            
            let current_velocity = velocity.direction * velocity.speed;
            let mut new_velocity = current_velocity;
            
            if !rigidbody.is_kinematic {
                //print!("Applying physics to entity {}\n", entity_id);
                // Apply gravity when the grav is tea..... lowkey doe we need a floor dont we so things dont just fall forever
                let gravity_force = self.gravity * rigidbody.mass;
                rigidbody.apply_force(gravity_force);
                
                // Calculate acceleration from forces (F = ma -> a = F/m) boring doe
                let acceleration = rigidbody.force * rigidbody.inverse_mass;
                
                // Integrate velocity (v = v0 + a*dt)
                new_velocity += acceleration * delta_time;
                
                // Apply impulses (instant velocity changes)
                new_velocity += rigidbody.impulse * rigidbody.inverse_mass;
                
                // Apply linear damping
                new_velocity *= 1.0 - rigidbody.linear_damping;
                
                // Apply axis locks
                if rigidbody.lock_axis.x { new_velocity.x = 0.0; }
                if rigidbody.lock_axis.y { new_velocity.y = 0.0; }
                if rigidbody.lock_axis.z { new_velocity.z = 0.0; }
            }
            
            // Update velocity component
            let speed = new_velocity.magnitude();
            if speed > 0.001 {
                velocity.direction = new_velocity.normalize();
                velocity.speed = speed;
            } else {
                velocity.direction = Vector3::new(0.0, 0.0, 0.0);
                velocity.speed = 0.0;
            }
            
            // Clear force and impulse accumulators
            rigidbody.force = Vector3::new(0.0, 0.0, 0.0);
            rigidbody.impulse = Vector3::new(0.0, 0.0, 0.0);
        }
    }
    
    /// Resolve collision with proper physics (called by collision system)
    pub fn resolve_collision(
        &mut self,
        movement_system: &mut MovementSystem,
        collision: &CollisionEvent,
    ) {
        let rb_a = self.rigidbodies.get(collision.entity_a);
        let rb_b = self.rigidbodies.get(collision.entity_b);
        
        // If neither has a rigidbody, use simple position separation
        if rb_a.is_none() && rb_b.is_none() {
            print!("using simple collision resolution\n");
            self.simple_position_resolution(movement_system, collision);
            return;
        }
        
        // Get rigidbody data
        let (inv_mass_a, rest_a, fric_a, is_static_a) = if let Some(rb) = rb_a {
            (rb.inverse_mass, rb.restitution, rb.friction, rb.is_static())
        } else {
            print!("object A has no rigidbody\n");
            (0.0, 0.0, 0.5, true) // No rigidbody = static
        };
        
        let (inv_mass_b, rest_b, fric_b, is_static_b) = if let Some(rb) = rb_b {
            (rb.inverse_mass, rb.restitution, rb.friction, rb.is_static())
        } else {
            print!("object B has no rigidbody\n");
            (0.0, 0.0, 0.5, true)
        };
        
        // Both static/kinematic - just separate positions
        if inv_mass_a == 0.0 && inv_mass_b == 0.0 {
            print!("both objects static - using simple collision resolution\n");
            self.simple_position_resolution(movement_system, collision);
            return;
        }
        
        // === POSITION CORRECTION (prevent sinking) ===
        let total_inv_mass = inv_mass_a + inv_mass_b;
        if total_inv_mass > 0.0 {
            let correction_percent = 1.0; // How much to correct (0-1)
            let slop = 0.01; // Allowed penetration (prevents jitter)
            
            let correction_magnitude = (collision.penetration - slop).max(0.0) / total_inv_mass * correction_percent;
            let correction = collision.normal * correction_magnitude;
            
            if inv_mass_a > 0.0 {
                if let Some(coords) = movement_system.get_coords_mut(collision.entity_a) {
                    coords.position += correction * inv_mass_a;
                }
            }
            
            if inv_mass_b > 0.0 {
                if let Some(coords) = movement_system.get_coords_mut(collision.entity_b) {
                    coords.position -= correction * inv_mass_b;
                }
            }
        }
        
        // === VELOCITY RESOLUTION (impulse-based) ===
        
        // Get velocities
        let vel_a = movement_system.get_velocity_mut(collision.entity_a)
            .map(|v| v.direction * v.speed)
            .unwrap_or(Vector3::new(0.0, 0.0, 0.0));
            
        let vel_b = movement_system.get_velocity_mut(collision.entity_b)
            .map(|v| v.direction * v.speed)
            .unwrap_or(Vector3::new(0.0, 0.0, 0.0));

        // Relative velocity
        let relative_velocity = vel_a - vel_b;
        let velocity_along_normal = relative_velocity.dot(collision.normal);
        
        // Don't resolve if velocities are separating
        if velocity_along_normal > 0.01 && collision.penetration < 0.01 {
            println!(">>> SKIPPING: Objects separating (vel_along_normal > 0.01)");
            return;
        }
        
        //im using the average restitution here so that uuuh like if the ground is bouncy or if the object is bouncy they will bounce if hit 
        let restitution = (rest_a + rest_b) / 2.0;

        
        // Calculate impulse scalar
        let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / total_inv_mass;
        let impulse = collision.normal * impulse_scalar;

        
        // Apply impulses to BOTH entities
        if inv_mass_a > 0.0 {
            if let Some(vel) = movement_system.get_velocity_mut(collision.entity_a) {
                let velocity_change = impulse * inv_mass_a;
                
                let current_vel = vel.direction * vel.speed;
                let new_vel = current_vel + velocity_change;
                
                let speed = new_vel.magnitude();
                if speed > 0.001 {
                    vel.direction = new_vel.normalize();
                    vel.speed = speed;
                } else {
                    vel.direction = Vector3::new(0.0, 0.0, 0.0);
                    vel.speed = 0.0;
                }
            }
        }
        
        if inv_mass_b > 0.0 {
            if let Some(vel) = movement_system.get_velocity_mut(collision.entity_b) {
                let velocity_change = -impulse * inv_mass_b;  // NEGATIVE impulse for entity B
                let current_vel = vel.direction * vel.speed;
                let new_vel = current_vel + velocity_change;
                
                let speed = new_vel.magnitude();
                if speed > 0.001 {
                    vel.direction = new_vel.normalize();
                    vel.speed = speed;
                } else {
                    vel.direction = Vector3::new(0.0, 0.0, 0.0);
                    vel.speed = 0.0;
                }
            }
        }
        
        // === FRICTION ===
        let friction = (fric_a + fric_b) / 2.0;

        if friction > 0.001 {
            // Recalculate relative velocity after impulse
            let vel_a = movement_system.get_velocity_mut(collision.entity_a)
                .map(|v| v.direction * v.speed)
                .unwrap_or(Vector3::new(0.0, 0.0, 0.0));
                
            let vel_b = movement_system.get_velocity_mut(collision.entity_b)
                .map(|v| v.direction * v.speed)
                .unwrap_or(Vector3::new(0.0, 0.0, 0.0));
            
            let relative_velocity = vel_a - vel_b;
            
            // Get tangent (perpendicular to normal)
            let tangent = relative_velocity - collision.normal * relative_velocity.dot(collision.normal);
            
            if tangent.magnitude() > 0.001 {
                let tangent = tangent.normalize();
                
                // Calculate friction impulse
                let friction_impulse_scalar = -relative_velocity.dot(tangent) / total_inv_mass;
                let friction_impulse = tangent * friction_impulse_scalar * friction;
                
                // Apply friction impulses DIRECTLY to velocity
                if inv_mass_a > 0.0 {
                    if let Some(vel) = movement_system.get_velocity_mut(collision.entity_a) {
                        let velocity_change = friction_impulse * inv_mass_a;
                        let current_vel = vel.direction * vel.speed;
                        let new_vel = current_vel + velocity_change;
                        
                        let speed = new_vel.magnitude();
                        if speed > 0.001 {
                            vel.direction = new_vel.normalize();
                            vel.speed = speed;
                        }
                    }
                }
                
                if inv_mass_b > 0.0 {
                    if let Some(vel) = movement_system.get_velocity_mut(collision.entity_b) {
                        let velocity_change = -friction_impulse * inv_mass_b;
                        let current_vel = vel.direction * vel.speed;
                        let new_vel = current_vel + velocity_change;
                        
                        let speed = new_vel.magnitude();
                        if speed > 0.001 {
                            vel.direction = new_vel.normalize();
                            vel.speed = speed;
                        }
                    }
                }
            }
        }
    }
    
    /// Simple position-based resolution (fallback for objects without rigidbodies)
    fn simple_position_resolution(&self, movement_system: &mut MovementSystem, collision: &CollisionEvent) {
        let separation = collision.normal * (collision.penetration / 2.0);
        
        if let Some(coords_a) = movement_system.get_coords_mut(collision.entity_a) {
            coords_a.position += separation;
        }
        
        if let Some(coords_b) = movement_system.get_coords_mut(collision.entity_b) {
            coords_b.position -= separation;
        }
    }
}