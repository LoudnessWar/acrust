use cgmath::{Matrix4, Vector3, Quaternion, Rad, Rotation3, Transform, InnerSpace};

pub struct WorldCoords {//Im realizing that this needs to be a trait. Then I can attach lists of world coords or something like that to an object
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>, // More flexible than Euler angles aparently idk
    pub scale: Vector3<f32>,
}

impl WorldCoords {
    pub fn new(x: f32, y: f32, z: f32, rotation: f32) -> Self {//do not need f32 for all these prolly lets be honest
        WorldCoords {
            position: Vector3::new(x, y, z),
            rotation: Quaternion::from_angle_y(Rad(rotation)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn new_empty() -> Self {
        WorldCoords {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_angle_y(Rad(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {//holy balls lol I forgot i had this
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    //vectors for the object just normals and shit idk not really 
    pub fn get_forward_vector(&self) -> Vector3<f32> {
        let rotation_matrix = Matrix4::from(self.rotation);
        let forward = rotation_matrix.transform_vector(Vector3::new(0.0, 0.0, -1.0));//TODO check if this vector stuff is correct
        forward.normalize()
    }
    
    pub fn get_left_vector(&self) -> Vector3<f32> {
        let forward = self.get_forward_vector();
        let up = Vector3::new(0.0, 1.0, 0.0); // World-up vector
        forward.cross(up).normalize() // Cross product gives a perpendicular left vector
    }

    //these doesnt use its own vector because you might want to move it off a parent vector or something or just like whatever direction you want tbh
    //these could be all one thing they do the same thing literally why not make them one
    //its because when I look at this I am like yeah ok that makes sense
    pub fn move_forward(&mut self, for_vec: Vector3<f32>, distance: f32) {
        self.position += for_vec * distance;
    }

    pub fn move_backward(&mut self, for_vec: Vector3<f32>, distance: f32) {
        self.position -= for_vec * distance;
    }

    pub fn move_left(&mut self, left_vec: Vector3<f32>, distance: f32) {
        self.position -= left_vec * distance;
    }

    pub fn move_right(&mut self, left_vec: Vector3<f32>, distance: f32) {
        self.position += left_vec * distance;
    }
    //add more getters and setter for rotation and scale later
    pub fn set_position(&mut self, new_position: Vector3<f32>) {
        self.position = new_position;
    }

    pub fn set_rotation(&mut self, new_rotation: f32){
        self.rotation = Quaternion::from_angle_y(Rad(new_rotation))
    }

    pub fn set_rotation_from_quaternion(&mut self, new_rotation: Quaternion<f32>){
        self.rotation = new_rotation
    }

    //this should be a reference, this is because, we have a set position 1 and 2 lets say we have a model or something with world coords
    //right we then want to get the position of that model somewhere else
    //ok so we call get position then again from that model we call it again
    //like &self.worldcoords.get_position() what we have done here
    //is move position into that model/function temporarily, then we try to make a reference to that changed owner ship
    //that is not something that is consistantly there, so the reference will not be able to
    //access that correct point in memory and just
    //probably pull nonsense
    //you can also just de refernece the borrow later on if needed but like... I wouldnt do that bruh unless i really needd owener ship but it is a
    //nice option so this is more flexible in this case
    pub fn get_position(&self) -> &Vector3<f32>{
        &self.position
    }

    // pub fn get_rotation
}

//ok so this is like a really dumb but also simple like 
//way to make it so that I can store a list of objects
//that are attachable to other objects
//and that is... keep world coords as its own thing
//but then also have this coords thing
//all it needs it set position
//get position
//and maybe update children
//i think ill add that but most of the time it will be empty
//the reason being
//I want it to not be too complicated for the user but I will add that later bc im still contimplating it so TODO
//also make worldcoords impliment coords trait... probably
//idk it needs to be something that the objects themselves inherit so maybe not actaully
pub trait Coords {
    fn update_position(&mut self);//rn does not take in , new_position: Vector3<f32> maybe add back later
}