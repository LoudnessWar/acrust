use acrust::input::transform::WorldCoords;
use cgmath::{Vector3, Matrix4, Rotation, Transform, InnerSpace};


//ok so I need a game object abstract class or smthing because this and camera lowkey have like the same functions
pub struct Player {
    pub transform: WorldCoords,
    pub speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Player {
            transform: WorldCoords::new(),
            speed: 0.1,
        }
    }

    // Method to calculate forward vector based on rotation
    fn get_forward_vector(&self, rotation: cgmath::Quaternion<f32>) -> Vector3<f32> {
        let rotation_matrix = Matrix4::from(rotation);
        let forward = rotation_matrix.transform_vector(Vector3::new(0.0, 0.0, -1.0));
        forward.normalize()
    }

    // Method to calculate left vector based on forward vector
    fn get_left_vector(&self, rotation: cgmath::Quaternion<f32>) -> Vector3<f32> {
        let forward = self.get_forward_vector(rotation);
        let up = Vector3::new(0.0, 1.0, 0.0); // World-up vector
        forward.cross(up).normalize()
    }

    pub fn move_forward(&mut self, camera_rotation: cgmath::Quaternion<f32>) {
        let forward = self.get_forward_vector(camera_rotation);
        self.transform.position += forward * self.speed;
    }

    pub fn move_backward(&mut self, camera_rotation: cgmath::Quaternion<f32>) {
        let forward = self.get_forward_vector(camera_rotation);
        self.transform.position -= forward * self.speed;
    }

    pub fn move_left(&mut self, camera_rotation: cgmath::Quaternion<f32>) {
        let left = self.get_left_vector(camera_rotation);
        self.transform.position -= left * self.speed;
    }

    pub fn move_right(&mut self, camera_rotation: cgmath::Quaternion<f32>) {
        let left = self.get_left_vector(camera_rotation);
        self.transform.position += left * self.speed;
    }
}