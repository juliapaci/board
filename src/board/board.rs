use super::store::Store;
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

// position is in pixels

type TextureHandle = Box<Texture2D>;
fn empty_texture_handle() -> TextureHandle {
    todo!()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemImage {
    /// path to cached item
    #[serde(skip)]
    #[serde(default = "empty_texture_handle")]
    handle: TextureHandle,

    position: (f32, f32),
    size: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemText {
    text: String,
    size: f32,

    position: (f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Item {
    Image(ItemImage),
    Text(ItemText),
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
    pub fn create<P: AsRef<std::path::Path>>(
        store_path: P,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> std::io::Result<Self>
    {
        let store_path = store_path.as_ref();
        let mut store = Store::create(store_path)?;
        let contents: Vec<String> = std::fs::read_to_string(&store_path.join("store.store"))?
            .lines()
            .map(String::from)
            .collect();
        store.clear()?;

        let mut items: Vec<Item> = Vec::<Item>::with_capacity(contents.len());
        for line in contents.iter() {
            items.push(store.read_line(line, rl, thread).unwrap())
        }

        Ok(Self {
            store,
            items,
            font: rl
                .load_font(
                    &thread,
                    std::path::PathBuf::new()
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

        // TODO: quadtree optimisations
        if let None = self.state.selected {
            match self.items.iter().position(|x| {
                match x {
                    Item::Image(i) => Rectangle::new(i.position.0, i.position.1, i.size, i.size),
                    Item::Text(i) => Rectangle::new(
                        i.position.0,
                        i.position.1,
                        d.measure_text(&i.text, i.size as _) as _,
                        i.size,
                    ),
                }
                .check_collision_point_rec(d.get_mouse_position())
            }) {
                Some(i) => self.state.selected = Some(i),
                None => return,
            }
        }

        #[inline]
        fn vector2_to_tuple(vec: Vector2) -> (f32, f32) {
            (vec.x, vec.y)
        }
        #[inline]
        fn add_tuples(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
            (a.0 + b.0, a.1 + b.1)
        }

        let mdelta = vector2_to_tuple(d.get_mouse_delta());
        match &mut self.items[self.state.selected.unwrap()] {
            Item::Image(x) => x.position = add_tuples(x.position, mdelta),
            Item::Text(x) => x.position = add_tuples(x.position, mdelta),
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
