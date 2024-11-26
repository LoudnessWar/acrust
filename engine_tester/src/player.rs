use acrust::input::transform::WorldCoords;
use cgmath::Vector3;

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

    pub fn move_forward(&mut self) {
        self.transform.position += Vector3::new(0.0, 0.0, -self.speed);
    }

    pub fn move_backward(&mut self) {
        self.transform.position -= Vector3::new(0.0, 0.0, -self.speed);
    }

    pub fn move_left(&mut self) {
        self.transform.position -= Vector3::new(self.speed, 0.0, 0.0);
    }

    pub fn move_right(&mut self) {
        self.transform.position += Vector3::new(self.speed, 0.0, 0.0);
    }
}