use std::collections::HashMap;
use cgmath::{Vector2, Vector3, InnerSpace};
use crate::model::transform::WorldCoords;
use super::components::Velocity;
use super::world::MovementSystem;

// Collision component types
#[derive(Debug, Clone)]
pub enum CollisionShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Sphere { radius: f32 },
    Box { width: f32, height: f32, depth: f32 },
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: CollisionShape,
    pub is_trigger: bool, // If true, doesn't prevent movement but still fires events
    pub layer: u32, // For collision filtering
}

impl Collider {
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: CollisionShape::Circle { radius },
            is_trigger: false,
            layer: 0,
        }
    }
    
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self {
            shape: CollisionShape::Rectangle { width, height },
            is_trigger: false,
            layer: 0,
        }
    }
    
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: CollisionShape::Sphere { radius },
            is_trigger: false,
            layer: 0,
        }
    }
    
    pub fn bounding_box(width: f32, height: f32, depth: f32) -> Self {
        Self {
            shape: CollisionShape::Box { width, height, depth },
            is_trigger: false,
            layer: 0,
        }
    }
    
    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self
    }
    
    pub fn with_layer(mut self, layer: u32) -> Self {
        self.layer = layer;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: u32,
    pub entity_b: u32,
    pub collision_point: Vector3<f32>,
    pub normal: Vector3<f32>, // Direction to separate entity_a from entity_b
    pub penetration: f32,
}

pub struct CollisionSystem {
    colliders: HashMap<u32, Collider>,
    collision_events: Vec<CollisionEvent>,
    // Collision matrix - which layers can collide with which
    collision_matrix: HashMap<(u32, u32), bool>,
}

impl CollisionSystem {
    pub fn new() -> Self {
        Self {
            colliders: HashMap::new(),
            collision_events: Vec::new(),
            collision_matrix: HashMap::new(),
        }
    }
    
    pub fn add_collider(&mut self, entity_id: u32, collider: Collider) {
        self.colliders.insert(entity_id, collider);
    }
    
    pub fn remove_collider(&mut self, entity_id: u32) {
        self.colliders.remove(&entity_id);
    }
    
    pub fn get_collider(&self, entity_id: u32) -> Option<&Collider> {
        self.colliders.get(&entity_id)
    }
    
    pub fn get_collider_mut(&mut self, entity_id: u32) -> Option<&mut Collider> {
        self.colliders.get_mut(&entity_id)
    }
    
    // Set which layers can collide with each other
    pub fn set_collision_layers(&mut self, layer_a: u32, layer_b: u32, can_collide: bool) {
        self.collision_matrix.insert((layer_a, layer_b), can_collide);
        self.collision_matrix.insert((layer_b, layer_a), can_collide);
    }
    
    fn can_collide(&self, layer_a: u32, layer_b: u32) -> bool {
        self.collision_matrix.get(&(layer_a, layer_b)).copied().unwrap_or(true)
    }
    
    pub fn update(&mut self, movement_system: &mut MovementSystem, delta_time: f32) {
        self.collision_events.clear();
        
        // Get all entities with both colliders and positions
        let mut entities_with_collision: Vec<(u32, Vector3<f32>, &Collider)> = Vec::new();
        
        for (entity_id, collider) in &self.colliders {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                entities_with_collision.push((*entity_id, coords.position, collider));
            }
        }
        
        // Check all pairs for collision
        for i in 0..entities_with_collision.len() {
            for j in (i + 1)..entities_with_collision.len() {
                let (entity_a, pos_a, collider_a) = entities_with_collision[i];
                let (entity_b, pos_b, collider_b) = entities_with_collision[j];
                
                // Check if these layers can collide
                if !self.can_collide(collider_a.layer, collider_b.layer) {
                    continue;
                }
                
                if let Some(collision) = self.check_collision(
                    entity_a, pos_a, collider_a,
                    entity_b, pos_b, collider_b
                ) {
                    self.collision_events.push(collision.clone());
                    
                    // Only resolve collision if neither is a trigger
                    if !collider_a.is_trigger && !collider_b.is_trigger {
                        self.resolve_collision(movement_system, &collision);
                    }
                }
            }
        }
    }
    
    //really im going to be honest, i have like no memory of how to do collision so i hope that this is correct
    fn check_collision(
        &self,
        entity_a: u32, pos_a: Vector3<f32>, collider_a: &Collider,
        entity_b: u32, pos_b: Vector3<f32>, collider_b: &Collider
    ) -> Option<CollisionEvent> {
        match (&collider_a.shape, &collider_b.shape) {
            // Circle vs Circle (2D)
            (CollisionShape::Circle { radius: r1 }, CollisionShape::Circle { radius: r2 }) => {
                let distance = (Vector2::new(pos_a.x, pos_a.y) - Vector2::new(pos_b.x, pos_b.y)).magnitude();
                let combined_radius = r1 + r2;
                
                if distance < combined_radius {
                    let normal = if distance > 0.0 {
                        (Vector2::new(pos_a.x, pos_a.y) - Vector2::new(pos_b.x, pos_b.y)).normalize()
                    } else {
                        Vector2::new(1.0, 0.0) // Default separation direction
                    };
                    
                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(
                            pos_b.x + normal.x * r2,
                            pos_b.y + normal.y * r2,
                            pos_a.z
                        ),
                        normal: Vector3::new(normal.x, normal.y, 0.0),
                        penetration: combined_radius - distance,
                    })
                } else {
                    None
                }
            },
            
            // Sphere vs Sphere (3D)
            (CollisionShape::Sphere { radius: r1 }, CollisionShape::Sphere { radius: r2 }) => {
                let distance = (pos_a - pos_b).magnitude();
                let combined_radius = r1 + r2;
                
                if distance < combined_radius {
                    let normal = if distance > 0.0 {
                        (pos_a - pos_b).normalize()
                    } else {
                        Vector3::new(1.0, 0.0, 0.0) // Default separation direction
                    };
                    
                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: pos_b + normal * *r2,//just deref here because lazy and like its a float maybe todo fix later to not be ass
                        normal,
                        penetration: combined_radius - distance,
                    })
                } else {
                    None
                }
            },
            
            // Rectangle vs Rectangle (2D AABB)
            (CollisionShape::Rectangle { width: w1, height: h1 }, 
             CollisionShape::Rectangle { width: w2, height: h2 }) => {
                let half_w1 = w1 / 2.0;
                let half_h1 = h1 / 2.0;
                let half_w2 = w2 / 2.0;
                let half_h2 = h2 / 2.0;
                
                let dx = pos_a.x - pos_b.x;
                let dy = pos_a.y - pos_b.y;
                
                let overlap_x = (half_w1 + half_w2) - dx.abs();
                let overlap_y = (half_h1 + half_h2) - dy.abs();
                
                if overlap_x > 0.0 && overlap_y > 0.0 {
                    // Choose the axis with smallest overlap for separation
                    let (normal, penetration) = if overlap_x < overlap_y {
                        (Vector3::new(if dx > 0.0 { 1.0 } else { -1.0 }, 0.0, 0.0), overlap_x)
                    } else {
                        (Vector3::new(0.0, if dy > 0.0 { 1.0 } else { -1.0 }, 0.0), overlap_y)
                    };
                    
                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(
                            pos_a.x - normal.x * penetration / 2.0,
                            pos_a.y - normal.y * penetration / 2.0,
                            pos_a.z
                        ),
                        normal,
                        penetration,
                    })
                } else {
                    None
                }
            },
            
            // Box vs Box (3D AABB)
            (CollisionShape::Box { width: w1, height: h1, depth: d1 }, 
             CollisionShape::Box { width: w2, height: h2, depth: d2 }) => {
                let half_w1 = w1 / 2.0;
                let half_h1 = h1 / 2.0;
                let half_d1 = d1 / 2.0;
                let half_w2 = w2 / 2.0;
                let half_h2 = h2 / 2.0;
                let half_d2 = d2 / 2.0;
                
                let dx = pos_a.x - pos_b.x;
                let dy = pos_a.y - pos_b.y;
                let dz = pos_a.z - pos_b.z;
                
                let overlap_x = (half_w1 + half_w2) - dx.abs();
                let overlap_y = (half_h1 + half_h2) - dy.abs();
                let overlap_z = (half_d1 + half_d2) - dz.abs();
                
                if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {
                    // Choose the axis with smallest overlap for separation
                    let (normal, penetration) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
                        (Vector3::new(if dx > 0.0 { 1.0 } else { -1.0 }, 0.0, 0.0), overlap_x)
                    } else if overlap_y <= overlap_z {
                        (Vector3::new(0.0, if dy > 0.0 { 1.0 } else { -1.0 }, 0.0), overlap_y)
                    } else {
                        (Vector3::new(0.0, 0.0, if dz > 0.0 { 1.0 } else { -1.0 }), overlap_z)
                    };
                    
                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(
                            pos_a.x - normal.x * penetration / 2.0,
                            pos_a.y - normal.y * penetration / 2.0,
                            pos_a.z - normal.z * penetration / 2.0
                        ),
                        normal,
                        penetration,
                    })
                } else {
                    None
                }
            },
            
            // Mixed shape collisions could be added here
            _ => None, // Unsupported collision pair
        }
    }
    
    fn resolve_collision(&self, movement_system: &mut MovementSystem, collision: &CollisionEvent) {
        // Simple position-based resolution
        let separation = collision.normal * (collision.penetration / 2.0);
        
        // Move entity A away from B
        if let Some(coords_a) = movement_system.get_coords_mut(collision.entity_a) {
            coords_a.position += separation;
        }
        
        // Move entity B away from A
        if let Some(coords_b) = movement_system.get_coords_mut(collision.entity_b) {
            coords_b.position -= separation;
        }
    }
    
    // Get collision events from the last update
    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }
    
    // Check if a specific entity collided this frame
    pub fn entity_collided_with(&self, entity_id: u32) -> Vec<u32> {
        self.collision_events
            .iter()
            .filter_map(|event| {
                if event.entity_a == entity_id {
                    Some(event.entity_b)
                } else if event.entity_b == entity_id {
                    Some(event.entity_a)
                } else {
                    None
                }
            })
            .collect()
    }
    
    // Check if two specific entities collided
    pub fn entities_collided(&self, entity_a: u32, entity_b: u32) -> bool {
        self.collision_events.iter().any(|event| {
            (event.entity_a == entity_a && event.entity_b == entity_b) ||
            (event.entity_a == entity_b && event.entity_b == entity_a)
        })
    }
}