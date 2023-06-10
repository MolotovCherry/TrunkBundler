mod css;
mod js;

use std::{fs, path::PathBuf, sync::Mutex};

use js::process_js;

use crate::asset_manager::{AssetManager, AssetType};
use nipper::Document;

pub struct PipelineProcessor {
    debug: bool,
    document: Document,
    asset_manager: Mutex<AssetManager>,
    html_file: PathBuf,
}

impl PipelineProcessor {
    pub fn new(html_file: PathBuf, debug: bool) -> Self {
        let html = fs::read_to_string(&html_file).expect("Failed to read html file");

        Self {
            debug,
            document: Document::from(&html),
            asset_manager: Mutex::new(AssetManager::new()),
            html_file,
        }
    }

    pub fn run(&mut self) {
        self.process_pipelines();
        self.finalize();
    }

    fn process_pipelines(&mut self) {
        process_js(&mut self.document, &self.asset_manager, self.debug);
    }

    fn finalize(&self) {
        let mut asset_manager = self.asset_manager.lock().unwrap();

        // add processed html file now
        let asset = AssetType::Memory((*self.document.html()).as_bytes().into());
        asset_manager.add(asset, &self.html_file);

        asset_manager.dump().expect("Failed to dump assets");
    }
}
