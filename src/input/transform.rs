use cgmath::{Matrix4, Vector3, Quaternion, Rad, Rotation3, Transform, InnerSpace};

pub struct WorldCoords {
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

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    //vectors for the object just normals and shit idk not really 
    pub fn get_forward_vector(&self) -> Vector3<f32> {
        let rotation_matrix = Matrix4::from(self.rotation);
        let forward = rotation_matrix.transform_vector(Vector3::new(0.0, 0.0, -1.0));
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


}
