use super::store::Store;
use macroquad::prelude::*;
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
    pub async fn create<P: AsRef<std::path::Path>>(store_path: P) -> std::io::Result<Self> {
        let store_path = store_path.as_ref();
        let store = Store::create(store_path)?;
        let contents: Vec<String> = std::fs::read_to_string(&store_path.join("store.store"))?
            .lines()
            .map(String::from)
            .collect();

        let mut items: Vec<Item> = Vec::<Item>::with_capacity(contents.len());
        for line in contents.iter() {
            items.push(store.read_line(line).await.unwrap())
        }

        Ok(Self {
            store,
            items,
            font: load_ttf_font(
                std::path::PathBuf::new()
                    .join("fonts")
                    .join("MeowScript-Regular.ttf")
                    .into_os_string()
                    .to_str()
                    .unwrap(),
            )
            .await
            .unwrap(),

            state: BoardState::default(),
        })
    }

    #[inline]
    pub fn add_text(&mut self, text: String) {
        self.items.push(Item::Text(ItemText::new(text)));
    }

    #[inline]
    pub async fn add_image(&mut self, url: &str) {
        self.items.push(Item::Image(ItemImage::new(Box::new(
            load_texture("test.png")
                .await
                .expect("couldnt load texture"),
        ))));
    }

    pub fn draw(&self) {
        self.items.iter().for_each(|x| match x {
            Item::Image(x) => x.draw(),
            Item::Text(x) => x.draw(&self.font),
        });

        // debug rectangles
        self.items.iter().for_each(|x| match x {
            Item::Image(x) => {
                draw_rectangle_lines(x.position.0, x.position.1, x.size, x.size, 1.0, RED)
            }
            Item::Text(x) => {
                let dim = measure_text(&x.text, Some(&self.font), 1, x.size);
                draw_rectangle_lines(x.position.0, x.position.1, dim.width, dim.height, 1.0, RED);
            }
        });
    }

    pub fn input(&mut self) {
        if !is_mouse_button_down(MouseButton::Left) {
            self.state.selected = None;
            return;
        }

        // TODO: quadtree optimisations
        if let None = self.state.selected {
            /// r: (x, y, w, h)
            fn inside(p: (f32, f32), r: (f32, f32, f32, f32)) -> bool {
                (p.0 >= r.0 && p.0 <= r.0 + r.2) && (p.1 >= r.1 && p.1 <= r.1 + r.3)
            }

            match self.items.iter().position(|x| {
                inside(
                    mouse_position(),
                    match x {
                        Item::Image(i) => (i.position.0, i.position.1, i.size, i.size),
                        Item::Text(i) => {
                            let dim = measure_text(&i.text, Some(&self.font), 1, i.size);
                            (i.position.0, i.position.1, dim.width, dim.height)
                        }
                    },
                )
            }) {
                Some(i) => {
                    // push to last so it gets drawn ontop
                    let last = self.items.len() - 1;
                    self.items.swap(i, last);

                    // self.state.selected = Some(i)
                    self.state.selected = Some(last)
                }
                None => return,
            }
        }

        #[inline]
        fn vec2_to_tuple(vec: Vec2) -> (f32, f32) {
            (vec.x, vec.y)
        }

        #[inline]
        fn add_tuples(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
            (a.0 + b.0, a.1 + b.1)
        }

        let mdelta = vec2_to_tuple(mouse_delta_position());
        match &mut self.items[self.state.selected.unwrap()] {
            Item::Image(x) => x.position = add_tuples(x.position, mdelta),
            Item::Text(x) => x.position = add_tuples(x.position, mdelta),
        }
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        self.store.clear()?;
        Ok(self.items.iter().for_each(|x| {
            if let Err(x) = self.store.add(x) {
                println!("Error: \"{x}\"");
            }
        }))
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

    fn draw(&self) {
        draw_texture(
            self.handle.as_ref(),
            self.position.0,
            self.position.1,
            WHITE,
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

    fn draw(&self, font: &Font) {
        draw_text_ex(
            &self.text,
            self.position.0,
            self.position.1,
            TextParams {
                font: Some(font),
                font_scale: self.size,
                color: WHITE,
                ..Default::default()
            },
        );
    }
}
