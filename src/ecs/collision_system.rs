use std::collections::HashMap;
use cgmath::{InnerSpace, Quaternion, Rotation, Vector2, Vector3, Zero};
use crate::ecs::physics::PhysicsSystem;
use crate::graphics::gl_wrapper::ShaderProgram;
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
    OBB {half_extents: Vector3<f32>, rotation: Quaternion<f32>}, // Oriented Bounding Box the half_extents is just like how far the wall of the box is from the center
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: CollisionShape,
    pub is_trigger: bool, // If true, doesn't prevent movement but still fires events
    pub layer: u32, // For collision filtering
    pub offset: Vector3<f32>,
}

impl Collider {
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: CollisionShape::Circle { radius },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        }
    }
    
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self {
            shape: CollisionShape::Rectangle { width, height },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        }
    }
    
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: CollisionShape::Sphere { radius },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        }
    }
    
    pub fn bounding_box(width: f32, height: f32, depth: f32) -> Self {
        Self {
            shape: CollisionShape::Box { width, height, depth },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn obb(half_extents: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self {
            shape: CollisionShape::OBB {half_extents, rotation },
            is_trigger: false,
            layer: 0,
            offset: Vector3::new(0.0, 0.0, 0.0),
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

    pub fn with_offset(mut self, offset: Vector3<f32>) -> Self {
        self.offset = offset;
        self
    }

     pub fn as_obb(&self) -> Option<(&Vector3<f32>, &Quaternion<f32>)> {
        match &self.shape {
            CollisionShape::OBB{half_extents, rotation} => Some((half_extents, rotation)),
            _ => None,
        }
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
    collision_shader: Option<ShaderProgram>,//todo here do I want to just use an ID and use SHADER_MANAGER or do I want to not do that... and have it store its self
    //i wonder if there is a way i could structure shader manager so that it is bascially all static functions that dont use an object but all mutate a single list of shader programs or just
    //shader program ideas
    //todo 
    //todo bc that might be really convientinet..... that might just be ecs though
}

impl CollisionSystem {
    pub fn new() -> Self {
        Self {
            colliders: HashMap::new(),
            collision_events: Vec::new(),
            collision_matrix: HashMap::new(),
            collision_shader: None,
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

    pub fn init_collision_debug(&mut self){
        let mut coll = ShaderProgram::new("shaders/vertex_debug.glsl", "shaders/fragment_shader.glsl");
        coll.create_uniforms(vec![
            "model",
            "view",
            "projection",
        ]);
        self.collision_shader = Some(coll);
    }
    


    //https://research.ncl.ac.uk/game/mastersdegree/gametechnologies/previousinformation/physics4collisiondetection/2017%20Tutorial%204%20-%20Collision%20Detection.pdf
    //Ok below I am goign to talk ig about SAT or like full name Seperate Axis Theorem. The idea behind it is that two convex objects do not collide if there exists a line(in 2d) or a plane(in 3d) that
    // passes betweeen the two objects without intersecing them. IE a lot simpler if you can draw a line that spereates them.
    //THE REASON I NEED TO USE THIS IS FOR BOX ES AND rectangles specifically oriented bounding boxes

    //ok but then the next question is how do I detemine if this line or plane exists.
    //it actually pretty intuitive imo. ok lets start like imagining in 2d space
    //right imagine a rectangle and a triangle or some simple polygon.

    //they are places onto a coordinate plane or whatever right at angles. ok now first we are going to ommit a step right this wont work probably first time but you will undserstand why you need to do this step if we do it like this
    //right first off, our goal here is simple it is to see if on the x or y axis there is a gap between the two shapes. Right so we take the biggest and smallest x and y vales of our two shapes then we see if they fall into the range
    //of each other. if they overlap on both the x and the y that means that they are colliding. if there is a gap that means they are not ok. that seems like it works what the problem. 

    //the issue is is that the axis of the shape is not aligned with the world axis(our grid). imagine a diamnond or a roatated square. Another shap on a normal coordinate system will apprear to be colliding if we just looked at the x and y if it has a point that
    //falls into one of the trianlges that form if you turn a diamond into a square by drawlng triangles to fill in its corners.

    //now ill be honset i am having a hard time wrapping my head around why we need to do the cross products of the edges... but basically i think if you use faces right
    //you dont get all the possible like ways to go about it ie like if something is angled you dont capture every possible like way to go about each face basically, so you could a project every angle of the face which uuh doesnt really work or find orthangonals from the edges
    //which ig is the same as find the normals in the 2d plane

    pub fn check_obb_collision(
        obb_a: &Collider,
        center_a: Vector3<f32>,
        obb_b: &Collider,
        center_b: Vector3<f32>,
    ) -> Option<(Vector3<f32>, f32)> {

    let (obb_data_a, obb_data_b) = match (&obb_a.shape, &obb_b.shape) {
        (CollisionShape::OBB { half_extents: half_a, rotation: rot_a }, 
         CollisionShape::OBB { half_extents: half_b, rotation: rot_b }) => {
            ((half_a, rot_a), (half_b, rot_b))
        },
        _ => return None,
    };
    

        let axes_a = Self::get_axes(obb_data_a.1);
        let axes_b = Self::get_axes( obb_data_b.1);
        
        let mut min_penetration = f32::MAX;
        let mut collision_normal = Vector3::zero();
        
        let mut test_axes = Vec::with_capacity(15);
        test_axes.extend_from_slice(&axes_a);
        test_axes.extend_from_slice(&axes_b);
        
        for i in 0..3 {
            for j in 0..3 {
                let axis = axes_a[i].cross(axes_b[j]);
                if axis.magnitude2() > 0.0001 {
                    test_axes.push(axis.normalize());
                }
            }
        }
        
        for axis in test_axes {
            let (min_a, max_a) = Self::project_obb_onto_axis(obb_data_a, center_a, axis);
            let (min_b, max_b) = Self::project_obb_onto_axis(obb_data_b, center_b, axis);
            
            match Self::ranges_overlap(min_a, max_a, min_b, max_b) {
                None => return None,
                Some(penetration) => {
                    if penetration < min_penetration {
                        min_penetration = penetration;
                        collision_normal = axis;
                        
                        let center_diff = center_b - center_a;
                        if collision_normal.dot(center_diff) < 0.0 {
                            collision_normal = -collision_normal;
                        }
                    }
                }
            }
        }
        
        Some((collision_normal, min_penetration))
    }

    fn ranges_overlap(min1: f32, max1: f32, min2: f32, max2: f32) -> Option<f32> {
        if max1 < min2 || max2 < min1 {
            None
        } else {
            let overlap = (max1 - min2).min(max2 - min1);
            Some(overlap)
        }
    }

    fn project_obb_onto_axis(data: (&Vector3<f32>, &Quaternion<f32>), center: Vector3<f32>, axis: Vector3<f32>) -> (f32, f32) {
        let corners = Self::get_corners(data, &center);
        let mut min = axis.dot(corners[0]);
        let mut max = min;
        
        for i in 1..8 {
            let projection = axis.dot(corners[i]);
            min = min.min(projection);
            max = max.max(projection);
        }
        
        (min, max)
    }

    pub fn get_axes(rotation: &Quaternion<f32>) -> [Vector3<f32>; 3] {
        [
            rotation.rotate_vector(Vector3::new(1.0, 0.0, 0.0)),
            rotation.rotate_vector(Vector3::new(0.0, 1.0, 0.0)),
            rotation.rotate_vector(Vector3::new(0.0, 0.0, 1.0)),
        ]
    }

    pub fn get_corners(data: (&Vector3<f32>, &Quaternion<f32>), center: &Vector3<f32>) -> [Vector3<f32>; 8] {
        let axes = Self::get_axes(data.1);
        let e = data.0; //this is left over code 
        [
            center + axes[0] * e.x + axes[1] * e.y + axes[2] * e.z,
            center + axes[0] * e.x + axes[1] * e.y - axes[2] * e.z,
            center + axes[0] * e.x - axes[1] * e.y + axes[2] * e.z,
            center + axes[0] * e.x - axes[1] * e.y - axes[2] * e.z,
            center - axes[0] * e.x + axes[1] * e.y + axes[2] * e.z,
            center - axes[0] * e.x + axes[1] * e.y - axes[2] * e.z,
            center - axes[0] * e.x - axes[1] * e.y + axes[2] * e.z,
            center - axes[0] * e.x - axes[1] * e.y - axes[2] * e.z,
        ]
    }

    // pub fn check_box_collision_with_rotation(
    //     entity_a: u32,
    //     pos_a: Vector3<f32>,
    //     rot_a: Quaternion<f32>,
    //     width_a: f32,
    //     height_a: f32,
    //     depth_a: f32,
    //     entity_b: u32,
    //     pos_b: Vector3<f32>,
    //     rot_b: Quaternion<f32>,
    //     width_b: f32,
    //     height_b: f32,
    //     depth_b: f32,
    // ) -> Option<CollisionEvent> {
    //     let obb_a = OBB::new(
    //         Vector3::new(width_a / 2.0, height_a / 2.0, depth_a / 2.0),
    //         rot_a,
    //     );
        
    //     let obb_b = OBB::new(
    //         Vector3::new(width_b / 2.0, height_b / 2.0, depth_b / 2.0),
    //         rot_b,
    //     );
        
    //     if let Some((normal, penetration)) = check_obb_collision(&obb_a, pos_a, &obb_b, pos_b) {
    //         let collision_point = pos_b + normal * (penetration / 2.0);
            
    //         Some(CollisionEvent {
    //             entity_a,
    //             entity_b,
    //             collision_point,
    //             normal,
    //             penetration,
    //         })
    //     } else {
    //         None
    //     }
    // }

    pub fn check_box_collision_with_rotation(
        obb_a: &Collider,
        obb_b: &Collider,
        entity_a: u32,
        pos_a: Vector3<f32>,
        entity_b: u32,
        pos_b: Vector3<f32>,
    ) -> Option<CollisionEvent> {
        if let Some((normal, penetration)) = Self::check_obb_collision(&obb_a, pos_a, &obb_b, pos_b) {
            let collision_point = pos_b + normal * (penetration / 2.0);
            
            Some(CollisionEvent {
                entity_a,
                entity_b,
                collision_point,
                normal,
                penetration,
            })
        } else {
            None
        }
    }

    pub fn update_no_physics(&mut self, movement_system: &mut MovementSystem, delta_time: f32) {
        self.collision_events.clear();
        
        // Get all entities with both colliders and positions
        let mut entities_with_collision: Vec<(u32, Vector3<f32>, &Collider)> = Vec::new();
        
        for (entity_id, collider) in &self.colliders {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                entities_with_collision.push((*entity_id, coords.position + collider.offset, collider));
            }
        }
        
        // Check all pairs for collision
        for i in 0..entities_with_collision.len() {
            for j in (i + 1)..entities_with_collision.len() {
                let (entity_a, pos_a, collider_a) = entities_with_collision[i];
                let (entity_b, pos_b, collider_b) = entities_with_collision[j];
                

                print!("comparing {} and {}\n", entity_a, entity_b);
                print!("positions: {:?} and {:?}\n", pos_a, pos_b);
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

    pub fn update(&mut self, movement_system: &mut MovementSystem, physics_system: &mut PhysicsSystem, delta_time: f32) {
        self.collision_events.clear();
        
        let mut entities_with_collision: Vec<(u32, Vector3<f32>, &Collider)> = Vec::new();
        
        //this is where the position of the colliders is calculated btw
        for (entity_id, collider) in &self.colliders {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                entities_with_collision.push((*entity_id, coords.position + collider.offset, collider));//no rotation or scaling yet
                //todo add
            }
        }
        
        for i in 0..entities_with_collision.len() {
            for j in (i + 1)..entities_with_collision.len() {
                let (entity_a, pos_a, collider_a) = entities_with_collision[i];
                let (entity_b, pos_b, collider_b) = entities_with_collision[j];
                
                if !self.can_collide(collider_a.layer, collider_b.layer) {
                    continue;
                }
                
                if let Some(collision) = self.check_collision(
                    entity_a, pos_a, collider_a,
                    entity_b, pos_b, collider_b
                ) {
                    self.collision_events.push(collision.clone());
                    
                    if !collider_a.is_trigger && !collider_b.is_trigger {
                        // Use physics system for resolution
                        physics_system.resolve_collision(movement_system, &collision);
                    }
                }
            }
        }
    }

    pub fn update_obb(&mut self, movement_system: &mut MovementSystem, physics_system: &mut PhysicsSystem, delta_time: f32) {
        self.collision_events.clear();
        
        let mut entities_with_collision: Vec<(u32, Vector3<f32>, &Collider)> = Vec::new();
        
        //this is where the position of the colliders is calculated btw
        for (entity_id, collider) in &self.colliders {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                entities_with_collision.push((*entity_id, coords.position + collider.offset, collider));//no rotation or scaling yet
                //todo add
            }
        }
        
        for i in 0..entities_with_collision.len() {
            for j in (i + 1)..entities_with_collision.len() {
                let (entity_a, pos_a, collider_a) = entities_with_collision[i];
                let (entity_b, pos_b, collider_b) = entities_with_collision[j];
                
                if !self.can_collide(collider_a.layer, collider_b.layer) {
                    continue;
                }
                
                if let Some(collision) = self.check_collision(
                    entity_a, pos_a, collider_a,
                    entity_b, pos_b, collider_b
                ) {
                    self.collision_events.push(collision.clone());
                    
                    if !collider_a.is_trigger && !collider_b.is_trigger {
                        // Use physics system for resolution
                        physics_system.resolve_collision(movement_system, &collision);
                    }
                }
            }
        }
    }
    
    //really im going to be honest, i have like no memory of how to do collision so i hope that this is correct
    //btw it was not and it was really wrong... its fixed now but yeah
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

            (CollisionShape::Circle { radius }, CollisionShape::Rectangle { width, height }) => {
                let half_w = width / 2.0;
                let half_h = height / 2.0;
                
                // Find the closest point on the rectangle to the circle's center
                let closest_x = pos_a.x.max(pos_b.x - half_w).min(pos_b.x + half_w);
                let closest_y = pos_a.y.max(pos_b.y - half_h).min(pos_b.y + half_h);
                
                // Calculate distance from circle center to this closest point
                let dx = pos_a.x - closest_x;
                let dy = pos_a.y - closest_y;
                let distance_squared = dx * dx + dy * dy;
                
                if distance_squared < radius * radius {
                    let distance = distance_squared.sqrt();
                    let penetration = radius - distance;
                    
                    let normal = if distance > 0.0 {
                        Vector2::new(dx, dy).normalize()
                    } else {
                        // Circle center is inside rectangle, push out along closest axis
                        let dx_edge = (pos_a.x - pos_b.x).abs() - half_w;
                        let dy_edge = (pos_a.y - pos_b.y).abs() - half_h;
                        
                        if dx_edge > dy_edge {
                            Vector2::new(if pos_a.x > pos_b.x { 1.0 } else { -1.0 }, 0.0)
                        } else {
                            Vector2::new(0.0, if pos_a.y > pos_b.y { 1.0 } else { -1.0 })
                        }
                    };
                    
                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(closest_x, closest_y, pos_a.z),
                        normal: Vector3::new(normal.x, normal.y, 0.0),
                        penetration,
                    })
                } else {
                    None
                }
            },

            // Rectangle vs Circle (2D) - just swap the entities
            (CollisionShape::Rectangle { width, height }, CollisionShape::Circle { radius }) => {
                let half_w = width / 2.0;
                let half_h = height / 2.0;
                
                // Find the closest point on the rectangle to the circle's center
                let closest_x = pos_b.x.max(pos_a.x - half_w).min(pos_a.x + half_w);
                let closest_y = pos_b.y.max(pos_a.y - half_h).min(pos_a.y + half_h);
                
                // Calculate distance from circle center to this closest point
                let dx = pos_b.x - closest_x;
                let dy = pos_b.y - closest_y;
                let distance_squared = dx * dx + dy * dy;
                
                if distance_squared < radius * radius {
                    let distance = distance_squared.sqrt();
                    let penetration = radius - distance;
                    
                    let normal = if distance > 0.0 {
                        Vector2::new(dx, dy).normalize()
                    } else {
                        // Circle center is inside rectangle, push out along closest axis
                        let dx_edge = (pos_b.x - pos_a.x).abs() - half_w;
                        let dy_edge = (pos_b.y - pos_a.y).abs() - half_h;
                        
                        if dx_edge > dy_edge {
                            Vector2::new(if pos_b.x > pos_a.x { 1.0 } else { -1.0 }, 0.0)
                        } else {
                            Vector2::new(0.0, if pos_b.y > pos_a.y { 1.0 } else { -1.0 })
                        }
                    };
                    
                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(closest_x, closest_y, pos_b.z),
                        normal: Vector3::new(-normal.x, -normal.y, 0.0), // Flip normal since entity_a is the rectangle
                        penetration,
                    })
                } else {
                    None
                }
            },

            (CollisionShape::Sphere { radius: r1 }, CollisionShape::Box { width: w2, height: h2, depth: d2 }) => {
                // Sphere vs AABB collision detection
                let half_w = w2 / 2.0;
                let half_h = h2 / 2.0;
                let half_d = d2 / 2.0;
                
                // Find the closest point on the box to the sphere's center
                let closest_x = pos_a.x.max(pos_b.x - half_w).min(pos_b.x + half_w);
                let closest_y = pos_a.y.max(pos_b.y - half_h).min(pos_b.y + half_h);
                let closest_z = pos_a.z.max(pos_b.z - half_d).min(pos_b.z + half_d);
                
                // Calculate distance from sphere center to this closest point
                let dx = pos_a.x - closest_x;
                let dy = pos_a.y - closest_y;
                let dz = pos_a.z - closest_z;
                let distance_squared = dx * dx + dy * dy + dz * dz;
                
                if distance_squared < r1 * r1 {
                    let distance = distance_squared.sqrt();
                    let penetration = r1 - distance;
                    
                    let normal = if distance > 0.0 {
                        Vector3::new(dx, dy, dz).normalize()
                    } else {
                        // Sphere center is inside box, push out along closest axis
                        let dx_edge = (pos_a.x - pos_b.x).abs() - half_w;
                        let dy_edge = (pos_a.y - pos_b.y).abs() - half_h;
                        let dz_edge = (pos_a.z - pos_b.z).abs() - half_d;
                        
                        if dx_edge <= dy_edge && dx_edge <= dz_edge {
                            Vector3::new(if pos_a.x > pos_b.x { 1.0 } else { -1.0 }, 0.0, 0.0)
                        } else if dy_edge <= dz_edge {
                            Vector3::new(0.0, if pos_a.y > pos_b.y { 1.0 } else { -1.0 }, 0.0)
                        } else {
                            Vector3::new(0.0, 0.0, if pos_a.z > pos_b.z { 1.0 } else { -1.0 })
                        }
                    };

                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(closest_x, closest_y, closest_z),
                        normal,
                        penetration,
                    })
                } else {
                    None
                }
            },

            (CollisionShape::Box { width: w2, height: h2, depth: d2 }, CollisionShape::Sphere { radius: r1 }) => {
                // Sphere vs AABB collision detection
                let half_w = w2 / 2.0;
                let half_h = h2 / 2.0;
                let half_d = d2 / 2.0;
                
                // Find the closest point on the box to the sphere's center
                let closest_x = pos_a.x.max(pos_b.x - half_w).min(pos_b.x + half_w);
                let closest_y = pos_a.y.max(pos_b.y - half_h).min(pos_b.y + half_h);
                let closest_z = pos_a.z.max(pos_b.z - half_d).min(pos_b.z + half_d);
                
                // Calculate distance from sphere center to this closest point
                let dx = pos_a.x - closest_x;
                let dy = pos_a.y - closest_y;
                let dz = pos_a.z - closest_z;
                let distance_squared = dx * dx + dy * dy + dz * dz;
                
                if distance_squared < r1 * r1 {
                    let distance = distance_squared.sqrt();
                    let penetration = r1 - distance;
                    
                    let normal = if distance > 0.0 {
                        Vector3::new(dx, dy, dz).normalize()
                    } else {
                        // Sphere center is inside box, push out along closest axis
                        let dx_edge = (pos_a.x - pos_b.x).abs() - half_w;
                        let dy_edge = (pos_a.y - pos_b.y).abs() - half_h;
                        let dz_edge = (pos_a.z - pos_b.z).abs() - half_d;
                        
                        if dx_edge <= dy_edge && dx_edge <= dz_edge {
                            Vector3::new(if pos_a.x > pos_b.x { 1.0 } else { -1.0 }, 0.0, 0.0)
                        } else if dy_edge <= dz_edge {
                            Vector3::new(0.0, if pos_a.y > pos_b.y { 1.0 } else { -1.0 }, 0.0)
                        } else {
                            Vector3::new(0.0, 0.0, if pos_a.z > pos_b.z { 1.0 } else { -1.0 })
                        }
                    };

                    Some(CollisionEvent {
                        entity_a,
                        entity_b,
                        collision_point: Vector3::new(closest_x, closest_y, closest_z),
                        normal,
                        penetration,
                    })
                } else {
                    None
                }
            },

            (CollisionShape::OBB { .. }, CollisionShape::OBB { .. }) => {
                Self::check_box_collision_with_rotation(
                    collider_a,
                    collider_b,
                    entity_a,
                    pos_a,
                    entity_b,
                    pos_b,
                )
            },
            
            // need to add all the mixed collision types later
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

    fn get_collider_as_mesh(&self, entity_id: u32, coords: &WorldCoords) -> Option<crate::model::mesh::Mesh> {
        let collider = self.get_collider(entity_id)?;
        match &collider.shape {
            CollisionShape::Sphere { radius } => {
                Some({
                    //lol so these have position so that I dont have to create a new shader for them and impliment a model matrix i am just plopping the position in here
                    let (vertices, indices) = crate::model::mesh::Mesh::create_sphere(*radius, 16, 16, *coords.get_position() + collider.offset);//16 was arbitrary choice for segments
                    crate::model::mesh::Mesh::new(&vertices, &indices)
                })
            },
            CollisionShape::Box { width, height, depth } => {
                Some({
                    let (vertices, indices) = crate::model::mesh::Mesh::create_box(*width, *height, *depth, *coords.get_position()  + collider.offset);//16 was arbitrary choice for segments
                    crate::model::mesh::Mesh::new(&vertices, &indices)
                })
            },
            CollisionShape::OBB { half_extents, rotation } => {
                let width = half_extents.x * 2.0;
                let height = half_extents.y * 2.0;
                let depth = half_extents.z * 2.0;
                Some({
                    let (vertices, indices) = crate::model::mesh::Mesh::create_box(width, height, depth, *coords.get_position()  + collider.offset);//16 was arbitrary choice for segments
                    crate::model::mesh::Mesh::new(&vertices, &indices)
                })
            },
            _ => None, // I am not really sure how to go about 2d shapes as meshes if i would even want to do that well let me rephrase this bc duh ofc they are meshes what i mean actually is i am lazy and they would need a different renderer and idk how they should look
        }
    }

    // pub fn draw_colliders(&self, movement_system: &MovementSystem, shader_program: &crate::graphics::gl_wrapper::ShaderProgram, view_matrix: &cgmath::Matrix4<f32>, projection_matrix: &cgmath::Matrix4<f32>) {
    //     for (entity_id, _) in &self.colliders {
    //         if let Some(collider_mesh) = self.get_collider_as_mesh(*entity_id) {
    //             //if let Some(coords) = movement_system.get_coords(*entity_id) {
    //                 //let model_matrix = coords.get_model_matrix();
    //                 //let mvp_matrix = *projection_matrix * *view_matrix * model_matrix;
                    
    //                 //shader_program.set_uniform_matrix4("u_MVP", &mvp_matrix);
                    
    //                 collider_mesh.draw();
    //             //}
    //         }
    //     }
    // }

    pub fn draw_colliders(&self, movement_system: &MovementSystem) {
        for (entity_id, _) in &self.colliders {
            if let Some(coords) = movement_system.get_coords(*entity_id) {
                if let Some(collider_mesh) = self.get_collider_as_mesh(*entity_id, coords) {
                
                    //let model_matrix = coords.get_model_matrix();
                    //let mvp_matrix = *projection_matrix * *view_matrix * model_matrix;
                    
                    //shader_program.set_uniform_matrix4("u_MVP", &mvp_matrix);
                    
                    collider_mesh.draw();
                }
            }
        }
    }

    pub fn draw_colliders_2(&mut self, movement_system: &MovementSystem, view_matrix: &cgmath::Matrix4<f32>, projection_matrix: &cgmath::Matrix4<f32>) {
        if let Some(shader) = self.collision_shader.as_mut(){
            shader.bind();
            shader.set_matrix4fv_uniform("view", view_matrix);
            shader.set_matrix4fv_uniform("projection", projection_matrix);
            for (entity_id, _) in &self.colliders {
                if let Some(coords) = movement_system.get_coords(*entity_id) {
                    if let Some(collider_mesh) = self.get_collider_as_mesh(*entity_id, coords) {
                    
                        let model_matrix = coords.get_model_matrix();
                        //let mvp_matrix = *projection_matrix * *view_matrix * model_matrix;
                        
                        shader.set_matrix4fv_uniform("model", &model_matrix);
                        
                        collider_mesh.draw();
                    }
                }
            }
        } else {
            print!("No collision shader available for drawing colliders.\n");
        }
    }
}