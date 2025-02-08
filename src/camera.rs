#[derive(Clone, Copy)]
pub struct Camera {
    pub centre: (f32, f32),
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            centre: (0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl Camera {
    // matches ggez::event::EventHandler
    pub fn mouse_wheel_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        Ok(self.zoom += y / 100.0)
    }

    pub fn screen_to_world(&self, p: (f32, f32), c: &ggez::Context) -> (f32, f32) {
        todo!();
    }

    // TODO: this is broken currently
    /// rect `r`: (x, y, w, h)
    pub fn contains(&self, r: (f32, f32, f32, f32), c: &ggez::Context) -> bool {
        (self.centre.0 >= r.0
            && self.centre.0
                <= (r.0 + r.2)
                    * c.gfx.window().inner_size().to_logical::<f32>(1.).width
                    * self.zoom)
            && (self.centre.1 >= r.1
                && self.centre.1
                    <= (r.1 + r.3)
                        * c.gfx.window().outer_size().to_logical::<f32>(1.).height
                        * self.zoom)
            || true
    }
}
