use cgmath::{InnerSpace, Matrix4, PerspectiveFov, Point3, Rad, Vector3, Quaternion, EuclideanSpace, Transform, Rotation3};
use crate::model::transform::WorldCoords;

pub struct Camera {
    pub transform: WorldCoords, // Camera's local transform
    pub projection: Matrix4<f32>, // Perspective or orthographic projection matrix
    pub view: Matrix4<f32>, // View matrix
    pub parent: Option<*const WorldCoords>,
}

impl Camera {
    pub fn new(perspective: PerspectiveFov<f32>) -> Self {
        let projection = Matrix4::from(perspective);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        Camera {
            transform: WorldCoords::new_empty(),
            projection,
            view,
            parent: None,
        }
    }

    pub fn update_view(&mut self) {
        let global_position = if let Some(parent) = self.parent {
            let parent_transform = unsafe { &*parent };
            parent_transform.position + self.transform.position
        } else {
            self.transform.position
        };
    
        let forward = self.get_forward_vector();
        let target = global_position + forward;
        
        let up = Vector3::new(0.0, 1.0, 0.0);
        let right = forward.cross(up).normalize();
        let corrected_up = right.cross(forward).normalize();
    
        self.view = Matrix4::look_at_rh(
            Point3::from_vec(global_position),
            Point3::from_vec(target),
            corrected_up
        );
    }

    pub fn get_vp_matrix(&self) -> Matrix4<f32> {
        self.projection * self.view
    }

    pub fn get_p_matrix(&self) -> Matrix4<f32> {
        self.projection
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        self.view
    }

    //Literally this is bc maybe you want this just for the camera and this will let you do it ezier
    pub fn get_forward_vector(&self) -> Vector3<f32> {
        self.transform.get_forward_vector()
    }

    pub fn get_left_vector(&self) -> Vector3<f32> {
        self.transform.get_left_vector()
    }

    pub fn get_rotation(&self) -> cgmath::Quaternion<f32> {
        self.transform.rotation
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        // First, create quaternions for rotation
        let yaw_rotation = Quaternion::from_angle_y(Rad(delta_yaw));
        let pitch_rotation = Quaternion::from_angle_x(Rad(delta_pitch));
    
        // Combine rotations
        let combined_rotation = yaw_rotation * self.transform.rotation * pitch_rotation;
    
        // Update rotation
        self.transform.rotation = combined_rotation;
    
        // Clamp pitch to prevent extreme rotations
        let pitch_limit = Rad(std::f32::consts::FRAC_PI_2 - 0.1); // Slightly less than 90 degrees
        let current_pitch = self.get_pitch_angle();
    
        if current_pitch.abs() > pitch_limit.0 {
            // If pitch exceeds limit, project back to the limit
            let corrected_rotation = if current_pitch > 0.0 {
                Quaternion::from_angle_x(pitch_limit)
            } else {
                Quaternion::from_angle_x(-pitch_limit)
            };
            self.transform.rotation = corrected_rotation;
        }
    }

    fn get_pitch_angle(&self) -> f32 {
        let rotation_matrix: Matrix4<f32> = Matrix4::from(self.transform.rotation);
    
        let forward = rotation_matrix.transform_vector(Vector3::new(0.0, 0.0, -1.0));
        forward.y.asin()
    }

    //attach and detach to a partent function
    pub fn attach_to(&mut self, parent: &WorldCoords) {
        self.parent = Some(parent as *const WorldCoords);
    }

    pub fn detach(&mut self) {
        self.parent = None;
    }
}

