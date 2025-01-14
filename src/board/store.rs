use raylib::prelude::*;

use std::fs::File;
// use std::os::unix::fs::FileExt;
use std::path::PathBuf;

use super::board;

pub struct StoreStructure {
    pub url: String,
    /// from the begining of the html, which img tag is the correct one
    /// TODO: some tag identifier
    pub img_id: u16,

    pub item: super::board::Item,
}

pub struct Store {
    /// file path listing all the items
    pub store: File,
    /// cache directory path with images of all cached items (name corrosponds to url)
    pub cache: PathBuf,
}

impl Store {
    pub fn create(store_path: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            store: File::open(store_path.join("store.store"))?,
            cache: store_path.join(".cache"),
        })
    }

    pub fn read_line(
        &self,
        index: u64,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> std::io::Result<board::Item> {
        // let mut line;
        // self.store.read_at(line, index)?;
        // let liter = line.iter();
        //
        // let ss = StoreStructure {
        //     url: String::from_utf8(
        //         line[0..liter
        //             .position(|c| *c == ' ' as _)
        //             .expect("incorrect store structure format")]
        //             .into(),
        //     )
        //     .unwrap(),
        //     img_id: format!(line[..liter
        //                 .position(|c| *c == ' ' as _)
        //                 .expect("incorrect store structure format")]
        // };

        Ok(board::Item::Image(board::ItemImage::new(
            rl.load_texture_from_image(thread, &Image::load_image("test.png").unwrap())
                .expect("couldnt load texture"),
        )))
    }
}
