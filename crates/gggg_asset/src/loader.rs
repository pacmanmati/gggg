use std::{collections::HashMap, fs::File, io::Read, path::Path};

use anyhow::anyhow;

#[derive(Clone)]
pub struct Asset {
    pub bytes: Vec<u8>,
}

pub struct Loader {
    assets: HashMap<String, Asset>,
}

impl Loader {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn get_font_by_family(&self) -> anyhow::Result<Asset> {
        let path = String::from("/");
        let asset = self
            .assets
            .get(&path)
            .ok_or(anyhow!("Path doesn't exist in asset map."))?;
        Ok(asset.clone())
    }

    pub fn load<S: AsRef<Path>>(&mut self, path: S) -> anyhow::Result<()> {
        let mut file = File::open(path.as_ref())?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        self.assets
            .insert(path.as_ref().to_str().unwrap().to_string(), Asset { bytes });
        Ok(())
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}
