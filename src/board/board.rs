use super::store::Store;
use raylib::prelude::*;
use std::io::{BufRead, BufReader};

// position is in pixels

trait Drawable {
    fn draw(&self, d: &mut RaylibDrawHandle);
}

pub struct ItemImage {
    /// path to cached item
    handle: &'static Texture2D,

    position: (f32, f32),
}

pub struct ItemText {
    text: String,
    size: i32,

    position: (f32, f32),
}

pub enum Item {
    Image(ItemImage),
    Text(ItemText),
}

impl Drawable for ItemImage {
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_texture(self.handle, self.position.0 as _, self.position.1 as _, Color::WHITE);
    }
}

impl Drawable for ItemText {
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_text(
            &self.text,
            self.position.0 as _,
            self.position.1 as _,
            self.size,
            Color::WHITE,
        );
    }
}

impl Item {
    fn draw(&self, d: &mut RaylibDrawHandle) {
        match self {
            Item::Image(x) => x.draw(d),
            Item::Text(x) => x.draw(d),
        };
    }
}

pub struct Board {
    store: Store,
    items: Vec<Item>,
}

impl Board {
    pub fn create(
        store_path: std::path::PathBuf,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> std::io::Result<Self> {
        let store = Store::create(store_path)?;
        let mut items = Vec::with_capacity(BufReader::new(&store.store).lines().count());

        items
            .iter_mut()
            .enumerate()
            .map(|(x, i)| *i = store.read_line(x as _, rl, thread).unwrap());

        Ok(Self { store, items })
    }
}

impl ItemImage {
    pub fn new(handle: &'static Texture2D) -> Self {
        Self {
            handle,
            position: (0., 0.)
        }
    }
}
