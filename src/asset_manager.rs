use std::path::Path;
use std::{fs, path::PathBuf};

use crate::{
    errors::{ApplicationError, Result},
    helpers::copy_dir,
};

#[allow(dead_code)]
#[derive(Debug)]
pub enum AssetType {
    // Source file to copy from
    Path(PathBuf),
    // Copy from in-memory instead
    Memory(Vec<u8>),
}

#[derive(Debug)]
pub struct Asset {
    pub destination: PathBuf,
    pub data: AssetType,
}

#[derive(Debug)]
pub struct AssetManager {
    pub assets: Vec<Asset>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self { assets: Vec::new() }
    }

    /// Destination must be absolute path
    pub fn add<P: AsRef<Path>>(&mut self, asset: AssetType, destination: P) {
        let destination = destination.as_ref();

        assert!(
            destination.is_absolute(),
            "Path `{}` needs be absolute",
            destination.display()
        );

        self.assets.push(Asset {
            destination: destination.to_owned(),
            data: asset,
        });
    }

    /// Any existing files will all be replaced, make sure this is what you want!
    pub fn dump(&mut self) -> Result<()> {
        for asset in self.assets.iter() {
            let parent = asset
                .destination
                .parent()
                .ok_or(ApplicationError::PathError)?;
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }

            assert!(
                parent.is_dir(),
                "parent `{}` is not a directory",
                parent.display()
            );

            match &asset.data {
                AssetType::Path(f) => {
                    if f.is_file() {
                        fs::copy(f, &asset.destination)?;
                    } else if f.is_dir() {
                        copy_dir(f, &asset.destination)?;
                    }
                }

                AssetType::Memory(m) => {
                    fs::write(&asset.destination, m)?;
                }
            }
        }

        self.assets.clear();

        Ok(())
    }
}
