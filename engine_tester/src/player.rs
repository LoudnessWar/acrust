use acrust::input::transform::WorldCoords;
use cgmath::Vector3;


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
}