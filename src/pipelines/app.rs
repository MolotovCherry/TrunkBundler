use std::{
    env,
    path::{Path, PathBuf},
    sync::Mutex,
};

use nipper::Document;
use swc_atoms::{js_word, JsWord};
use swc_bundler::ModuleRecord;
use swc_common::Span;
use swc_ecma_ast::EsVersion;
use swc_ecma_ast::KeyValueProp;

use crate::{
    asset_manager::{AssetManager, AssetType},
    js_bundler::{compile_js, generate_config, ModuleType},
};
use glob::glob;

use super::html::process_html;

pub fn process_app(
    html_file: &Path,
    document: &mut Document,
    asset_manager: &Mutex<AssetManager>,
    debug: bool,
) {
    // do processing of app files if in release mode
    if !debug {
        let processed_html = process_html(&document.html());

        let sel = document.select(r#"link[data-bundler][rel="app"]"#);
        let app = &*sel
            .attr("data-package")
            .expect("`data-app` attribute missing");

        // now process js
        let glob_dir = env::var("TRUNK_STAGING_DIR").unwrap();
        let glob_dir = Path::new(&glob_dir);
        let glob_dir = glob_dir.join(&format!("{app}*.js"));
        // workaround for glob, which doesn't like \\?\; also, cause path stripping won't strip \\?\
        let glob_dir = Path::new(
            glob_dir
                .as_os_str()
                .to_str()
                .unwrap()
                .strip_prefix(r#"\\?\"#)
                .unwrap(),
        );

        for entry in glob(glob_dir.to_str().unwrap()).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        let mut js_config = generate_config(debug);

                        let output_dir = path.parent().unwrap();
                        let module_name = path.file_stem().unwrap().to_str().unwrap().to_string();

                        js_config.module_type = ModuleType::Es;
                        js_config.modules = vec![module_name];
                        js_config.output = Some(PathBuf::from(path.file_name().unwrap()));

                        let compress_options = js_config.minify.compress.as_mut().unwrap();
                        compress_options.module = true;
                        compress_options.ecma = EsVersion::latest();

                        // ! swc currently breaks the output when minified, so we have no choice but to not minify for now
                        // TODO: Also, think about using a standalone compiled function instead, then we can avoid the hook
                        // ! https://github.com/swc-project/swc/issues/7513
                        js_config.config.minify = false;

                        compile_js(
                            output_dir,
                            output_dir,
                            js_config,
                            asset_manager,
                            debug,
                            Some(Box::new(ImportMetaFix)),
                        )
                        .expect("Failed to compile js");
                    } else {
                        panic!("Ambiguous path `{path:?}`\nPlease do not have ambiguous non-file `{{package}}-*.js` paths")
                    }
                }

                Err(e) => panic!("{:?}", e),
            }
        }

        let mut asset_manager = asset_manager.lock().unwrap();
        let asset = AssetType::Memory(processed_html.into_bytes());
        asset_manager.add(asset, html_file);
    }
}

pub struct ImportMetaFix;

impl swc_bundler::Hook for ImportMetaFix {
    fn get_import_meta_props(
        &self,
        span: Span,
        _: &ModuleRecord,
    ) -> std::result::Result<Vec<KeyValueProp>, anyhow::Error> {
        use swc_ecma_ast::*;

        // a fix for the import.meta.url generated in the app js
        Ok(vec![KeyValueProp {
            key: PropName::Ident(Ident::new(js_word!("url"), span)),
            value: Box::new(Expr::Ident(Ident::new(
                JsWord::from("import.meta.url"),
                span,
            ))),
        }])
    }
}
