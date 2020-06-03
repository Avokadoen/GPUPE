use std::vec::Vec;
use sdl2::keyboard::Keycode;

struct InputHandler {
    active_keys: Vec<Keycode>,
}

impl Default for InputHandler {
    fn default() -> InputHandler {
        Self {
            // preallocate some keys, probably wont be above 1o
            active_keys: Vec::with_capacity(10),
        }
    }
}

impl InputHandler {
    fn on_key_down(&mut self, key: Keycode) {
        self.active_keys.push(key);
    }

    fn on_key_up(&mut self, key: Keycode) {
        match self.active_keys.iter().position(|k| k == Keycode) {
            Some(n) => self.active_keys.swap_remove(n),
            None => (),
        };
    }
}