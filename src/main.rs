mod asset_manager;
mod errors;
mod helpers;
mod js_bundler;
mod pipelines;

use pipelines::PipelineProcessor;

use std::{env, path::Path};

fn main() {
    let is_debug = env::var("TRUNK_PROFILE").unwrap() == "debug";

    // we need to get the html file from the staging directory, not from the source dir
    let html_file = env::var("TRUNK_HTML_FILE").unwrap();
    let html_file = Path::new(&html_file).file_name().unwrap();
    let staging_dir = env::var("TRUNK_STAGING_DIR").unwrap();
    let staging_dir = Path::new(&staging_dir);
    let html_file = staging_dir.join(html_file);

    PipelineProcessor::new(html_file, is_debug).run();
}
