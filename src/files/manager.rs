use std::path::PathBuf;

use anyhow::{Result, anyhow};
use log::*;

use super::usb;

/// This FileManager only writes images
pub struct FileManager {
    write_location: PathBuf,
    max_index: usize,
}

impl FileManager {
    pub fn new(location: PathBuf) -> Result<FileManager> {
        let dir_entries = std::fs::read_dir(&location)?;

        let mut max_index = 0;
        for dir_entry in dir_entries {
            let dir_entry = dir_entry?;
            if !dir_entry.path().is_file() {
                continue;
            }

            let file_name = dir_entry.file_name();
            let path = &std::path::Path::new(&file_name);
            let basename = path.file_stem().unwrap();
            let basename = basename.to_str().unwrap();
            if !basename.starts_with("image") {
                continue;
            }

            let image_index = basename.strip_prefix("image").unwrap();
            let index: usize = match image_index.parse::<usize>() {
                Ok(index) => index,
                Err(err) => {
                    warn!("Error while parsing image index '{}': {:?}", image_index, err);
                    continue;
                },
            };

            max_index = std::cmp::max(max_index, index);
        }

        Ok(FileManager {
            write_location: location,
            max_index
        })
    }

    pub fn default() -> Result<FileManager> {
        Self::new(usb::StorageDevices::collect()
            .drives()
            .first()
            .ok_or_else(|| anyhow!("No USB device connected"))?
            .mount_point()
            .to_path_buf()
        )
    }

    pub fn next_image_location(&mut self, ext: &str) -> PathBuf {
        self.max_index += 1;
        let path = self.write_location.join(format!("image{}.{}", self.max_index, ext));
        return path;
    }

    pub fn write_loc_exists(&self) -> bool {
        self.write_location.exists()
    }
}
