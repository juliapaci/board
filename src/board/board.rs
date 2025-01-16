use super::store::Store;
use ggez::{
    graphics::{self, Canvas, Color, DrawParam, Image, Rect, Text},
    input::mouse,
    mint::Point2,
    Context,
};
use serde::{Deserialize, Serialize};

// position is in pixels

type ImageHandle = Box<Image>;
fn empty_texture_handle() -> ImageHandle {
    todo!()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemImage {
    /// path to cached item
    #[serde(skip)]
    #[serde(default = "empty_texture_handle")]
    handle: ImageHandle,
    position: (f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemText {
    text: String,
    scale: f32,

    position: (f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Item {
    Image(ItemImage),
    Text(ItemText),
}

struct BoardState {
    // index of selected item in items array
    selected: Option<usize>,
    // (background, text) colours
    colours: (Color, Color)
}

impl BoardState {
    fn new() -> Self {
        Self {
            selected: None,
            colours: (crate::LIGHT, crate::DARK)
        }
    }

    pub fn set_colours(&mut self, c: (Color, Color)) {
        self.colours = c;
    }
}

pub struct Board {
    store: Store,
    items: Vec<Item>,

    state: BoardState,
}

impl Board {
    pub fn create<P: AsRef<std::path::Path>>(
        store_path: P,
        ctx: &mut Context,
    ) -> std::io::Result<Self> {
        let store_path = store_path.as_ref();
        let store = Store::create(store_path)?;
        let contents: Vec<String> = std::fs::read_to_string(&store_path.join("store.store"))?
            .lines()
            .map(String::from)
            .collect();

        let mut items: Vec<Item> = Vec::<Item>::with_capacity(contents.len());
        for line in contents.iter() {
            items.push(store.read_line(line, ctx)?)
        }

        // ctx.gfx.add_font(
        //     "fancy font",
        //     graphics::FontData::from_path(ctx, "fonts/MeowScript-Regular.ttf").unwrap()
        // );

        Ok(Self {
            store,
            items,

            state: BoardState::new()
        })
    }

    #[inline]
    pub fn add_text(&mut self, text: String) {
        self.items.push(Item::Text(ItemText::new(text)));
    }

    #[inline]
    pub fn add_image(&mut self, url: &str, ctx: &Context) {
        self.items.push(Item::Image(ItemImage::new(Box::new(
            graphics::Image::from_path(ctx, "test.png").expect("couldnt load texture"),
        ))));
    }

    pub fn draw(&self, c: &mut Canvas, cc: &Context) {
        self.items.iter().for_each(|x| match x {
            Item::Image(x) => x.draw(c),
            Item::Text(x) => x.draw(c, self.state.colours.1),
        });

        // debug rectangles
        self.items.iter().enumerate().for_each(|(i, x)| {
            c.draw(
                &graphics::Mesh::new_rectangle(
                    cc,
                    graphics::DrawMode::stroke(1.0),
                    match x {
                        Item::Image(x) => Rect::new(
                            x.position.0,
                            x.position.1,
                            x.handle.width() as _,
                            x.handle.height() as _,
                        ),
                        Item::Text(x) => {
                            let dim = x.text().measure(cc).unwrap();
                            Rect::new(x.position.0, x.position.1, dim.x, dim.y)
                        }
                    },
                    if i == self.state.selected.unwrap_or(i + 1) {Color::from_rgb(168, 50, 84)} else {Color::from_rgb(209, 65, 86)},
                )
                .expect("couldnt make the rectangle outline thing"),
                DrawParam::new(),
            )
        });
    }

    pub fn input(&mut self, c: &Context) {
        if !c.mouse.button_pressed(mouse::MouseButton::Left) {
            self.state.selected = None;
            return;
        }

        #[inline]
        fn point2_to_tuple<T>(p: Point2<T>) -> (T, T) {
            (p.x, p.y)
        }

        // TODO: quadtree optimisations
        if let None = self.state.selected {
            /// r: (x, y, w, h)
            #[inline]
            fn inside(p: (f32, f32), r: (f32, f32, f32, f32)) -> bool {
                (p.0 >= r.0 && p.0 <= r.0 + r.2) && (p.1 >= r.1 && p.1 <= r.1 + r.3)
            }

            match self.items.iter().position(|x| {
                inside(
                    point2_to_tuple::<f32>(c.mouse.position()),
                    match x {
                        Item::Image(i) => (
                            i.position.0,
                            i.position.1,
                            i.handle.width() as _,
                            i.handle.height() as _,
                        ),
                        Item::Text(i) => {
                            let dim = i.text().measure(c).unwrap();
                            (i.position.0, i.position.1, dim.x, dim.y)
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
        fn add_tuples(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
            (a.0 + b.0, a.1 + b.1)
        }

        let mdelta: (f32, f32) = point2_to_tuple::<f32>(c.mouse.delta());
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

    pub fn set_colours(&mut self, c: (Color, Color)) {
        self.state.set_colours(c);
    }
}

impl ItemImage {
    pub fn new(handle: Box<Image>) -> Self {
        Self {
            handle,
            position: (0., 0.),
        }
    }

    fn draw(&self, c: &mut Canvas) {
        c.draw(
            self.handle.as_ref(),
            DrawParam::new()
                .dest([self.position.0, self.position.1])
                .color(Color::WHITE),
        );
    }
}

impl ItemText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            scale: 100.,
            position: (0., 0.),
        }
    }

    #[inline]
    pub fn text(&self) -> Text {
        Text::new(&self.text).set_scale(self.scale).clone()
    }

    fn draw(&self, c: &mut Canvas, colour: Color) {
        c.draw(
            &self.text(),
            DrawParam::new()
                .color(colour)
                .dest([self.position.0, self.position.1]),
        );
    }
}
