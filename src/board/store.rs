use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use std::fs::{File, OpenOptions};
// use std::os::unix::fs::FileExt;
use std::path::PathBuf;

use super::board;

#[derive(Serialize, Deserialize)]
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
        _ = std::fs::create_dir(&store_path);
        let store_path = store_path.join("store.store");
        if let Ok(_) = File::create_new(&store_path) {
            std::fs::write(&store_path, "");
        }

        Ok(Self {
            store: OpenOptions::new().read(true).write(true).append(true).open(&store_path)?,
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

        Ok(board::Item::Image(board::ItemImage::new(Box::new(
            rl.load_texture_from_image(thread, &Image::load_image("test.png").unwrap())
                .expect("couldnt load texture"),
        ))))
    }
}
