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
    let rb_a = self.rigidbodies.get(collision.entity_a);
    let rb_b = self.rigidbodies.get(collision.entity_b);
    
    if rb_a.is_none() && rb_b.is_none() {
        self.simple_position_resolution(movement_system, collision);
        return;
    }

    let (inv_mass_a, rest_a, fric_a, is_static_a) = if let Some(rb) = rb_a {
        (rb.inverse_mass, rb.restitution, rb.friction, rb.is_static())
    } else {
        (0.0, 0.0, 0.5, true)
    };
    
    let (inv_mass_b, rest_b, fric_b, is_static_b) = if let Some(rb) = rb_b {
        (rb.inverse_mass, rb.restitution, rb.friction, rb.is_static())
    } else {
        (0.0, 0.0, 0.5, true)
    };

    let mut normal = collision.normal.normalize();

    if inv_mass_a == 0.0 && inv_mass_b == 0.0 {
        self.simple_position_resolution(movement_system, collision);
        return;
    }
    
    // === POSITION CORRECTION ===
    let total_inv_mass = inv_mass_a + inv_mass_b;
    if total_inv_mass > 0.0 {
        let correction_percent = 0.8;
        let slop = 0.01;
        
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
    
    // === VELOCITY RESOLUTION ===
    let vel_a = movement_system.get_velocity_mut(collision.entity_a)
        .map(|v| v.direction * v.speed)
        .unwrap_or(Vector3::zero());
        
    let vel_b = movement_system.get_velocity_mut(collision.entity_b)
        .map(|v| v.direction * v.speed)
        .unwrap_or(Vector3::zero());

    let relative_velocity = vel_a - vel_b;
    let velocity_along_normal = relative_velocity.dot(normal);
    
    // Don't resolve if objects are separating
    if velocity_along_normal > 0.0 {
        return;
    }
    
    let restitution = (rest_a + rest_b) / 2.0;
    let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / total_inv_mass;
    let impulse = normal * impulse_scalar;
    
    // Apply normal impulse
    if inv_mass_a > 0.0 {
        if let Some(vel) = movement_system.get_velocity_mut(collision.entity_a) {
            let new_vel = vel.direction * vel.speed + impulse * inv_mass_a;
            let speed = new_vel.magnitude();
            if speed > 0.001 {
                vel.direction = new_vel.normalize();
                vel.speed = speed;
            } else {
                vel.speed = 0.0;
                vel.direction = Vector3::zero();
            }
        }
    }
    
    if inv_mass_b > 0.0 {
        if let Some(vel) = movement_system.get_velocity_mut(collision.entity_b) {
            let new_vel = vel.direction * vel.speed - impulse * inv_mass_b;
            let speed = new_vel.magnitude();
            if speed > 0.001 {
                vel.direction = new_vel.normalize();
                vel.speed = speed;
            } else {
                vel.speed = 0.0;
                vel.direction = Vector3::zero();
            }
        }
    }
    
    // === FRICTION (SIMPLIFIED FOR ROLLING) ===
    let friction = (fric_a + fric_b) / 2.0;
    
    if friction < 0.001 {
        return;
    }
    
    // Get updated velocities after normal impulse
    let vel_a = movement_system.get_velocity_mut(collision.entity_a)
        .map(|v| v.direction * v.speed)
        .unwrap_or(Vector3::zero());
        
    let vel_b = movement_system.get_velocity_mut(collision.entity_b)
        .map(|v| v.direction * v.speed)
        .unwrap_or(Vector3::zero());
    
    // Get positions AT THE TIME OF COLLISION
    // CRITICAL: The collision_point was calculated using old positions
    // We need to recalculate r vectors based on CURRENT positions
    let pos_a = movement_system.get_coords(collision.entity_a)
        .map(|c| c.position)
        .unwrap_or(Vector3::zero());
    let pos_b = movement_system.get_coords(collision.entity_b)
        .map(|c| c.position)
        .unwrap_or(Vector3::zero());
    
    // OPTION 1: Recalculate contact point based on current positions
    // Assume entity B is the sphere (you should check this properly)
    // For now, let's use whichever gives us a reasonable radius
    let r_a = collision.collision_point - pos_a;
    let r_b = collision.collision_point - pos_b;
    
    // Determine which is the sphere based on which r vector is closer to a reasonable radius
    let (sphere_entity, sphere_pos, sphere_r, obb_entity, obb_pos, obb_r) = 
        if r_b.magnitude() < 5.0 && r_b.magnitude() > 1.0 {
            // B is probably the sphere
            (collision.entity_b, pos_b, r_b, collision.entity_a, pos_a, r_a)
        } else if r_a.magnitude() < 5.0 && r_a.magnitude() > 1.0 {
            // A is probably the sphere
            (collision.entity_a, pos_a, r_a, collision.entity_b, pos_b, r_b)
        } else {
            // Can't determine, use as-is
            (collision.entity_a, pos_a, r_a, collision.entity_b, pos_b, r_b)
        };
    
    // Recalculate contact point on sphere surface using CURRENT position
    let sphere_radius = sphere_r.magnitude();
    let contact_point_corrected = sphere_pos - normal * sphere_radius;
    let r_sphere = contact_point_corrected - sphere_pos;
    let r_obb = contact_point_corrected - obb_pos;
    
    println!("DEBUG: pos_a: {:?}, pos_b: {:?}", pos_a, pos_b);
    println!("DEBUG: old collision_point: {:?}", collision.collision_point);
    println!("DEBUG: corrected contact_point: {:?}", contact_point_corrected);
    println!("DEBUG: sphere_pos: {:?}, sphere_radius: {}", sphere_pos, sphere_radius);
    println!("DEBUG: r_sphere magnitude: {}", r_sphere.magnitude());
    
    // Use corrected values
    let (r_a_final, r_b_final) = if sphere_entity == collision.entity_a {
        (r_sphere, r_obb)
    } else {
        (r_obb, r_sphere)
    };
    
    // Get angular data - use corrected entity IDs
    let (angular_vel_a, inv_inertia_a, can_rotate_a, is_sphere_a) = 
        if inv_mass_a > 0.0 {
            self.rigidbodies.get(collision.entity_a)
                .map(|rb| {
                    let is_sphere = sphere_entity == collision.entity_a;
                    (rb.angular_velocity, rb.inverse_inertia, !rb.lock_rotation, is_sphere)
                })
                .unwrap_or((Vector3::zero(), Vector3::zero(), false, false))
        } else {
            (Vector3::zero(), Vector3::zero(), false, false)
        };
    
    let (angular_vel_b, inv_inertia_b, can_rotate_b, is_sphere_b) = 
        if inv_mass_b > 0.0 {
            self.rigidbodies.get(collision.entity_b)
                .map(|rb| {
                    let is_sphere = sphere_entity == collision.entity_b;
                    (rb.angular_velocity, rb.inverse_inertia, !rb.lock_rotation, is_sphere)
                })
                .unwrap_or((Vector3::zero(), Vector3::zero(), false, false))
        } else {
            (Vector3::zero(), Vector3::zero(), false, false)
        };
    
    // Use corrected r vectors
    let r_a = r_a_final;
    let r_b = r_b_final;
    
    // Calculate velocity at contact point
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
    
    let relative_contact_vel = contact_vel_a - contact_vel_b;
    
    // Get tangent (perpendicular to normal)
    let tangent_vel = relative_contact_vel - normal * relative_contact_vel.dot(normal);
    
    if tangent_vel.magnitude() < 0.001 {
        return; // No sliding
    }
    
    let tangent = tangent_vel.normalize();
    
    // SIMPLIFIED: For sphere on plane, use basic friction model
    // The friction impulse opposes sliding at the contact point
    let friction_impulse_magnitude = -relative_contact_vel.dot(tangent);
    
    // Clamp to Coulomb friction limit
    let max_friction = impulse_scalar.abs() * friction;
    let friction_impulse_magnitude = friction_impulse_magnitude.clamp(-max_friction, max_friction);
    
    let friction_impulse = tangent * friction_impulse_magnitude;
    
    println!("=== FRICTION DEBUG ===");
    println!("r_a: {:?}, magnitude: {}", r_a, r_a.magnitude());
    println!("r_b: {:?}, magnitude: {}", r_b, r_b.magnitude());
    println!("Contact velocity A: {:?}", contact_vel_a);
    println!("Relative contact velocity: {:?}", relative_contact_vel);
    println!("Tangent: {:?}", tangent);
    println!("Friction impulse: {:?}", friction_impulse);
    println!("r_a: {:?}, magnitude: {}", r_a, r_a.magnitude());
    
    // Apply linear friction
    if inv_mass_a > 0.0 {
        if let Some(vel) = movement_system.get_velocity_mut(collision.entity_a) {
            let new_vel = vel.direction * vel.speed + friction_impulse * inv_mass_a;
            let speed = new_vel.magnitude();
            if speed > 0.001 {
                vel.direction = new_vel.normalize();
                vel.speed = speed;
            } else {
                vel.speed = 0.0;
                vel.direction = Vector3::zero();
            }
            println!("New linear vel A: {:?}", vel.direction * vel.speed);
        }
    }
    
    if inv_mass_b > 0.0 {
        if let Some(vel) = movement_system.get_velocity_mut(collision.entity_b) {
            let new_vel = vel.direction * vel.speed - friction_impulse * inv_mass_b;
            let speed = new_vel.magnitude();
            if speed > 0.001 {
                vel.direction = new_vel.normalize();
                vel.speed = speed;
            } else {
                vel.speed = 0.0;
                vel.direction = Vector3::zero();
            }
        }
    }
    
    // Apply angular friction (creates rolling)
    if can_rotate_a {
        if let Some(rb_a) = self.rigidbodies.get_mut(collision.entity_a) {
            let torque = r_a.cross(friction_impulse);
            let angular_change = Vector3::new(
                torque.x * inv_inertia_a.x,
                torque.y * inv_inertia_a.y,
                torque.z * inv_inertia_a.z,
            );
            
            println!("Torque: {:?}", torque);
            println!("Angular change: {:?}", angular_change);
            println!("Old angular vel: {:?}", rb_a.angular_velocity);
            
            rb_a.angular_velocity += angular_change;
            
            println!("New angular vel: {:?}", rb_a.angular_velocity);
            
            // CRITICAL: For proper rolling, check the rolling condition
            // For a sphere: v = ω × r, so ω = r × v / r²
            if is_sphere_a {
                let sphere_r_mag = r_a.magnitude();
                let expected_angular_vel = r_a.cross(vel_a) / (sphere_r_mag * sphere_r_mag);
                println!("Expected angular vel for rolling: {:?}", expected_angular_vel);
            }
        }
    }
    
    if can_rotate_b {
        if let Some(rb_b) = self.rigidbodies.get_mut(collision.entity_b) {
            let torque = r_b.cross(-friction_impulse);
            let angular_change = Vector3::new(
                torque.x * inv_inertia_b.x,
                torque.y * inv_inertia_b.y,
                torque.z * inv_inertia_b.z,
            );
            rb_b.angular_velocity += angular_change;
        }
    }
    
    println!("=== END FRICTION ===\n");
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