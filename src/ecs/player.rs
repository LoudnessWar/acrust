use crate::{graphics::camera::{Camera, CameraMode}, model::transform::WorldCoords};
use cgmath::{InnerSpace, Vector3};


//ok so I need a game object abstract class or smthing because this and camera lowkey have like the same functions
pub struct Player {
    pub transform: WorldCoords,
    pub speed: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, z:f32, rotation: f32) -> Self {
        Player {
            transform: WorldCoords::new(x, y ,z , rotation),
            speed: 0.1,
        }
    }
    
    pub fn new_coords(coords: WorldCoords) -> Self {
        Player {
            transform: coords,
            speed: 0.1,
        }
    }

    pub fn move_forward(&mut self, for_vec: Vector3<f32>) {
        self.transform.move_forward(for_vec, self.speed)
    }

    pub fn move_backward(&mut self, for_vec: Vector3<f32>) {
        self.transform.move_backward(for_vec, self.speed)
    }

    pub fn move_left(&mut self, left_vec: Vector3<f32>) {
        self.transform.move_left(left_vec, self.speed)
    }

    pub fn move_right(&mut self, left_vec: Vector3<f32>) {
        self.transform.move_right(left_vec, self.speed)
    }

    //ok theoretically the up vector is just the cross product of the left and forward vector
    //most games just let you move up or down... why dont I do that... WAIT bruh lol
    //where is my old player code lol
    // pub fn move_forward(&mut self, for_vec: Vector3<f32>) {
    //     self.transform.move_forward(for_vec, self.speed)
    // }

    pub fn move_up(&mut self) {
        self.transform.position += Vector3::new(0.0, self.speed, 0.0);
    }

    pub fn move_down(&mut self) {
        self.transform.position -= Vector3::new(0.0, self.speed, 0.0);
    }

    //gets tuff

    pub fn get_WorldCoords(&self) -> &WorldCoords{
        &self.transform
    }

    pub fn get_position(&self) -> &Vector3<f32>{
        self.transform.get_position()
    }

    //moving with camera... idk if I like this tbh
    pub fn move_forward_with_camera(&mut self, camera: &Camera) {
        let dir = camera.get_move_forward_vector();
        self.transform.position.x += dir.x * self.speed;
        self.transform.position.z += dir.z * self.speed;
    }
    
    pub fn move_backward_with_camera(&mut self, camera: &Camera) {
        let dir = camera.get_move_forward_vector();
        self.transform.position.x -= dir.x * self.speed;
        self.transform.position.z -= dir.z * self.speed;
    }
    
    pub fn move_left_with_camera(&mut self, camera: &Camera) {
        let dir = camera.get_move_left_vector();
        self.transform.position.x += dir.x * self.speed;
        self.transform.position.z += dir.z * self.speed;
    }
    
    pub fn move_right_with_camera(&mut self, camera: &Camera) {
        let dir = camera.get_move_left_vector();
        self.transform.position.x -= dir.x * self.speed;
        self.transform.position.z -= dir.z * self.speed;
    }
    // pub fn move_forward_with_camera(&mut self, camera: &Camera) {
    //     let direction = match camera.mode {
    //         CameraMode::FirstPerson => camera.get_forward_vector(),
    //         CameraMode::ThirdPerson => {
    //             // In third person, we want to move in the direction the camera is looking horizontally
    //             let forward = camera.get_forward_vector();
    //             // Project the forward vector onto the horizontal plane
    //             Vector3::new(forward.x, 0.0, forward.z).normalize()
    //         },
    //         _ => camera.get_forward_vector(),
    //     };
        
    //     self.transform.position.x += direction.x * self.speed;
    //     self.transform.position.z += direction.z * self.speed;
    // }
    
    // pub fn move_backward_with_camera(&mut self, camera: &Camera) {
    //     let direction = match camera.mode {
    //         CameraMode::FirstPerson => camera.get_forward_vector(),
    //         CameraMode::ThirdPerson => {
    //             // In third person, we want to move in the direction the camera is looking horizontally
    //             let forward = camera.get_forward_vector();
    //             // Project the forward vector onto the horizontal plane
    //             Vector3::new(forward.x, 0.0, forward.z).normalize()
    //         },
    //         _ => camera.get_forward_vector(),
    //     };
        
    //     self.transform.position.x -= direction.x * self.speed;
    //     self.transform.position.z -= direction.z * self.speed;
    // }
    
    // pub fn move_right_with_camera(&mut self, camera: &Camera) {
    //     let direction = match camera.mode {
    //         CameraMode::FirstPerson => camera.get_left_vector(),
    //         CameraMode::ThirdPerson => {
    //             // In third person, we want to strafe relative to the camera's orientation
    //             let left = camera.get_left_vector();
    //             // Project the left vector onto the horizontal plane
    //             Vector3::new(left.x, 0.0, left.z).normalize()//is this like slow TODO look into
    //         },
    //         _ => camera.get_left_vector(),
    //     };
        
    //     self.transform.position.x -= direction.x * self.speed;
    //     self.transform.position.z -= direction.z * self.speed;
    // }
    
    // pub fn move_left_with_camera(&mut self, camera: &Camera) {
    //     let direction = match camera.mode {
    //         CameraMode::FirstPerson => camera.get_left_vector(),
    //         CameraMode::ThirdPerson => {
    //             // In third person, we want to strafe relative to the camera's orientation
    //             let left = camera.get_left_vector();
    //             // Project the left vector onto the horizontal plane
    //             Vector3::new(left.x, 0.0, left.z).normalize()
    //         },
    //         _ => camera.get_left_vector(),
    //     };
        
    //     self.transform.position.x += direction.x * self.speed;
    //     self.transform.position.z += direction.z * self.speed;
    // }
}