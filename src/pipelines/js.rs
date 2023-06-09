use std::path::PathBuf;
use std::{env, path::Path};
use std::{fs, sync::Mutex};

use nipper::Document;
use rayon::prelude::*;

use crate::{
    asset_manager::AssetManager,
    js_bundler::{compile_js, generate_config},
};

#[derive(Debug)]
struct JsProject {
    js_root: PathBuf,
    modules: Vec<String>,
    external_modules: Option<Vec<String>>,
    output: PathBuf,
    selection: usize,
}

pub fn process_js(document: &mut Document, asset_manager: &Mutex<AssetManager>, debug: bool) {
    let sel = document.select(r#"link[data-bundler][rel="js"]"#);
    let mut selections = sel.iter().collect::<Vec<_>>();

    let source_dir = env::var("TRUNK_SOURCE_DIR").unwrap();
    let source_dir = Path::new(&source_dir);

    let staging_dir = env::var("TRUNK_STAGING_DIR").unwrap();
    let staging_dir = Path::new(&staging_dir);

    let mut projects = vec![];
    for (i, selection) in selections.iter().enumerate() {
        let href = &*selection.attr("href").expect("`href` attr not found");
        let js_root = fs::canonicalize(source_dir.join(href)).expect("Failed to canonicalize path");

        let modules = selection
            .attr("data-modules")
            .expect("`data-modules` not found");
        let modules = modules
            .split_terminator(',')
            .map(|m| m.trim().to_owned())
            .collect::<Vec<_>>();

        let external_modules = selection.attr("data-external-modules").map(|s| {
            s.split_terminator(',')
                .map(|m| m.trim().to_owned())
                .collect::<Vec<_>>()
        });

        let output = selection
            .attr("data-output")
            .map(|t| PathBuf::from(&*t))
            .unwrap_or(PathBuf::from(if debug { "dist.js" } else { "dist.min.js" }));

        projects.push(JsProject {
            js_root,
            modules,
            external_modules,
            output,
            selection: i,
        });
    }

    let outputs_to_selections = projects
        .iter()
        .map(|p| (p.output.clone(), p.selection))
        .collect::<Vec<_>>();

    // parallelize js compilation
    projects.into_par_iter().panic_fuse().for_each(|p| {
        let mut js_config = generate_config(debug);
        js_config.modules = p.modules;
        js_config.external_modules = p.external_modules;
        js_config.output = Some(p.output);

        compile_js(
            p.js_root,
            staging_dir.to_path_buf(),
            js_config,
            asset_manager,
            debug,
            None,
        )
        .expect("Failed to compile js");
    });

    // replace tags in html
    for (output, i_selection) in outputs_to_selections.into_iter() {
        let selection = &mut selections[i_selection];
        let output = output.display().to_string();
        // ensure we don't do any double //
        let output = output.strip_prefix('/').unwrap_or(&output);

        let public_url = env::var("TRUNK_PUBLIC_URL").unwrap();
        selection.replace_with_html(format!(r#"<script src="{public_url}{output}"></script>"#));
    }
}
