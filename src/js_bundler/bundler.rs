use std::{
    collections::HashMap,
    error::Error,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use swc_atoms::JsWord;
use swc_bundler::Bundler;
use swc_common::{FileName, Mark, SourceMap, GLOBALS};
use swc_ecma_codegen::{
    text_writer::{omit_trailing_semi, JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_loader::{
    resolvers::{lru::CachingResolver, node::NodeModulesResolver},
    TargetEnv,
};
use swc_ecma_minifier::option::ExtraOptions;
use swc_ecma_transforms::fixer;
use swc_ecma_visit::VisitMutWith;

use walkdir::WalkDir;

use super::{
    loading::{Hook, Loader},
    SwcrcConfig,
};

use crate::{
    asset_manager::{AssetManager, AssetType},
    errors::{ApplicationError, Result},
    helpers::SliceExt,
};

fn find_first<P: AsRef<Path>>(paths: &[P]) -> Result<PathBuf> {
    let mut result = None;
    for path in paths {
        let path = path.as_ref();
        if path.is_file() {
            result = Some(path.to_owned());
            break;
        }
    }

    result.ok_or(ApplicationError::NotFound)
}

pub fn compile_js<P: AsRef<Path>>(
    root: P,
    output_dir: P,
    mut config: SwcrcConfig,
    asset_manager: &Mutex<AssetManager>,
    debug: bool,
    hook: Option<Box<dyn swc_bundler::Hook>>,
) -> Result<()> {
    let root = root.as_ref();
    assert!(
        root.is_dir(),
        "root path `{}` does not exist, or is not a directory",
        root.display()
    );

    // all needed directories get created on asset dump
    let output_dir = output_dir.as_ref();

    // set default config outputs if non-existent
    if config.output.is_none() {
        config.output = Some(PathBuf::from(if debug { "dist.js" } else { "dist.min.js" }));
    }

    //
    // Push external lib folders files for adding to output file
    //
    let mut external_libs = vec![];

    let libs_path = root.join("lib");

    for entry in WalkDir::new(&libs_path).into_iter().flatten() {
        let p = entry.path();
        let is_file = p.is_file();

        if is_file && p.extension().is_some_and(|e| e == "js") {
            external_libs.push(p.to_owned());
        }
    }
    //
    // End
    //

    let src_dir = root.join("src");

    let mut entries = HashMap::new();

    for module in config.modules.iter().map(|n| &**n) {
        // first try root dir, then src dir. Both paths are valid for a module path
        let file = find_first(&[
            root.join(&format!("{module}.js")),
            root.join(&format!("{module}.jsx")),
            root.join(&format!("{module}.ts")),
            root.join(&format!("{module}.tsx")),
            src_dir.join(&format!("{module}.js")),
            src_dir.join(&format!("{module}.jsx")),
            src_dir.join(&format!("{module}.ts")),
            src_dir.join(&format!("{module}.tsx")),
        ])
        .unwrap_or_else(|_| panic!("Module {module} not found"));

        entries.insert(module.to_owned(), FileName::Real(file));
    }

    let mut external_modules = vec![];
    if let Some(modules) = &config.external_modules {
        for module in modules {
            external_modules.push(JsWord::from(module.clone()));
        }
    }

    let output_bundle_js = output_dir.join(config.output.as_ref().unwrap());

    bundle_js(
        debug,
        config,
        entries,
        external_modules,
        external_libs,
        asset_manager,
        output_bundle_js,
        hook,
    )
}

#[allow(clippy::too_many_arguments)]
fn bundle_js(
    debug: bool,
    mut swc_config: SwcrcConfig,
    entries: HashMap<String, FileName>,
    external_modules: Vec<JsWord>,
    external_libs: Vec<PathBuf>,
    asset_manager: &Mutex<AssetManager>,
    output_bundle_js: PathBuf,
    hook: Option<Box<dyn swc_bundler::Hook>>,
) -> Result<()> {
    let cm = Arc::<SourceMap>::default();

    let minify_cfg = swc_config.minify;
    let minify = swc_config.config.minify;

    let globals = Box::default();
    let mut bundler = Bundler::new(
        &globals,
        cm.clone(),
        Loader {
            cm: cm.clone(),
            env: swc_config.env.take(),
            vars: swc_config.vars.take(),
            typeofs: swc_config.typeofs.take(),
            debug,
        },
        CachingResolver::new(
            4096,
            NodeModulesResolver::new(TargetEnv::Browser, Default::default(), true),
        ),
        swc_bundler::Config {
            require: true,
            disable_inliner: !swc_config.inline,
            external_modules,
            disable_fixer: minify,
            disable_hygiene: minify,
            disable_dce: false,
            module: swc_config.module_type.into(),
        },
        hook.unwrap_or(Box::new(Hook)),
    );

    let mut modules = bundler
        .bundle(entries)
        .map_err(Into::<Box<dyn Error>>::into)?;

    if minify {
        modules = modules
            .into_iter()
            .map(|mut b| {
                let cm = cm.clone();

                GLOBALS.set(&globals, || {
                    b.module = swc_ecma_minifier::optimize(
                        b.module.into(),
                        cm,
                        None,
                        None,
                        &minify_cfg,
                        &ExtraOptions {
                            unresolved_mark: Mark::new(),
                            top_level_mark: Mark::new(),
                        },
                    )
                    .expect_module();
                    b.module.visit_mut_with(&mut fixer(None));
                    b
                })
            })
            .collect();
    }

    // process final bundled js

    let mut output = Vec::new();

    let mut external_libs_peek = external_libs.iter().peekable();

    while let Some(lib) = external_libs_peek.next() {
        let mut file = std::fs::File::open(lib).unwrap();
        file.read_to_end(&mut output).unwrap();

        output.trim_end();

        if external_libs_peek.peek().is_some() {
            output.extend_from_slice(&[b'\n', b'\n']);
        } else {
            output.push(b'\n');
            // double newlines for modules
            if !modules.is_empty() {
                output.push(b'\n');
            }
        }
    }

    let mut modules_peek = modules.iter().peekable();
    while let Some(bundled) = modules_peek.next() {
        {
            let wr = JsWriter::new(cm.clone(), "\n", &mut output, None);
            let mut emitter = Emitter {
                cfg: swc_config.config,
                cm: cm.clone(),
                comments: None,
                wr: if minify {
                    Box::new(omit_trailing_semi(wr)) as Box<dyn WriteJs>
                } else {
                    Box::new(wr) as Box<dyn WriteJs>
                },
            };

            emitter.emit_module(&bundled.module).unwrap();
        }

        output.trim_end();

        if modules_peek.peek().is_some() {
            output.extend_from_slice(&[b'\n', b'\n']);
        } else {
            output.push(b'\n');
        }
    }

    let mut asset_manager = asset_manager.lock().unwrap();
    let asset = AssetType::Memory(output);
    asset_manager.add(asset, output_bundle_js);

    Ok(())
}
