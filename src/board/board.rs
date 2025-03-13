use std::io::Write;

use crate::camera::Camera;

use super::store::Store;
use ggez::{
    event::MouseButton,
    graphics::{self, Canvas, Color, DrawParam, Image, Rect, Text},
    input::keyboard::KeyCode,
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

#[derive(Clone, Copy)]
pub enum Selectable {
    Item(usize), // for item management
    Board,       // for camera movements
}

struct BoardState {
    // in world coords
    last_press: (f32, f32),
    // index of selected item in items array
    selected: Option<Selectable>,
    // (background, text) colours
    colours: (Color, Color),
}

impl BoardState {
    fn new() -> Self {
        Self {
            last_press: (0.0, 0.0),
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

    pub camera: Camera,
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
            camera: Camera::new(&ctx),
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
        self.screen_iter(cc).for_each(|x| match x {
            Item::Image(x) => x.draw(self.camera, c),
            Item::Text(x) => x.draw(self.camera, c, self.state.colours.1),
        });
    }

    pub fn draw_bounds(&self, c: &mut Canvas, cc: &Context) {
        // debug rectangles
        self.screen_iter(cc).enumerate().for_each(|(i, x)| {
            c.draw(
                &graphics::Mesh::new_rectangle(
                    cc,
                    graphics::DrawMode::stroke(1.0),
                    Rect::new(
                        x.to_rect(self.camera, cc).0,
                        x.to_rect(self.camera, cc).1,
                        x.to_rect(self.camera, cc).2,
                        x.to_rect(self.camera, cc).3,
                    ),
                    if let Some(Selectable::Item(s)) = self.state.selected {
                        if s == i {
                            Color::from_rgb(168, 50, 84)
                        } else {
                            Color::from_rgb(209, 65, 86)
                        }
                    } else {
                        Color::from_rgb(209, 65, 86)
                    },
                )
                .expect("couldnt make the rectangle outline thing"),
                DrawParam::new(),
            )
        });
    }

    pub fn draw_selection_info(&self, c: &mut Canvas, cc: &Context) {
        if self.state.selected.is_none() {
            return;
        }

        let screen_last = self.camera.world_to_screen(self.state.last_press);
        let world_mouse = self
            .camera
            .screen_to_world((cc.mouse.position().x, cc.mouse.position().y));

        c.draw(
            &graphics::Mesh::new_line(
                cc,
                &[[screen_last.0, screen_last.1].into(), cc.mouse.position()],
                1.0,
                Color::RED,
            )
            .expect("couldnt draw the selection lines"),
            DrawParam::new(),
        );

        c.draw(
            &graphics::Mesh::new_line(
                cc,
                &[
                    [self.state.last_press.0, self.state.last_press.1].into(),
                    [world_mouse.0, world_mouse.1],
                ],
                1.0,
                Color::BLUE,
            )
            .expect("couldnt draw the selection lines"),
            DrawParam::new(),
        )
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

        #[derive(Debug, PartialOrd, PartialEq, Eq)]
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

        let mut image = Image {
            url: "".to_owned(),
            resolution: (0, 0),
        };

        let img_regex = regex::Regex::new("<img.*>").unwrap();
        // backreferencing the delimiter would be optimal but alas i cant find a crate which supports it
        // ill prob end up implementing it on my own before switching to a real html parser
        let url_regex =
            regex::Regex::new(r#"src(\s*)=(\s*)("|')?(http(s?)://.+\.(jpg|jpeg|png))("|')"#)
                .unwrap();
        let width_regex = regex::Regex::new(r"width(\s*)=(\s*).([0-9]+).").unwrap();
        let height_regex = regex::Regex::new(r"height(\s*)=(\s*).([0-9]+).").unwrap();
        for m in img_regex.find_iter(&body) {
            println!("\n");
            let s = m.as_str();

            println!("{s}");

            let Some(url) = url_regex.captures(s) else {
                println!("failed at url parsing");
                continue;
            };
            let url = &url[4];

            let width = if let Some(cwidth) = width_regex.captures(s) {
                cwidth[3].parse::<usize>().unwrap_or(0)
            } else {
                println!("failed at width parsing");
                0
            };
            let height = if let Some(cheight) = height_regex.captures(s) {
                cheight[3].parse::<usize>().unwrap_or(0)
            } else {
                println!("failed at height parsing");
                0
            };

            image = image.max(Image {
                url: url.to_owned(),
                resolution: (width, height),
            });

            println!("{}, {}, {}\n\n", url.to_owned(), width, height);
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

    pub fn set_selection(&mut self, selection: Selectable) {
        self.state.selected = match selection {
            Selectable::Item(i) => {
                // push to last so it gets drawn ontop
                let last = self.items.len() - 1;
                self.items.swap(i, last);

                // self.state.selected = Some(i)
                Some(Selectable::Item(last))
            }

            Selectable::Board => Some(selection),
        }
    }

    // iterater containing items only on screen
    fn screen_iter<'a>(&'a self, c: &'a Context) -> impl Iterator<Item = &'a Item> {
        self.items
            .iter()
            .filter(|i| self.camera.contains(i.to_rect(self.camera, c)))
    }

    /// index corresponding to the selected item
    pub fn select(&self, pos: (f32, f32), c: &Context) -> Option<usize> {
        // TODO: quadtree optimisations
        // TODO: take into account rotation

        /// rect `r`: (x, y, w, h)
        #[inline]
        fn inside(p: (f32, f32), r: (f32, f32, f32, f32)) -> bool {
            (p.0 >= r.0 && p.0 <= r.0 + r.2) && (p.1 >= r.1 && p.1 <= r.1 + r.3)
        }

        self.screen_iter(c)
            .position(|x| inside(pos, x.to_rect(self.camera, c)))
    }

    pub fn manage(&mut self, c: &Context) {
        if (!c.mouse.button_pressed(MouseButton::Left)
            && !c.mouse.button_pressed(MouseButton::Right))
            || self.state.selected.is_none()
        {
            self.state.selected = None;
            return;
        }

        #[inline]
        fn add_tuples<T: std::ops::Add<Output = T>>(a: (T, T), b: (T, T)) -> (T, T) {
            (a.0 + b.0, a.1 + b.1)
        }

        #[inline]
        fn sub_tuples<T: std::ops::Sub<Output = T>>(a: (T, T), b: (T, T)) -> (T, T) {
            (a.0 - b.0, a.1 - b.1)
        }

        #[inline]
        fn div_tuple<T: std::ops::Div<Output = T> + std::marker::Copy>(t: (T, T), f: T) -> (T, T) {
            (t.0 / f, t.1 / f)
        }

        let mdelta = (c.mouse.delta().x, c.mouse.delta().y);
        match self.state.selected.unwrap() {
            Selectable::Item(i) => {
                let item = &mut self.items[i];

                // scale
                if c.keyboard.is_key_pressed(KeyCode::E)
                    || (c.mouse.button_pressed(MouseButton::Left)
                        && c.mouse.button_pressed(MouseButton::Right))
                {
                    match item {
                        Item::Image(x) => x.scale = add_tuples(x.scale, div_tuple(mdelta, 100.0)),
                        Item::Text(x) => x.scale += mdelta.0 + mdelta.1,
                    }
                }
                // rotation
                else if c.keyboard.is_key_pressed(KeyCode::R)
                    || (c.mouse.button_pressed(MouseButton::Right)
                        && !c.mouse.button_pressed(MouseButton::Left))
                {
                    match item {
                        Item::Image(x) => x.rotation += (mdelta.0 + mdelta.1) / 180.,
                        Item::Text(x) => x.rotation += (mdelta.0 + mdelta.1) / 180.,
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

            Selectable::Board => {
                // zoom
                if c.mouse.button_pressed(MouseButton::Left)
                    && c.mouse.button_pressed(MouseButton::Right)
                {
                    self.camera.add_zoom((mdelta.0 + mdelta.1) / 100.0)
                }
                // pivot
                else if c.mouse.button_pressed(MouseButton::Left) {
                    self.camera.centre = add_tuples(self.camera.centre, mdelta)
                }
                // glide
                else if c.mouse.button_pressed(MouseButton::Right) {
                    self.camera.centre = add_tuples(
                        self.camera.centre,
                        div_tuple(
                            sub_tuples(
                                self.state.last_press,
                                (c.mouse.position().x, c.mouse.position().y),
                            ),
                            10.0,
                        ),
                    )
                }
            }
        }
    }

    pub fn selected(&self) -> Option<Selectable> {
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

    pub fn set_last_press(&mut self, p: (f32, f32)) {
        self.state.last_press = self.camera.screen_to_world(p);
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

    #[inline]
    pub fn world_position(&self, camera: Camera) -> (f32, f32) {
        (
            (self.position.0 + camera.centre.0) * camera.zoom,
            (self.position.1 + camera.centre.1) * camera.zoom,
        )
    }

    #[inline]
    pub fn world_scale(&self, camera: Camera) -> (f32, f32) {
        (self.scale.0 * camera.zoom, self.scale.1 * camera.zoom)
    }

    fn draw(&self, cam: Camera, c: &mut Canvas) {
        c.draw(
            self.handle(),
            DrawParam::new()
                .dest([self.world_position(cam).0, self.world_position(cam).1])
                .scale([self.world_scale(cam).0, self.world_scale(cam).1])
                .rotation(self.rotation)
                .color(Color::WHITE),
        );
    }
    pub fn to_rect(&self, cam: Camera) -> (f32, f32, f32, f32) {
        (
            self.world_position(cam).0,
            self.world_position(cam).1,
            self.world_scale(cam).0 * self.handle().width() as f32,
            self.world_scale(cam).1 * self.handle().height() as f32,
        )
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
            rotation: 0.0,
        }
    }

    #[inline]
    pub fn text(&self, camera: Camera) -> Text {
        Text::new(&self.text)
            .set_scale(self.scale * camera.zoom)
            .clone()
    }

    #[inline]
    pub fn world_position(&self, camera: Camera) -> (f32, f32) {
        (
            (self.position.0 + camera.centre.0) * camera.zoom,
            (self.position.1 + camera.centre.1) * camera.zoom,
        )
    }

    fn draw(&self, cam: Camera, c: &mut Canvas, colour: Color) {
        c.draw(
            &self.text(cam),
            DrawParam::new()
                .dest([self.world_position(cam).0, self.world_position(cam).1])
                .rotation(self.rotation)
                .color(colour),
        );
    }

    pub fn to_rect(&self, cam: Camera, c: &Context) -> (f32, f32, f32, f32) {
        let dim = self.text(cam).measure(c).unwrap();
        (
            self.world_position(cam).0,
            self.world_position(cam).1,
            dim.x,
            dim.y,
        )
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
            Item::Image(ref mut i) => i.scale = scale,
        }

        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        match self {
            Item::Text(ref mut i) => i.rotation = rotation,
            Item::Image(ref mut i) => i.rotation = rotation,
        }

        self
    }

    pub fn to_rect(&self, cam: Camera, c: &Context) -> (f32, f32, f32, f32) {
        match self {
            Self::Text(i) => i.to_rect(cam, c),
            Self::Image(i) => i.to_rect(cam),
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Item::Text(_) => "Text",
                Item::Image(_) => "Image",
            }
        )
    }
}
