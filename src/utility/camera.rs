pub struct Camera {
    /// uniform scale of our quad(s)
    zoom: f64,
    /// change in zoom after 1 second of active zooming
    zoom_speed: f64,
    position: [f32; 2],
    pan_speed: f32,
}

pub enum Direction {
    In = 1,
    Out = -1,
}

impl Default for Camera {
    fn default() -> Camera {
        Self {
            zoom: 1.0,
            zoom_speed: 0.5,
            position: [0.0, 0.0],
            pan_speed: 10.0
        }
    }
}

impl Camera {
    pub fn zoom(&mut self, delta_time: f64, direction: Direction) {
        self.zoom += delta_time * self.zoom_speed;
    }
}