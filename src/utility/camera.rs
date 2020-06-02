use cgmath::{Vector2};
pub struct Camera {
    /// uniform scale of our quad(s)
    zoom: f32,
    /// change in zoom after 1 second of active zooming
    zoom_speed: f32,
    position: Vector2<f32>,
    pan_speed: f32,
}

impl Default for Camera {
    fn default() -> Camera {
        Self {
            zoom: 1.0,
            zoom_speed: 4.0,
            position: Vector2::new(0.0, 0.0),
            pan_speed: 10.0
        }
    }
}

impl Camera {
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    // TODO: direction can only be 1.0 or -1.0 find a way to enforce this
    pub fn modify_zoom(&mut self, delta_time: f64, direction: f32) {
        self.zoom += delta_time as f32 * self.zoom_speed * direction;
    }

    pub fn pan(&mut self, delta_time: f64, direction: Vector2<f32>) {

    }
}