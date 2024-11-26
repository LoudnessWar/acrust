use cgmath::{InnerSpace, Matrix4, PerspectiveFov, Point3, Rad, Vector3, Angle, EuclideanSpace};

pub struct Camera {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>, // Pitch, Yaw, Roll
    pub projection: Matrix4<f32>, // Perspective or orthographic projection matrix
    pub view: Matrix4<f32>, // View matrix
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
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            projection,
            view,
        }
    }

    //ok so this now is way more intuitive... nothing was based off of world coordinates earlier... I need to go through
    //and check that that wasnt the reason why bad things were previously happening
    pub fn update_view(&mut self) {
        // Calculate the forward vector
        let forward = self.get_forward_vector();
    
        // Calculate the target position (where the camera is looking at)
        let target = self.position + forward;
    
        // Define the up vector (world up, usually along the y-axis)
        let up = Vector3::new(0.0, 1.0, 0.0);
    
        // Create a view matrix using look_at (fixes rotation issues)
        self.view = Matrix4::look_at_rh(
            Point3::from_vec(self.position),
            Point3::from_vec(target),
            up,
        );
    }

    //le old
    // pub fn update_view(&mut self) {
    //     let rotation_matrix = Matrix4::from_angle_y(Rad(self.rotation.y))
    //         * Matrix4::from_angle_x(Rad(self.rotation.x));
    //     let translation_matrix = Matrix4::from_translation(-self.position);
    
    //     self.view = rotation_matrix * translation_matrix;
    // }

    pub fn get_vp_matrix(&self) -> Matrix4<f32> {
        self.projection * self.view
    }

    fn get_forward_vector(&self) -> Vector3<f32> {
        let pitch = Rad(self.rotation.x);
        let yaw = Rad(self.rotation.y);
    
        Vector3::new(
            yaw.sin() * pitch.cos(), // Corrected x component
            -pitch.sin(),           // Corrected y component (negated for camera pitch)
            -yaw.cos() * pitch.cos(), // Corrected z component
        )
        .normalize()
    }
    
    fn get_left_vector(&self) -> Vector3<f32> {
        let forward = self.get_forward_vector();
        let up = Vector3::new(0.0, 1.0, 0.0); // World-up vector
        forward.cross(up).normalize() // Cross product gives a perpendicular left vector
    }

    // fn get_left_vector(&self) -> Vector3<f32> {
    //     let rotation_matrix = Matrix4::from_angle_y(Rad(self.rotation.y));
    //     let left = Vector3::new(-1.0, 0.0, 0.0); // Default left vector
    //     (rotation_matrix * left.extend(0.0)).truncate()
    // }

    //todo make it so that these all dont have an update view in them
    //its very redundant!!! can just do one update at the end prolly
    pub fn move_forward(&mut self, distance: f32) {
    let forward = self.get_forward_vector();
    self.position += forward * distance;
    self.update_view();
    }

    pub fn move_backward(&mut self, distance: f32) {
        let forward = self.get_forward_vector();
        self.position -= forward * distance;
        self.update_view();
    }

    pub fn move_left(&mut self, distance: f32) {
        let left = self.get_left_vector();
        self.position -= left * distance;
        self.update_view();
    }

    pub fn move_right(&mut self, distance: f32) {
        let left = self.get_left_vector();
        self.position += left * distance;
        self.update_view();
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.rotation.y += delta_yaw;
        self.rotation.x += delta_pitch;
        self.update_view();
    }
}






























// use cgmath::{Matrix4, Vector3, Deg, PerspectiveFov, Rad, Euler, Angle};

// pub struct Camera {
//     position: Vector3<f32>,
//     rotation: Euler<Rad<f32>>,
//     projection_matrix: Matrix4<f32>,
//     view_matrix: Matrix4<f32>,
// }

// impl Camera {
//     pub fn new(position: Vector3<f32>, rotation: Euler<Rad<f32>>, fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
//         let projection_matrix = Matrix4::from(PerspectiveFov {
//             fovy: Rad(fov),
//             aspect: aspect_ratio,
//             near,
//             far,
//         });

//         let mut camera = Camera {
//             position,
//             rotation,
//             projection_matrix,
//             view_matrix: Matrix4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0),
//         };

//         camera.update_view_matrix();
//         camera
//     }

//     pub fn update_view_matrix(&mut self) {
//         // Compute the view matrix using position and rotation (camera space)
//         let translation = Matrix4::from_translation(-self.position);
//         let rotation = Matrix4::from(self.rotation);
//         self.view_matrix = rotation * translation;
//     }

//     pub fn get_view_matrix(&self) -> &Matrix4<f32> {
//         &self.view_matrix
//     }

//     pub fn get_projection_matrix(&self) -> &Matrix4<f32> {
//         &self.projection_matrix
//     }

//     // Add methods to move and rotate the camera here as needed.
// }
