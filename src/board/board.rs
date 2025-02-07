use std::io::Write;

use super::store::Store;
use ggez::{
    graphics::{self, Canvas, Color, DrawParam, Image, Rect, Text},
    input::{keyboard::KeyCode, mouse},
    mint::Point2,
    Context,
};
use reqwest;
use serde::{Deserialize, Serialize};

// position is in pixels

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemImage {
    /// path to cached item
    /// only optional because of serialisation
    #[serde(skip)]
    handle: Option<Image>,
    pub position: (f32, f32),
    pub scale: (f32, f32),
    pub rotation: f32,

    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemText {
    text: String,

    position: (f32, f32),
    scale: f32,
    rotation: f32,
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
    colours: (Color, Color),
}

impl BoardState {
    fn new() -> Self {
        Self {
            selected: None,
            colours: (crate::LIGHT, crate::DARK),
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
            items.push(match store.read_line(line, ctx) {
                Ok(i) => i,
                Err(e) => {
                    println!("line {line} couldnt be read: {e}");
                    continue;
                }
            })
        }

        ctx.gfx.add_font(
            "fancy font",
            graphics::FontData::from_path(ctx, "/fonts/MeowScript-Regular.ttf").unwrap(),
        );

        Ok(Self {
            store,
            items,

            state: BoardState::new(),
        })
    }

    #[inline]
    pub fn add_text(&mut self, text: String) {
        self.items.push(Item::Text(ItemText::new(text)));
    }

    #[inline]
    pub fn add_image(
        &mut self,
        url: &str,
        ctx: &Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.items.push(Item::Image(Self::image_from_url(
            &self.store,
            &Self::get_image_source_from_url(url)?,
            ctx,
        )?));

        Ok(())
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
                            x.handle().width() as _,
                            x.handle().height() as _,
                        ),
                        Item::Text(x) => {
                            let dim = x.text().measure(cc).unwrap();
                            Rect::new(x.position.0, x.position.1, dim.x, dim.y)
                        }
                    },
                    if i == self.state.selected.unwrap_or(i + 1) {
                        Color::from_rgb(168, 50, 84)
                    } else {
                        Color::from_rgb(209, 65, 86)
                    },
                )
                .expect("couldnt make the rectangle outline thing"),
                DrawParam::new(),
            )
        });
    }

    // gets the image of the greatest resolution
    // TODO: use a proper html parser and give options
    //       of possible images to the user
    fn get_image_source_from_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let body;

        match reqwest::blocking::get(url) {
            Ok(b) => body = b.text()?,
            Err(e) => return Err(Box::new(e)),
        };

        #[derive(Debug)]
        struct Image {
            url: String,
            resolution: (usize, usize),
        }
        use std::cmp::*;
        impl Ord for Image {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.resolution.cmp(&other.resolution)
            }
        }

        impl PartialOrd for Image {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for Image {
            fn eq(&self, other: &Self) -> bool {
                self.resolution == other.resolution
            }
        }

        impl Eq for Image {}

        let mut image = Image {
            url: "".to_owned(),
            resolution: (0, 0),
        };

        let img_regex = regex::Regex::new("<img.*>").unwrap();
        // backreferencing the delimiter would be optimal but alas i cant find a crate which supports it
        // ill prob end up implementing it on my own before switching to a real html parser
        let url_regex =
            regex::Regex::new(r#"src(\s)*=(\s)*("|')?(http(s?)://.+\.(jpg|jpeg|png))("|')"#)
                .unwrap();
        let width_regex = regex::Regex::new(r"width(\s)*=(\s)*.([0-9]+).").unwrap();
        let height_regex = regex::Regex::new(r"height(\s)*=(\s)*.([0-9]+).").unwrap();
        for m in img_regex.find_iter(&body) {
            let s = m.as_str();

            let Some(url) = url_regex.captures(s) else {
                continue;
            };
            let Some(width) = width_regex.captures(s) else {
                continue;
            };
            let Some(height) = height_regex.captures(s) else {
                continue;
            };

            image = image.max(Image {
                url: url[4].to_owned(),
                resolution: (
                    width[3].parse::<usize>().unwrap_or_default(),
                    height[3].parse::<usize>().unwrap_or_default(),
                ),
            });
        }

        Ok(image.url.to_owned())
    }

    #[inline]
    pub fn name_from_url(url: &str) -> &str {
        url.split('/').last().unwrap_or(url)
    }

    pub fn image_from_url(
        store: &Store,
        url: &str,
        ctx: &Context,
    ) -> Result<ItemImage, Box<dyn std::error::Error>> {
        let img_bytes = reqwest::blocking::get(url)?.bytes()?.to_vec();

        let path = store.cache.join(Self::name_from_url(url));
        let mut file = std::fs::File::create(&path)?;
        file.write(&img_bytes)?;

        ItemImage::from_path(store, url, ctx).map_err(Box::from)
    }

    pub fn set_selection(&mut self, selection: Option<usize>) {
        self.state.selected = match selection {
            Some(i) => {
                // push to last so it gets drawn ontop
                let last = self.items.len() - 1;
                self.items.swap(i, last);

                // self.state.selected = Some(i)
                Some(last)
            }
            None => None,
        }
    }

    /// index corresponding to the selected item
    pub fn select(&self, pos: (f32, f32), c: &Context) -> Option<usize> {
        // TODO: quadtree optimisations
        // TODO: take into account rotation

        /// r: (x, y, w, h)
        #[inline]
        fn inside(p: (f32, f32), r: (f32, f32, f32, f32)) -> bool {
            (p.0 >= r.0 && p.0 <= r.0 + r.2) && (p.1 >= r.1 && p.1 <= r.1 + r.3)
        }

        self.items.iter().position(|x| {
            inside(
                pos,
                match x {
                    Item::Image(i) => (
                        i.position.0,
                        i.position.1,
                        i.handle().width() as _,
                        i.handle().height() as _,
                    ),
                    Item::Text(i) => {
                        let dim = i.text().measure(c).unwrap();
                        (i.position.0, i.position.1, dim.x, dim.y)
                    }
                },
            )
        })
    }

    pub fn manage(&mut self, c: &Context) {
        if !c.mouse.button_pressed(mouse::MouseButton::Left) || self.state.selected.is_none() {
            self.state.selected = None;
            return;
        }

        #[inline]
        fn point2_to_tuple<T>(p: Point2<T>) -> (T, T) {
            (p.x, p.y)
        }

        #[inline]
        fn add_tuples<T: std::ops::Add<Output = T>>(a: (T, T), b: (T, T)) -> (T, T) {
            (a.0 + b.0, a.1 + b.1)
        }

        #[inline]
        fn div_tuple<T: std::ops::Div<Output = T> + std::marker::Copy>(t: (T, T), f: T) -> (T, T) {
            (t.0/f, t.1/f)
        }

        let mdelta = point2_to_tuple(c.mouse.delta());
        let item = &mut self.items[self.state.selected.unwrap()];

        // scale
        if c.keyboard.is_key_pressed(KeyCode::E) {
            match item {
                Item::Image(x) => x.scale = add_tuples(x.scale, div_tuple(mdelta, 100.0)),
                Item::Text(x) => x.scale += mdelta.0 + mdelta.1
            }
        }

        // rotation
        else if c.keyboard.is_key_pressed(KeyCode::R) {
            match item {
                Item::Image(x) => x.rotation += (mdelta.0 + mdelta.1)/180.,
                Item::Text(x) => x.rotation += (mdelta.0 + mdelta.1)/180.
            }
        }

        // position
        else {
            match item {
                Item::Image(x) => x.position = add_tuples(x.position, mdelta),
                Item::Text(x) => x.position = add_tuples(x.position, mdelta),
            }
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected
    }

    pub fn remove(&mut self, i: usize) {
        self.items.remove(i);
        self.state.selected = None;
    }

    pub fn get(&self, i: usize) -> Option<&Item> {
        self.items.get(i)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        self.store.clear()?;
        Ok(self.items.iter().for_each(|x| {
            if let Err(e) = self.store.add(x) {
                println!("Error saving an item ({x:?}): \"{e}\"");
            }
        }))
    }

    pub fn set_colours(&mut self, c: (Color, Color)) {
        self.state.set_colours(c);
    }
}

impl ItemImage {
    pub fn new(handle: Image, url: String) -> Self {
        Self {
            handle: Some(handle),
            position: (0., 0.),
            scale: (1., 1.),
            rotation: 0.,
            url,
        }
    }

    pub fn handle<'a>(&'a self) -> &'a Image {
        // this should never be None so its okay to unwrap
        self.handle.as_ref().unwrap()
    }

    fn draw(&self, c: &mut Canvas) {
        c.draw(
            self.handle(),
            DrawParam::new()
                .dest([self.position.0, self.position.1])
                .scale([self.scale.0, self.scale.1])
                .rotation(self.rotation)
                .color(Color::WHITE),
        );
    }

    pub fn from_path(store: &Store, url: &str, ctx: &Context) -> ggez::GameResult<Self> {
        Ok(ItemImage::new(
            graphics::Image::from_path(
                ctx,
                std::path::PathBuf::from("/")
                    .join(store.cache.clone())
                    .join(Board::name_from_url(url)),
            )?,
            url.to_owned(),
        ))
    }
}

impl ItemText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            position: (0., 0.),
            scale: 100.,
            rotation: 0.0
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
                .dest([self.position.0, self.position.1])
                .rotation(self.rotation)
                .color(colour)
        );
    }
}

impl Item {
    pub fn with_position(mut self, pos: (f32, f32)) -> Self {
        match self {
            Item::Text(ref mut i) => i.position = pos,
            Item::Image(ref mut i) => i.position = pos,
        }

        self
    }

    /// for [`Item::Text`] only the `scale.0` is used
    pub fn with_scale(mut self, scale: (f32, f32)) -> Self {
        match self {
            Item::Text(ref mut i) => i.scale = scale.0,
            Item::Image(ref mut i) => i.scale = scale
        }

        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        match self {
            Item::Text(ref mut i) => i.rotation = rotation,
            Item::Image(ref mut i) => i.rotation = rotation
        }

        self
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Item::Text(_) => "Text",
            Item::Image(_) => "Image"
        })
    }
}
