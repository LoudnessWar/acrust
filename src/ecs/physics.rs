use cgmath::{InnerSpace, Quaternion, Rotation3, Vector3, Zero};
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

    // Angular motion
    pub angular_velocity: Vector3<f32>,  // Rotation speed (radians/sec) around each axis
    pub torque: Vector3<f32>,            // Accumulated rotational force
    pub angular_impulse: Vector3<f32>,   // Instant angular velocity changes
    pub angular_damping: f32,            // Rotational drag (0.0 = no damping, 1.0 = full)
    
    // Inertia (resistance to rotation)
    pub inertia_tensor: Vector3<f32>,    // Diagonal inertia tensor (simplified)
    pub inverse_inertia: Vector3<f32>,   // Cached inverse for performance
    
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
            angular_velocity: Vector3::zero(),
            torque: Vector3::zero(),
            angular_impulse: Vector3::zero(),
            angular_damping: 0.05,
            inertia_tensor: Vector3::new(1.0, 1.0, 1.0),
            inverse_inertia: Vector3::new(1.0, 1.0, 1.0),
            is_kinematic: false,
            lock_rotation: false,
            lock_axis: Vector3::new(false, false, false),
        }
    }

    pub fn sphere(mass: f32, radius: f32) -> Self {
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        
        // Moment of inertia for solid sphere: I = (2/5) * m * r²
        let inertia = (2.0 / 5.0) * mass * radius * radius;
        let inertia_tensor = Vector3::new(inertia, inertia, inertia);
        let inverse_inertia = if mass > 0.0 {
            Vector3::new(1.0 / inertia, 1.0 / inertia, 1.0 / inertia)
        } else {
            Vector3::zero()
        };
        
        Self {
            mass,
            inverse_mass,
            restitution: 0.3,
            friction: 0.6,
            force: Vector3::zero(),
            impulse: Vector3::zero(),
            linear_damping: 0.01,
            
            angular_velocity: Vector3::zero(),
            torque: Vector3::zero(),
            angular_impulse: Vector3::zero(),
            angular_damping: 0.05,
            
            inertia_tensor,
            inverse_inertia,
            
            is_kinematic: false,
            lock_rotation: false,
            lock_axis: Vector3::new(false, false, false),
        }
    }

    pub fn box_shape(mass: f32, width: f32, height: f32, depth: f32) -> Self {
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        
        // Moment of inertia for box: I_x = (1/12) * m * (h² + d²), etc.
        let inertia_x = (1.0 / 12.0) * mass * (height * height + depth * depth);
        let inertia_y = (1.0 / 12.0) * mass * (width * width + depth * depth);
        let inertia_z = (1.0 / 12.0) * mass * (width * width + height * height);
        
        let inertia_tensor = Vector3::new(inertia_x, inertia_y, inertia_z);
        let inverse_inertia = if mass > 0.0 {
            Vector3::new(1.0 / inertia_x, 1.0 / inertia_y, 1.0 / inertia_z)
        } else {
            Vector3::zero()
        };
        
        Self {
            mass,
            inverse_mass,
            restitution: 0.4,
            friction: 0.5,
            force: Vector3::zero(),
            impulse: Vector3::zero(),
            linear_damping: 0.01,
            
            angular_velocity: Vector3::zero(),
            torque: Vector3::zero(),
            angular_impulse: Vector3::zero(),
            angular_damping: 0.05,
            
            inertia_tensor,
            inverse_inertia,
            
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
            angular_velocity: Vector3::zero(),
            torque: Vector3::zero(),
            angular_impulse: Vector3::zero(),
            angular_damping: 0.05,
            inertia_tensor: Vector3::zero(),
            inverse_inertia: Vector3::zero(),
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
            angular_velocity: Vector3::zero(),
            torque: Vector3::zero(),
            angular_impulse: Vector3::zero(),
            angular_damping: 0.05,
            inertia_tensor: Vector3::new(1.0, 1.0, 1.0),
            inverse_inertia: Vector3::new(1.0, 1.0, 1.0),
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

    pub fn apply_torque(&mut self, torque: Vector3<f32>) {
        if !self.lock_rotation && !self.is_kinematic {
            self.torque += torque;
        }
    }
    
    pub fn apply_angular_impulse(&mut self, impulse: Vector3<f32>) {
        if !self.lock_rotation && !self.is_kinematic {
            self.angular_impulse += impulse;
        }
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
        for (entity_id, rigidbody) in self.rigidbodies.iter_mut() {
            if rigidbody.is_static() {
                continue;
            }
            
            // === LINEAR PHYSICS ===
            let velocity = movement_system.get_velocity_mut(*entity_id);
            if velocity.is_none() {
                print!("No velocity component for entity {}, skipping physics update\n", entity_id);
                continue;
            }
            let velocity = velocity.unwrap();
            
            let current_velocity = velocity.direction * velocity.speed;
            let mut new_velocity = current_velocity;
            
            if !rigidbody.is_kinematic {
                // Apply gravity
                let gravity_force = self.gravity * rigidbody.mass;
                rigidbody.apply_force(gravity_force);
                
                // Calculate acceleration from forces (F = ma -> a = F/m)
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
                velocity.direction = Vector3::zero();
                velocity.speed = 0.0;
            }
            
            // Clear force and impulse accumulators
            // rigidbody.force = Vector3::zero();
            // rigidbody.impulse = Vector3::zero();
            
            // === ANGULAR PHYSICS (NEW) ===
            if !rigidbody.is_kinematic && !rigidbody.lock_rotation {
                // Apply angular impulses (instant angular velocity changes)
                rigidbody.angular_velocity += Vector3::new(
                    rigidbody.angular_impulse.x * rigidbody.inverse_inertia.x,
                    rigidbody.angular_impulse.y * rigidbody.inverse_inertia.y,
                    rigidbody.angular_impulse.z * rigidbody.inverse_inertia.z,
                );
                rigidbody.angular_impulse = Vector3::zero();
                
                // Apply torques (T = I * α -> α = T / I)
                let angular_acceleration = Vector3::new(
                    rigidbody.torque.x * rigidbody.inverse_inertia.x,
                    rigidbody.torque.y * rigidbody.inverse_inertia.y,
                    rigidbody.torque.z * rigidbody.inverse_inertia.z,
                );
                rigidbody.angular_velocity += angular_acceleration * delta_time;
                rigidbody.torque = Vector3::zero();
                
                // Apply angular damping
                rigidbody.angular_velocity *= 1.0 - rigidbody.angular_damping;
                
                // Integrate rotation (update the actual rotation)
                if rigidbody.angular_velocity.magnitude2() > 0.0001 {
                    if let Some(coords) = movement_system.get_coords_mut(*entity_id) {
                        let angle = rigidbody.angular_velocity.magnitude() * delta_time;
                        if angle > 0.0001 {
                            let axis = rigidbody.angular_velocity.normalize();
                            let rotation_delta = Quaternion::from_axis_angle(axis, cgmath::Rad(angle));
                            coords.set_rotation_from_quaternion(coords.rotation * rotation_delta);
                        }
                    }
                }
            } else if rigidbody.lock_rotation {
                // Clear angular motion if rotation is locked
                rigidbody.angular_velocity = Vector3::zero();
                rigidbody.torque = Vector3::zero();
                rigidbody.angular_impulse = Vector3::zero();
            }

                        // Clear force and impulse accumulators
            rigidbody.force = Vector3::zero();
            rigidbody.impulse = Vector3::zero();
            
        }
    }
    
    /// Resolve collision with proper physics (called by collision system)
    pub fn resolve_collision(
        &mut self,
        movement_system: &mut MovementSystem,
        collision: &CollisionEvent,
    ) {

        println!( ">>> Resolving collision between entities {} and {}", collision.entity_a, collision.entity_b);
        // println!("Collision normal: {:?}, penetration: {}\n", collision.normal, collision.penetration);
        // println!("Collision point: {:?}", collision.collision_point);
        // println!("----------------------------------------");
        
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


        //lol this is lowkey bogus lol
        //todo fix this
        let mut normal = collision.normal;
        normal = normal.normalize();

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
            let correction = normal * correction_magnitude;
            
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
        let velocity_along_normal = relative_velocity.dot(normal);
        

        if velocity_along_normal.abs() < 0.01 && collision.penetration < 0.05 {
        // They're resting - project velocities to be parallel to contact
            if inv_mass_a > 0.0 {
                if let Some(vel) = movement_system.get_velocity_mut(collision.entity_a) {
                    let current_vel = vel.direction * vel.speed;
                    let tangent_vel = current_vel - normal * current_vel.dot(normal);
                    let speed = tangent_vel.magnitude();
                    if speed > 0.001 {
                        vel.direction = tangent_vel.normalize();
                        vel.speed = speed;
                    } else {
                        vel.speed = 0.0;
                    }
                }
            }
        }

        // println!("Vel A: {:?}, Vel B: {:?}", vel_a, vel_b);
        // println!("Relative velocity: {:?}", relative_velocity);
        // println!("Velocity along normal: {}", velocity_along_normal);
        
        //im using the average restitution here so that uuuh like if the ground is bouncy or if the object is bouncy they will bounce if hit 
        let restitution = (rest_a + rest_b) / 2.0;

        
        // Calculate impulse scalar
        let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / total_inv_mass;
        let impulse = normal * impulse_scalar;

        
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

                if vel.speed < 0.01 {
                    vel.speed = 0.0;
                    vel.direction = Vector3::new(0.0, 0.0, 0.0);
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
    println!(">>> FRICTION SECTION");
    
    // Get positions for contact point calculation
    let pos_a = movement_system.get_coords(collision.entity_a)
        .map(|c| c.position)
        .unwrap_or(Vector3::zero());
    let pos_b = movement_system.get_coords(collision.entity_b)
        .map(|c| c.position)
        .unwrap_or(Vector3::zero());
    
    println!("pos_a (A center): {:?}", pos_a);
    println!("pos_b (B center): {:?}", pos_b);
    
    // Recalculate velocities after normal impulse
    let vel_a = movement_system.get_velocity_mut(collision.entity_a)
        .map(|v| v.direction * v.speed)
        .unwrap_or(Vector3::zero());
        
    let vel_b = movement_system.get_velocity_mut(collision.entity_b)
        .map(|v| v.direction * v.speed)
        .unwrap_or(Vector3::zero());
    
    println!("Linear vel_a: {:?}, vel_b: {:?}", vel_a, vel_b);
    
    // Get contact point relative to centers (r vectors)
    let r_a = collision.collision_point - pos_a;
    let r_b = collision.collision_point - pos_b;
    
    println!("Contact point: {:?}", collision.collision_point);
    println!("r_a (contact relative to A): {:?}", r_a);
    println!("r_b (contact relative to B): {:?}", r_b);
    
    // Get angular velocities and inertia from rigidbodies
    // ONLY for non-static objects
    let (angular_vel_a, inv_inertia_a, can_rotate_a) = 
        if inv_mass_a > 0.0 {
            self.rigidbodies.get(collision.entity_a)
                .map(|rb| (rb.angular_velocity, rb.inverse_inertia, !rb.lock_rotation))
                .unwrap_or((Vector3::zero(), Vector3::zero(), false))
        } else {
            (Vector3::zero(), Vector3::zero(), false)
        };
    
    let (angular_vel_b, inv_inertia_b, can_rotate_b) = 
        if inv_mass_b > 0.0 {
            self.rigidbodies.get(collision.entity_b)
                .map(|rb| (rb.angular_velocity, rb.inverse_inertia, !rb.lock_rotation))
                .unwrap_or((Vector3::zero(), Vector3::zero(), false))
        } else {
            (Vector3::zero(), Vector3::zero(), false)
        };
    
    println!("Angular vel_a: {:?}, inv_inertia_a: {:?}, can_rotate: {}", 
             angular_vel_a, inv_inertia_a, can_rotate_a);
    println!("Angular vel_b: {:?}, inv_inertia_b: {:?}, can_rotate: {}", 
             angular_vel_b, inv_inertia_b, can_rotate_b);
    
    // Calculate velocity at contact point (linear + rotational contribution)
    // ONLY include angular contribution if object can rotate
    let contact_vel_a = if can_rotate_a {
        vel_a + angular_vel_a.cross(r_a)
    } else {
        vel_a
    };
    
    let contact_vel_b = if can_rotate_b {
        vel_b + angular_vel_b.cross(r_b)
    } else {
        vel_b
    };
    
    println!("Contact vel_a: {:?}", contact_vel_a);
    println!("Contact vel_b: {:?}", contact_vel_b);
    
    let relative_velocity = contact_vel_a - contact_vel_b;
    println!("Relative velocity at contact: {:?}", relative_velocity);
    
    // Get tangent (perpendicular to normal, in the plane of contact)
    let tangent = relative_velocity - normal * relative_velocity.dot(normal);
    println!("Tangent (before normalize): {:?}, magnitude: {}", tangent, tangent.magnitude());
    
    if tangent.magnitude() > 0.001 {
        let tangent = tangent.normalize();
        println!("Tangent (normalized): {:?}", tangent);
        println!("Normal: {:?}", normal);
        
        // Calculate effective mass for friction
        let mut effective_mass = inv_mass_a + inv_mass_b;
        
        // Add angular terms ONLY for objects that can rotate
        if can_rotate_a {
            let r_a_cross_t = r_a.cross(tangent);
            let angular_term_a = r_a_cross_t.dot(Vector3::new(
                inv_inertia_a.x * r_a_cross_t.x,
                inv_inertia_a.y * r_a_cross_t.y,
                inv_inertia_a.z * r_a_cross_t.z,
            ));
            println!("r_a × tangent: {:?}, angular_term_a: {}", r_a_cross_t, angular_term_a);
            effective_mass += angular_term_a;
        }
        
        if can_rotate_b {
            let r_b_cross_t = r_b.cross(tangent);
            let angular_term_b = r_b_cross_t.dot(Vector3::new(
                inv_inertia_b.x * r_b_cross_t.x,
                inv_inertia_b.y * r_b_cross_t.y,
                inv_inertia_b.z * r_b_cross_t.z,
            ));
            println!("r_b × tangent: {:?}, angular_term_b: {}", r_b_cross_t, angular_term_b);
            effective_mass += angular_term_b;
        }
        
        println!("Effective mass: {}", effective_mass);
        
        if effective_mass > 0.0001 {
            // Calculate friction impulse magnitude
            let friction_impulse_scalar = -relative_velocity.dot(tangent) / effective_mass;
            let friction_impulse = tangent * friction_impulse_scalar * friction;
            
            println!("Friction impulse scalar (before friction): {}", friction_impulse_scalar);
            println!("Final friction impulse: {:?}, magnitude: {}", friction_impulse, friction_impulse.magnitude());
            
            // Apply LINEAR friction impulses
            if inv_mass_a > 0.0 {
                if let Some(vel) = movement_system.get_velocity_mut(collision.entity_a) {
                    let velocity_change = friction_impulse * inv_mass_a;
                    println!("Linear velocity change for A: {:?}", velocity_change);
                    
                    let current_vel = vel.direction * vel.speed;
                    let new_vel = current_vel + velocity_change;
                    
                    let speed = new_vel.magnitude();
                    if speed > 0.001 {
                        vel.direction = new_vel.normalize();
                        vel.speed = speed;
                    } else {
                        vel.direction = Vector3::zero();
                        vel.speed = 0.0;
                    }
                }
            }
            
            if inv_mass_b > 0.0 {
                if let Some(vel) = movement_system.get_velocity_mut(collision.entity_b) {
                    let velocity_change = -friction_impulse * inv_mass_b;
                    println!("Linear velocity change for B: {:?}", velocity_change);
                    
                    let current_vel = vel.direction * vel.speed;
                    let new_vel = current_vel + velocity_change;
                    
                    let speed = new_vel.magnitude();
                    if speed > 0.001 {
                        vel.direction = new_vel.normalize();
                        vel.speed = speed;
                    } else {
                        vel.direction = Vector3::zero();
                        vel.speed = 0.0;
                    }
                }
            }
            
            // Apply ANGULAR friction impulses (THIS CREATES THE ROLLING!)
            // ONLY apply to objects that can rotate
            if can_rotate_a {
                if let Some(rb_a) = self.rigidbodies.get_mut(collision.entity_a) {
                    let angular_impulse = r_a.cross(friction_impulse);
                    println!("Angular impulse for A (r × F): {:?}", angular_impulse);
                    
                    let angular_change = Vector3::new(
                        angular_impulse.x * inv_inertia_a.x,
                        angular_impulse.y * inv_inertia_a.y,
                        angular_impulse.z * inv_inertia_a.z,
                    );
                    println!("Angular velocity change for A: {:?}", angular_change);
                    println!("Old angular velocity A: {:?}", rb_a.angular_velocity);
                    
                    rb_a.angular_velocity += angular_change;
                    
                    println!("New angular velocity A: {:?}", rb_a.angular_velocity);
                }
            }
            
            if can_rotate_b {
                if let Some(rb_b) = self.rigidbodies.get_mut(collision.entity_b) {
                    let angular_impulse = r_b.cross(-friction_impulse);
                    println!("Angular impulse for B (r × -F): {:?}", angular_impulse);
                    
                    let angular_change = Vector3::new(
                        angular_impulse.x * inv_inertia_b.x,
                        angular_impulse.y * inv_inertia_b.y,
                        angular_impulse.z * inv_inertia_b.z,
                    );
                    println!("Angular velocity change for B: {:?}", angular_change);
                    println!("Old angular velocity B: {:?}", rb_b.angular_velocity);
                    
                    rb_b.angular_velocity += angular_change;
                    
                    println!("New angular velocity B: {:?}", rb_b.angular_velocity);
                }
            }
        } else {
            println!("Effective mass too small, skipping friction");
        }
    } else {
        println!("Tangent magnitude too small ({}), no sliding motion", tangent.magnitude());
    }
    
    println!(">>> END FRICTION SECTION\n");
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