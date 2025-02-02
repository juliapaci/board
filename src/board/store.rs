use serde::{Deserialize, Serialize};
use std::io::Write;

use std::fs::{File, OpenOptions};
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
        let cache_path = store_path.join(".cache");
        _ = std::fs::create_dir(&store_path);
        _ = std::fs::create_dir(&cache_path);

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
            cache: cache_path,
        })
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        self.store.set_len(0)
    }

    pub fn read_line(&self, line: &str, c: &ggez::Context) -> Result<board::Item, String> {
        use board::Item;

        Ok(
            match serde_json::from_str(line).or(Err("from_str failed"))? {
                Item::Text(i) => Item::Text(i),
                Item::Image(i) => Item::Image(
                    board::Board::image_from_url(self, &i.url, c)
                        .or(Err("get_image_from_url failed"))?.with_position(i.position),
                ),
            },
        )
    }

    #[inline]
    pub fn add(&mut self, item: &board::Item) -> std::io::Result<()> {
        writeln!(self.store, "{}", serde_json::to_string(item).unwrap())
    }
}
