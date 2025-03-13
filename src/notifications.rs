// TODO: maybe different display/update stuff for different levels of urgency
pub trait Notification {
    fn update(&mut self, by: f32) -> bool;
    fn display(&self, canvas: &mut ggez::graphics::Canvas, colour: ggez::graphics::Color);
}

#[derive(Debug)]
pub struct Notifications<T: Notification + Default> {
    list: Vec<T>,
    colour: ggez::graphics::Color,
}

impl<T: Notification + Default> Default for Notifications<T> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            colour: ggez::graphics::Color::WHITE,
        }
    }
}

impl<T: Notification + Default> Notifications<T> {
    pub fn with_colour(colour: ggez::graphics::Color) -> Self {
        Self {
            list: Default::default(),
            colour,
        }
    }

    pub fn set_colour(&mut self, colour: ggez::graphics::Color) {
        self.colour = colour
    }
}

impl<T: Notification + Default> Notifications<T> {
    pub fn display_all(&self, canvas: &mut ggez::graphics::Canvas) {
        self.list
            .iter()
            .for_each(|n| n.display(canvas, self.colour))
    }

    pub fn update_all(&mut self, by: f32) {
        let mut to_remove: Vec<usize> = vec![];
        for (i, n) in self.list.iter_mut().enumerate() {
            if !n.update(by) {
                to_remove.push(i);
            }
        }

        for i in to_remove {
            self.list.remove(i);
        }
    }

    pub fn add(&mut self, n: T) {
        self.list.push(n)
    }
}

#[derive(Default, Debug)]
pub struct MyNotification {
    body: String,
    time: f32,
}

impl Notification for MyNotification {
    fn update(&mut self, by: f32) -> bool {
        self.time -= by;
        self.time >= 0.0
    }

    fn display(&self, canvas: &mut ggez::graphics::Canvas, colour: ggez::graphics::Color) {
        canvas.draw(
            ggez::graphics::Text::new(&self.body).set_scale(30.0),
            ggez::graphics::DrawParam::new()
                .color(colour)
                .dest([0.0, 0.0])
                .rotation(self.time),
        )
    }
}

impl MyNotification {
    pub fn new(body: String, time: f32) -> Self {
        Self { body, time }
    }
}
