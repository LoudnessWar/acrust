use cgmath::{EuclideanSpace, InnerSpace, Matrix4, One, PerspectiveFov, Point3, Quaternion, Rad, Rotation, Rotation3, Transform, Vector3};
use crate::model::transform::WorldCoords;

pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
    Fixed,
    Free,//TODO free cam implementation
}

pub struct Camera {
    pub transform: WorldCoords,
    pub projection: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub parent: Option<*const WorldCoords>,
    pub follow_offset: Option<Vector3<f32>>,
    pub mode: CameraMode,
    pub third_person_rotation: Quaternion<f32>,
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
            follow_offset: None,
            mode: CameraMode::Fixed,
            third_person_rotation: Quaternion::one(),
        }
    }

    pub fn new_reversed_z(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_rad / 2.0).tan();
        let projection = Matrix4::<f32>::new(
            f / aspect, 0.0,  0.0,                  0.0,
            0.0,         f,    0.0,                  0.0,
            0.0,         0.0,  near / (far - near), -1.0,
            0.0,         0.0,  (far * near) / (far - near), 0.0,
        );

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
            follow_offset: None,
            mode: CameraMode::Fixed,
            third_person_rotation: Quaternion::one(),
        }
    }

    pub fn update_view(&mut self) {
        let (global_position, target) = match self.mode {
            CameraMode::FirstPerson => {
                if let Some(parent) = self.parent {
                    let parent_transform = unsafe { &*parent };
                    let position = parent_transform.position;
                    let forward = self.transform.get_forward_vector();//maybe this needs inverse?
                    let target = position + forward;
                    (position, target)
                } else {
                    // Standalone first-person camera
                    let forward = self.get_forward_vector();
                    let position = self.transform.position;
                    let target = position + forward;
                    (position, target)
                }
            },
            CameraMode::ThirdPerson => {
                if let Some(parent) = self.parent {
                    let parent_transform = unsafe { &*parent };
                    
                    // Get the offset (default to a position behind and above if none specified)
                    let offset = self.follow_offset.unwrap_or(Vector3::new(0.0, 1.5, 5.0));
                    
                    // Calculate the orbital position using the third_person_rotation
                    // This rotation is relative to the default behind position
                    let rotated_offset = self.third_person_rotation.rotate_vector(offset);
                    
                    // Camera position is parent position plus the rotated offset
                    let position = parent_transform.position + rotated_offset;
                    
                    // Look at the parent's position (or slightly above it for better view)
                    let target = parent_transform.position + Vector3::new(0.0, 1.0, 0.0);
                    
                    (position, target)
                } else {
                    // Standalone third-person camera (no parent)
                    // Just use ordinary camera position and forward vector
                    (self.transform.position, self.transform.position + self.get_forward_vector())
                }
            },
            CameraMode::Fixed => {
                let forward = self.get_forward_vector();
                let position = self.transform.position;
                let target = position + forward;
                (position, target)
            },
            CameraMode::Free => {
                let forward = self.get_forward_vector();
                let position = self.transform.position;
                let target = position + forward;
                (position, target)
            }
        };
    
        // Calculate the view matrix
        let forward = (target - global_position).normalize();
        let right = forward.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
        let corrected_up = right.cross(forward).normalize();
    
        self.view = Matrix4::look_at_rh(
            Point3::from_vec(global_position),
            Point3::from_vec(target),
            corrected_up,
        );
    }
    
    pub fn get_vp_matrix(&self) -> Matrix4<f32> {
        self.projection * self.view
    }

    pub fn get_p_matrix(&self) -> &Matrix4<f32> {
        &self.projection
    }

    pub fn get_view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn get_forward_vector(&self) -> Vector3<f32> {
        self.transform.get_forward_vector()
    }

    pub fn get_left_vector(&self) -> Vector3<f32> {
        self.transform.get_left_vector()
    }

    pub fn get_rotation(&self) -> cgmath::Quaternion<f32> {
        self.transform.rotation
    }

    pub fn set_third_person_distance(&mut self, distance: f32) {
        if let Some(offset) = &mut self.follow_offset {
            let direction = offset.normalize();
            *offset = direction * distance;
        } else {
            self.follow_offset = Some(Vector3::new(0.0, 5.0, distance));
        }
    }

    pub fn adjust_third_person_distance(&mut self, delta: f32) {
        if let Some(offset) = &mut self.follow_offset {
            let current_length = offset.magnitude();
            let new_length = (current_length + delta).max(2.0).min(20.0); // Clamp between 2.0 and 20.0
            
            *offset = offset.normalize() * new_length;
        }
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        let yaw_rotation = Quaternion::from_angle_y(Rad(delta_yaw));
        let pitch_rotation = Quaternion::from_angle_x(Rad(delta_pitch));

        match self.mode {
            CameraMode::ThirdPerson => {//dude yaw and pitch is this guy crazy!!!
                self.third_person_rotation = yaw_rotation * self.third_person_rotation * pitch_rotation;
                
                let forward = self.third_person_rotation.rotate_vector(Vector3::new(0.0, 0.0, -1.0));
                let pitch = forward.y.asin();
                
                let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.1;
                if pitch.abs() > pitch_limit {
                    let adjustment = if pitch > 0.0 {
                        pitch - pitch_limit
                    } else {
                        pitch + pitch_limit
                    };
                    
                    let correction = Quaternion::from_angle_x(Rad(-adjustment));
                    self.third_person_rotation = self.third_person_rotation * correction;
                }
            },
            _ => {

                let combined_rotation = yaw_rotation * self.transform.rotation * pitch_rotation;
                self.transform.rotation = combined_rotation;

                let pitch_limit = Rad(std::f32::consts::FRAC_PI_2 - 0.1);
                let current_pitch = self.get_pitch_angle();

                if current_pitch.abs() > pitch_limit.0 {
                    let corrected_rotation = if current_pitch > 0.0 {
                        Quaternion::from_angle_x(pitch_limit)
                    } else {
                        Quaternion::from_angle_x(-pitch_limit)
                    };
                    self.transform.rotation = corrected_rotation;
                }
            },
        }
    }

    pub fn get_move_forward_vector(&self) -> Vector3<f32> {
        match self.mode {
            CameraMode::ThirdPerson => {
                let forward = self.third_person_rotation.rotate_vector(Vector3::new(0.0, 0.0, -1.0));
                Vector3::new(forward.x, 0.0, forward.z).normalize()
            },
            _ => {
                let forward = self.get_forward_vector();
                Vector3::new(forward.x, 0.0, forward.z).normalize()//TODO add a toggel for this maybe want to beable to look down to move down
            }
        }
    }

    pub fn get_move_left_vector(&self) -> Vector3<f32> {
        match self.mode {
            CameraMode::ThirdPerson => {
                let left = self.third_person_rotation.rotate_vector(Vector3::new(-1.0, 0.0, 0.0));
                Vector3::new(left.x, 0.0, left.z).normalize()
            },
            _ => {
                let left = self.get_left_vector();
                Vector3::new(-left.x, 0.0, -left.z).normalize()//TODO this is a cheap trick of a fix why is this flipped
            }
        }
    }

    // pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
    //     let yaw_rotation = Quaternion::from_angle_y(Rad(delta_yaw));
    //     let pitch_rotation = Quaternion::from_angle_x(Rad(delta_pitch));

    //     match self.mode {
    //         CameraMode::ThirdPerson => {
    //             self.third_person_rotation = yaw_rotation * self.third_person_rotation * pitch_rotation;
    //         },
    //         _ => {
    //             let combined_rotation = yaw_rotation * self.transform.rotation * pitch_rotation;
    //             self.transform.rotation = combined_rotation;

    //             let pitch_limit = Rad(std::f32::consts::FRAC_PI_2 - 0.1);
    //             let current_pitch = self.get_pitch_angle();

    //             if current_pitch.abs() > pitch_limit.0 {
    //                 let corrected_rotation = if current_pitch > 0.0 {
    //                     Quaternion::from_angle_x(pitch_limit)
    //                 } else {
    //                     Quaternion::from_angle_x(-pitch_limit)
    //                 };
    //                 self.transform.rotation = corrected_rotation;
    //             }
    //         },
    //     }
    // }

    fn get_pitch_angle(&self) -> f32 {
        let rotation_matrix: Matrix4<f32> = Matrix4::from(self.transform.rotation);
        let forward = rotation_matrix.transform_vector(Vector3::new(0.0, 0.0, -1.0));
        forward.y.asin()
    }

    pub fn attach_to(&mut self, parent: &WorldCoords, offset: Vector3<f32>) {
        self.parent = Some(parent as *const WorldCoords);
        self.follow_offset = Some(offset);
    }

    pub fn detach(&mut self) {
        self.parent = None;
        self.follow_offset = None;
    }

    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
    }


    //I could use set_mode for these but... y
    pub fn fp(&mut self) {
        println!("fp");
        self.mode = CameraMode::FirstPerson;
    }

    pub fn tp(&mut self) {
        println!("tp");
        self.mode = CameraMode::ThirdPerson;
    }

    pub fn fixed(&mut self) {
        println!("fix");
        self.mode = CameraMode::Fixed;
    }

    pub fn free(&mut self) {
        println!("free");
        self.mode = CameraMode::Free;
    }

    pub fn cycle_mode(&mut self){
        match self.mode {
            CameraMode::FirstPerson => self.tp(),
            CameraMode::ThirdPerson => self.fixed(),
            CameraMode::Fixed => self.free(),
            CameraMode::Free => self.fp(),
        }
    }
}

// use cgmath::{EuclideanSpace, InnerSpace, Matrix4, PerspectiveFov, Point3, Quaternion, Rad, Rotation, Rotation3, Transform, Vector3};
// use crate::model::transform::WorldCoords;

// pub struct Camera {
//     pub transform: WorldCoords, // Camera's local transform
//     pub projection: Matrix4<f32>, // Perspective or orthographic projection matrix
//     pub view: Matrix4<f32>, // View matrix
//     pub parent: Option<*const WorldCoords>,
//     pub follow_offset: Option<Vector3<f32>>,
// }


// //ok I actually have a deliemma. Use Reverse Z or use linear space
// //also how do we handle camera player stuff
// //so either we put the player right in from of the camera getting like the direction the camera is looking at at an offset
// //or we move the player with the camera at an offset... 2 is probably better
// //we shouldnt have both follow the model. Thats a bad idea
// impl Camera {
//     pub fn new(perspective: PerspectiveFov<f32>) -> Self {
//         let projection = Matrix4::from(perspective);
//         let view = Matrix4::look_at_rh(
//             Point3::new(0.0, 0.0, 0.0),
//             Point3::new(0.0, 0.0, -1.0),
//             Vector3::new(0.0, 1.0, 0.0),
//         );
//         Camera {
//             transform: WorldCoords::new_empty(),
//             projection,
//             view,
//             parent: None,
//             follow_offset: None,
//         }
//     }

//     pub fn new_reversed_z(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> Self {

//         let f = 1.0 / (fov_y_rad / 2.0).tan();
    
//         let projection = Matrix4::<f32>::new(
//             f / aspect, 0.0,  0.0,                  0.0,
//             0.0,         f,    0.0,                  0.0,
//             0.0,         0.0,  near / (far - near), -1.0,
//             0.0,         0.0,  (far * near) / (far - near), 0.0,
//         );
    
//         let view = Matrix4::look_at_rh(
//             Point3::new(0.0, 0.0, 0.0),
//             Point3::new(0.0, 0.0, -1.0),
//             Vector3::new(0.0, 1.0, 0.0),
//         );
    
//         Camera {
//             transform: WorldCoords::new_empty(),
//             projection,
//             view,
//             parent: None,
//             follow_offset: None,
//         }
//     }

//     pub fn update_view(&mut self) {
//         let (global_position, target) = if let Some(parent) = self.parent {
//             let parent_transform = unsafe { &*parent };

//             let offset = self.follow_offset.unwrap_or(Vector3::new(0.0, 0.0, 0.0));
//             let rotated_offset = parent_transform.rotation.rotate_vector(offset);
//             let camera_position = parent_transform.position + rotated_offset;
//             let look_target = parent_transform.position;

//             (camera_position, look_target)
//         } else {
//             let forward = self.get_forward_vector();
//             let camera_position = self.transform.position;
//             let target = camera_position + forward;
//             (camera_position, target)
//         };

//         let forward = (target - global_position).normalize();
//         let right = forward.cross(Vector3::new(0.0, 1.0, 0.0)).normalize();
//         let corrected_up = right.cross(forward).normalize();

//         self.view = Matrix4::look_at_rh(
//             Point3::from_vec(global_position),
//             Point3::from_vec(target),
//             corrected_up,
//         );
//     }

//     // pub fn update_view(&mut self) {
//     //     let global_position = if let Some(parent) = self.parent {
//     //         let parent_transform = unsafe { &*parent };
//     //         parent_transform.position + self.transform.position
//     //     } else {
//     //         self.transform.position
//     //     };
    
//     //     let forward = self.get_forward_vector();
//     //     let target = global_position + forward;
        
//     //     let up = Vector3::new(0.0, 1.0, 0.0);
//     //     let right = forward.cross(up).normalize();
//     //     let corrected_up = right.cross(forward).normalize();
    
//     //     self.view = Matrix4::look_at_rh(
//     //         Point3::from_vec(global_position),
//     //         Point3::from_vec(target),
//     //         corrected_up
//     //     );
//     // }

//     pub fn get_vp_matrix(&self) -> Matrix4<f32> {//ok to like shader here bc its new value produced as product of the other two
//         self.projection * self.view
//     }

//     pub fn get_p_matrix(&self) -> &Matrix4<f32> {
//         &self.projection
//     }

//     pub fn get_view(&self) -> &Matrix4<f32> {
//         &self.view
//     }

//     //Literally this is bc maybe you want this just for the camera and this will let you do it ezier
//     pub fn get_forward_vector(&self) -> Vector3<f32> {
//         self.transform.get_forward_vector()
//     }

//     pub fn get_left_vector(&self) -> Vector3<f32> {
//         self.transform.get_left_vector()
//     }

//     pub fn get_rotation(&self) -> cgmath::Quaternion<f32> {
//         self.transform.rotation
//     }

//     pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
//         // First, create quaternions for rotation
//         let yaw_rotation = Quaternion::from_angle_y(Rad(delta_yaw));
//         let pitch_rotation = Quaternion::from_angle_x(Rad(delta_pitch));
    
//         // Combine rotations
//         let combined_rotation = yaw_rotation * self.transform.rotation * pitch_rotation;
    
//         // Update rotation
//         self.transform.rotation = combined_rotation;
    
//         // Clamp pitch to prevent extreme rotations
//         let pitch_limit = Rad(std::f32::consts::FRAC_PI_2 - 0.1); // Slightly less than 90 degrees
//         let current_pitch = self.get_pitch_angle();
    
//         if current_pitch.abs() > pitch_limit.0 {
//             // If pitch exceeds limit, project back to the limit
//             let corrected_rotation = if current_pitch > 0.0 {
//                 Quaternion::from_angle_x(pitch_limit)
//             } else {
//                 Quaternion::from_angle_x(-pitch_limit)
//             };
//             self.transform.rotation = corrected_rotation;
//         }
//     }

//     fn get_pitch_angle(&self) -> f32 {
//         let rotation_matrix: Matrix4<f32> = Matrix4::from(self.transform.rotation);
    
//         let forward = rotation_matrix.transform_vector(Vector3::new(0.0, 0.0, -1.0));
//         forward.y.asin()
//     }

//     //attach and detach to a partent function
//     pub fn attach_to(&mut self, parent: &WorldCoords, offset: Vector3<f32>) {
//         self.parent = Some(parent as *const WorldCoords);
//         self.follow_offset = Some(offset);
//     }

//     pub fn detach(&mut self) {
//         self.parent = None;
//         self.follow_offset = None;
//     }
// }

