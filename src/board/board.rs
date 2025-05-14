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
pub enum ImageType {
    Web(String),    // url (from a web page)
    Online(String), // url (directly an image)
    // TODO: not sure if local argument should hold the cached location (so just the name) or the actual path. probs the absolute path since we can always infer the cache location but idk
    Local(String), // path
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemImage {
    /// path to cached item
    /// only optional because of serialisation
    #[serde(skip)]
    handle: Option<Image>,
    pub position: (f32, f32),
    pub scale: (f32, f32),
    pub rotation: f32,

    pub kind: ImageType,
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
    /// for item management
    Item(usize),
    /// for camera movements
    Board,
}

struct BoardState {
    /// in world coords
    last_press: (f32, f32),
    /// index of selected item in items array
    selected: Option<Selectable>,
    /// (background, text) colours
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
    // TODO: prob should just use `Image` instead of `ItemImage`
    choices: Vec<(Option<ItemImage>, String)>,

    pub camera: Camera,
    state: BoardState,
}

impl ImageType {
    #[inline]
    /// returns empty argument if no matches
    pub fn type_from_argument(argument: &str) -> Self {
        if argument.starts_with("http") {
            if argument.ends_with(".png")
                || argument.ends_with(".jpg")
                || argument.ends_with(".jpeg")
                || argument.ends_with(".gif")
            {
                ImageType::Online(argument.to_owned())
            } else {
                ImageType::Web(argument.to_owned())
            }
        } else if argument.starts_with("/") {
            ImageType::Local(argument.to_owned())
        } else {
            ImageType::Local("".to_owned())
        }
    }

    #[inline]
    pub fn argument(&self) -> &str {
        match self {
            ImageType::Web(url) => &url,
            ImageType::Online(url) => &url,
            ImageType::Local(path) => &path,
        }
    }
}

impl Board {
    const CHOICE_AMOUNT: usize = 4;

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
            choices: Vec::default(),

            state: BoardState::new(),
            camera: Camera::new(&ctx),
        })
    }

    #[inline]
    pub fn add_text(&mut self, text: String) {
        self.items.push(Item::Text(ItemText::new(text)));
    }

    pub fn add_image(
        &mut self,
        kind: ImageType,
        ctx: &Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match kind {
            ImageType::Web(url) => {
                self.add_choices_from_url(&url)?;
                self.add_choices_images(ctx);
            }
            ImageType::Online(url) => {
                self.items.push(Item::Image(ItemImage::image_from_url(
                    &self.store,
                    &url,
                    ctx,
                )?));
            }
            ImageType::Local(path) => {
                self.items.push(Item::Image(ItemImage::image_from_path(
                    &self.store,
                    &path,
                    ctx,
                )?));
            }
        }

        Ok(())
    }

    /// adds the images for each choice up to [`Self::CHOICE_AMOUNT`]
    pub fn add_choices_images(&mut self, ctx: &Context) {
        for c in self
            .choices
            .iter_mut()
            .skip_while(|c| c.0.is_some())
            .take(Self::CHOICE_AMOUNT)
        {
            *c = (
                ItemImage::image_from_url(&self.store, &c.1, ctx).ok(),
                c.1.clone(),
            )
        }
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

        let screen_last = self.last_press_screen();
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

    /// gets a list of images from a web source and adds their urls to the choice list
    fn add_choices_from_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let body = reqwest::blocking::get(url)?.text()?;
        let img_regex = regex::Regex::new("<img.*>").unwrap();
        // [4] would be the url
        let url_regex =
            regex::Regex::new(r#"src(\s*)=(\s*)("|')((http(s?):)?//(\?|.[^("|')])+)("|') "#)
                .unwrap();

        for m in img_regex.find_iter(&body) {
            let Some(url) = url_regex.captures(m.as_str()) else {
                println!("failed at url parsing for: {}", m.as_str());
                continue;
            };

            println!("url: {}", url[4].to_owned());
            self.choices.push((None, url[4].to_owned()));
        }

        Ok(())
    }

    #[inline]
    pub fn name_from_path(path: &str) -> &str {
        path.split('/').last().unwrap_or(path)
        // sometimes urls have params so we dont want those
        // &name[0..name.find('?').unwrap_or(name.len()-1)]
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

        let mdelta = div_tuple((c.mouse.delta().x, c.mouse.delta().y), self.camera.zoom);
        match self.state.selected.unwrap() {
            Selectable::Item(i) => {
                let item = &mut self.items[i];

                // TODO: like proper square scale if shift is held
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
                                self.camera.screen_to_world(div_tuple(
                                    (c.mouse.position().x, c.mouse.position().y),
                                    self.camera.zoom
                                ))
                            ),
                            10.0,
                        ),
                    )
                }
            }
        }
    }

    pub fn last_press_screen(&self) -> (f32, f32) {
        self.camera.world_to_screen(self.state.last_press)
    }

    pub fn selected(&self) -> Option<Selectable> {
        self.state.selected
    }

    pub fn remove(&mut self, i: usize) -> std::io::Result<()> {
        let item = &self.items[i];
        match item {
            Item::Image(i) => self.store.remove_cached(i.kind.argument())?,
            _ => {}
        }

        self.items.remove(i);
        self.state.selected = None;

        Ok(())
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
    pub fn new(handle: Image, argument: &str) -> Self {
        Self {
            handle: Some(handle),
            position: (0., 0.),
            scale: (1., 1.),
            rotation: 0.,
            kind: ImageType::type_from_argument(argument),
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

    /// downloads image from url, caches it in our storage, gives a handle to it
    pub fn image_from_url(
        store: &Store,
        url: &str,
        ctx: &Context,
    ) -> Result<ItemImage, Box<dyn std::error::Error>> {
        let img_bytes = reqwest::blocking::get(url)?.bytes()?.to_vec();

        let path = store.cache.join(Board::name_from_path(url));
        let mut file = std::fs::File::create(&path)?;
        file.write(&img_bytes)?;

        ItemImage::from_path(store, url, ctx).map_err(Box::from)
    }

    pub fn image_from_path(
        store: &Store,
        path: &str,
        ctx: &Context,
    ) -> Result<ItemImage, Box<dyn std::error::Error>> {
        let cache_path = store.cache.join(Board::name_from_path(path));
        std::fs::copy(path, cache_path)?;

        ItemImage::from_path(store, path, ctx).map_err(Box::from)
    }

    pub fn from_path(store: &Store, argument: &str, ctx: &Context) -> ggez::GameResult<Self> {
        Ok(ItemImage::new(
            graphics::Image::from_path(
                ctx,
                std::path::PathBuf::from("/")
                    .join(store.cache.clone())
                    .join(Board::name_from_path(argument)),
            )?,
            argument,
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
