use cgmath::{Matrix4, Vector3, Quaternion, Rad, Rotation3};

pub struct WorldCoords {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>, // More flexible than Euler angles aparently idk
    pub scale: Vector3<f32>,
}

impl WorldCoords {
    pub fn new() -> Self {
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
}
