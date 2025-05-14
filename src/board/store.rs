use std::io::Write;

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use super::board;

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
                    if self.is_cached(board::Board::name_from_path(i.kind.argument())) {
                        board::ItemImage::from_path(self, i.kind.argument(), c)
                            .or(Err("failed to load image from path"))?
                    } else {
                        board::ItemImage::image_from_url(self, i.kind.argument(), c)
                            .or(Err("get_image_from_url failed"))?
                    },
                )
                .with_position(i.position)
                .with_scale(i.scale)
                .with_rotation(i.rotation),
            },
        )
    }

    #[inline]
    pub fn add(&mut self, item: &board::Item) -> std::io::Result<()> {
        writeln!(self.store, "{}", serde_json::to_string(item).unwrap())
    }

    #[inline]
    pub fn is_cached(&self, name: &str) -> bool {
        self.cache.join(name).exists()
    }

    #[inline]
    pub fn remove_cached(&self, name: &str) -> std::io::Result<()> {
        if self.is_cached(name) {
            std::fs::remove_file(self.cache.join(name))
        } else {
            Ok(())
        }
    }
}
