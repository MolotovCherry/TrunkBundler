mod css;
mod js;

use std::{fs, path::PathBuf, sync::Mutex};

use js::process_js;

use crate::asset_manager::AssetManager;
use nipper::Document;

pub struct PipelineProcessor {
    debug: bool,
    document: Document,
    asset_manager: Mutex<AssetManager>,
}

impl PipelineProcessor {
    pub fn new(html_file: PathBuf, debug: bool) -> Self {
        let html = fs::read_to_string(html_file).expect("Failed to read html file");

        Self {
            debug,
            document: Document::from(&html),
            asset_manager: Mutex::new(AssetManager::new()),
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
        asset_manager.dump().expect("Failed to dump assets");
    }
}
