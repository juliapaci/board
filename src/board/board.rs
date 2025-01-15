use super::store::Store;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};

// position is in pixels

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
    size: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ItemText {
    text: String,
    size: f32,

    position: (f32, f32),
}

#[derive(Serialize, Deserialize)]
pub enum Item {
    Image(ItemImage),
    Text(ItemText),
}

fn vector2_to_tuple(vec: Vector2) -> (f32, f32) {
    (vec.x, vec.y)
}

#[derive(Default)]
struct BoardState {
    // index of selected item in items array
    selected: Option<usize>,
}

pub struct Board {
    store: Store,
    items: Vec<Item>,
    font: Font,

    state: BoardState,
}

impl Board {
    pub fn create(
        store_path: std::path::PathBuf,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> std::io::Result<Self> {
        let store = Store::create(store_path)?;
        let items = Vec::with_capacity(BufReader::new(&store.store).lines().count())
            .iter()
            .map(|_: &Item| store.read_line(rl, thread).unwrap())
            .collect();

        Ok(Self {
            store,
            items,
            font: rl
                .load_font(
                    &thread,
                    std::path::PathBuf::new()
                        .join("..")
                        .join("fonts")
                        .join("MeowScript-Regular.ttf")
                        .into_os_string()
                        .to_str()
                        .unwrap(),
                )
                .unwrap(),

            state: BoardState::default(),
        })
    }

    #[inline]
    pub fn add_text(&mut self, text: String) {
        self.items.push(Item::Text(ItemText::new(text)));
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        self.items.iter().for_each(|x| match x {
            Item::Image(x) => x.draw(d),
            Item::Text(x) => x.draw(d, &self.font),
        });

        self.items.iter().for_each(|x| match x {
            Item::Image(x) => d.draw_rectangle_lines(
                x.position.0 as _,
                x.position.1 as _,
                x.size as _,
                x.size as _,
                Color::RED,
            ),
            Item::Text(x) => d.draw_rectangle_lines(
                x.position.0 as _,
                x.position.1 as _,
                d.measure_text(&x.text, x.size as _) as _,
                x.size as _,
                Color::RED,
            ),
        });
    }

    pub fn input(&mut self, d: &mut RaylibDrawHandle) {
        if !d.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
            self.state.selected = None;
            return;
        }

        let mpos = d.get_mouse_position();

        // TODO: quadtree optimisations
        if let None = self.state.selected {
            match self.items.iter().position(|x| {
                match x {
                    Item::Image(i) => Rectangle::new(i.position.0, i.position.1, i.size, i.size),
                    Item::Text(i) => Rectangle::new(i.position.0, i.position.1, d.measure_text(&i.text, i.size as _) as _, i.size),
                }
                .check_collision_point_rec(mpos)
            }) {
                Some(i) => self.state.selected = Some(i),
                None => return,
            }
        }

        match &mut self.items[self.state.selected.unwrap()] {
            Item::Image(x) => x.position = vector2_to_tuple(mpos),
            Item::Text(x) => x.position = vector2_to_tuple(mpos),
        }
    }

    pub fn save(&mut self) {
        self.items.iter().for_each(|x| {
            if let Err(x) = self.store.add(x) {
                println!("Error: \"{x}\"");
            }
        });
    }
}

impl ItemImage {
    pub fn new(handle: Box<Texture2D>) -> Self {
        Self {
            handle,
            position: (0., 0.),
            size: 0.0,
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_texture(
            self.handle.as_ref(),
            self.position.0 as _,
            self.position.1 as _,
            Color::WHITE,
        );
    }
}

impl ItemText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            size: 100.,
            position: (0., 0.),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, font: &Font) {
        d.draw_text_ex(
            font,
            &self.text,
            Vector2::new(self.position.0, self.position.1),
            self.size,
            1.0,
            Color::WHITE,
        );
    }
}
