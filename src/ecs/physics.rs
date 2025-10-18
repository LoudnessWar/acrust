
//lol these are just to start
pub enum physicsType {
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
pub struct physicsEntity { //this is more like physics entity data doe btw
    pub name: String,//probabl not needed
    pub phys_type: physicsType,

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

impl physicsEntity {
    pub fn static_body(name: &str, position: Vector3<f32>, collider: ColliderData) -> Self {
        Self {
            name: name.to_string(),
            archetype: PhysicsArchetype::Static,
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
    
    pub fn dynamic_body(name: &str, position: Vector3<f32>, collider: ColliderData, mass: f32) -> Self {
        Self {
            name: name.to_string(),
            archetype: PhysicsArchetype::Dynamic,
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
    
    pub fn kinematic_body(name: &str, position: Vector3<f32>, collider: ColliderData) -> Self {
        Self {
            name: name.to_string(),
            archetype: PhysicsArchetype::Kinematic,
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
    
    pub fn trigger(name: &str, position: Vector3<f32>, shape: CollisionShapeData) -> Self {
        Self {
            name: name.to_string(),
            archetype: PhysicsArchetype::Trigger,
            position,
            rotation: 0.0,
            collider: ColliderData {
                shape,
                is_trigger: true,
                layer: 0,
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

pub struct RigidBody {
    pub mass: f32,
    pub restitution: f32,
    pub friction: f32,
    pub force_accumulator: Vector3<f32>,
}


pub struct PhysicsSystem {
    rigidbodies: ComponentStorage<RigidBody>,
}

