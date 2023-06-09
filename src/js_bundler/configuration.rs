use std::path::PathBuf;

use serde::Deserialize;
use swc_bundler::ModuleType as SwcModuleType;
use swc_common::collections::AHashMap;
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::Config;
use swc_ecma_minifier::option::{CompressOptions, MangleOptions, MinifyOptions, TopLevelOptions};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct SwcrcConfig {
    #[serde(default)]
    #[serde(rename = "moduleType")]
    pub module_type: ModuleType,
    pub output: Option<PathBuf>,
    #[serde(default = "always_true")]
    pub inline: bool,
    pub modules: Vec<String>,
    #[serde(rename = "externalModules")]
    pub external_modules: Option<Vec<String>>,
    #[serde(default)]
    pub minify: MinifyOptions,
    #[serde(default)]
    pub config: Config,
    pub env: Option<AHashMap<String, String>>,
    pub vars: Option<AHashMap<String, String>>,
    pub typeofs: Option<AHashMap<String, String>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub enum ModuleType {
    #[default]
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "iife")]
    Iife,
}
impl From<ModuleType> for SwcModuleType {
    fn from(value: ModuleType) -> Self {
        match value {
            ModuleType::Es => SwcModuleType::Es,
            ModuleType::Iife => SwcModuleType::Iife,
        }
    }
}

fn always_true() -> bool {
    true
}

pub fn generate_config(debug: bool) -> SwcrcConfig {
    let codegen_config = Config {
        target: EsVersion::latest(),
        minify: !debug,
        ..Default::default()
    };

    let mut config = SwcrcConfig {
        module_type: ModuleType::Iife,
        config: codegen_config,
        ..Default::default()
    };

    // set options for release build
    // will compress, mangle, and minify everything
    if !debug {
        config.minify = MinifyOptions {
            rename: true,
            compress: Some(CompressOptions {
                top_level: Some(TopLevelOptions { functions: true }),
                ..Default::default()
            }),
            mangle: Some(MangleOptions {
                top_level: Some(true),
                keep_class_names: false,
                keep_fn_names: false,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    config
}
