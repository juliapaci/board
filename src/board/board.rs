use super::store::Store;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Seek, SeekFrom};

// position is in pixels

trait Drawable {
    fn draw(&self, d: &mut RaylibDrawHandle);
}

type TextureHandle = Box<Texture2D>;
fn empty_texture_handle() -> TextureHandle {
    todo!()
}

#[derive(Serialize, Deserialize)]
pub struct ItemImage {
    /// path to cached item
    #[serde(skip)]
    #[serde(default = "empty_texture_handle")]
    handle: TextureHandle,

    position: (f32, f32),
}

#[derive(Serialize, Deserialize)]
pub struct ItemText {
    text: String,
    size: i32,

    position: (f32, f32),
}

#[derive(Serialize, Deserialize)]
pub enum Item {
    Image(ItemImage),
    Text(ItemText),
}

impl Drawable for ItemImage {
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_texture(
            self.handle.as_ref(),
            self.position.0 as _,
            self.position.1 as _,
            Color::WHITE,
        );
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

        items.push(Item::Text(ItemText::new("tesslkdajsdljt".to_owned())));
        items.push(Item::Text(ItemText::new("lkndsc".to_owned())));

        Ok(Self { store, items })
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        self.items.iter().for_each(|x| x.draw(d));
    }
}

impl ItemImage {
    pub fn new(handle: Box<Texture2D>) -> Self {
        Self {
            handle,
            position: (0., 0.),
        }
    }
}

impl ItemText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            size: 0,
            position: (0., 0.)
        }
    }
}
