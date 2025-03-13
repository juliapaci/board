#[derive(Clone, Copy)]
pub struct Camera {
    pub centre: (f32, f32), // x, y
    pub zoom: f32,
    pub resolution: (f32, f32), // w, h
}

impl Camera {
    pub fn new(c: &ggez::Context) -> Self {
        let res = c.gfx.window().inner_size().to_logical(1.0);
        Self {
            centre: (0.0, 0.0),
            zoom: 1.0,
            resolution: (res.width, res.height),
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
        Ok(self.add_zoom(y / 100.0))
    }

    pub fn add_zoom(&mut self, by: f32) {
        self.zoom += by;
        self.zoom = self.zoom.max(0.0);
    }

    pub fn screen_to_world(&self, p: (f32, f32)) -> (f32, f32) {
        (p.0 + self.centre.0, p.1 + self.centre.1)
    }

    pub fn world_to_screen(&self, p: (f32, f32)) -> (f32, f32) {
        let view_offset = (p.0 - self.centre.0, p.1 - self.centre.1);
        let view_offset = (view_offset.0 * self.zoom, view_offset.1 * self.zoom);

        (
            view_offset.0 + self.resolution.0 / 2.0,
            view_offset.1 - (self.zoom + self.resolution.1 / 2.0)
        )
    }

    // TODO: this is broken currently
    /// rect `r`: (x, y, w, h)
    pub fn contains(&self, r: (f32, f32, f32, f32)) -> bool {
        (self.centre.0 >= r.0
            && self.centre.0 <= (r.0 + r.2) * self.resolution.0 as f32 * self.zoom)
            && (self.centre.1 >= r.1
                && self.centre.1 <= (r.1 + r.3) * self.resolution.1 as f32 * self.zoom)
            || true
    }
}
