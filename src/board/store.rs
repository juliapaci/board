use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;

use std::fs::{File, OpenOptions};
// use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

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
    pub fn create<P>(store_path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let store_path = store_path.as_ref();
        _ = std::fs::create_dir(&store_path);
        let store_file_path = store_path.join("store.store");
        if let Err(_) = File::create_new(&store_file_path) {
            std::fs::copy(&store_file_path, store_path.join("backup.store"))?;
        }

        Ok(Self {
            store: OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
                .open(&store_file_path)?,
            cache: store_file_path.join(".cache"),
        })
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        self.store.set_len(0)
    }

    pub fn read_line(
        &self,
        line: &str,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> std::io::Result<board::Item> {
        use board::Item;

        Ok(match serde_json::from_str(line)? {
            Item::Text(i) => Item::Text(i),
            Item::Image(i) => Item::Image(board::ItemImage::new(Box::new(
                rl.load_texture_from_image(thread, &Image::load_image("test.png").unwrap())
                    .expect("couldnt load texture"),
            ))),
        })
    }

    #[inline]
    pub fn add(&mut self, item: &board::Item) -> std::io::Result<()> {
        writeln!(
            self.store,
            "{}",
            serde_json::to_string(item).unwrap()
        )
    }
}
