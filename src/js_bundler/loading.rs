use swc_atoms::js_word;
use swc_bundler::{Load, ModuleData, ModuleRecord};
use swc_common::{
    collections::AHashMap,
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, Mark, SourceMap, Span,
};
use swc_ecma_ast::{EsVersion, KeyValueProp};
use swc_ecma_parser::{parse_file_as_module, EsConfig, Syntax, TsConfig};
use swc_ecma_visit::FoldWith;

use super::globals::Variables;

pub struct Hook;

impl swc_bundler::Hook for Hook {
    fn get_import_meta_props(
        &self,
        span: Span,
        module_record: &ModuleRecord,
    ) -> std::result::Result<Vec<KeyValueProp>, anyhow::Error> {
        use swc_ecma_ast::*;

        let file_name = module_record.file_name.to_string();

        Ok(vec![
            KeyValueProp {
                key: PropName::Ident(Ident::new(js_word!("url"), span)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span,
                    raw: None,
                    value: file_name.into(),
                }))),
            },
            KeyValueProp {
                key: PropName::Ident(Ident::new(js_word!("main"), span)),
                value: Box::new(if module_record.is_entry {
                    Expr::Member(MemberExpr {
                        span,
                        obj: Box::new(Expr::MetaProp(MetaPropExpr {
                            span,
                            kind: MetaPropKind::ImportMeta,
                        })),
                        prop: MemberProp::Ident(Ident::new(js_word!("main"), span)),
                    })
                } else {
                    Expr::Lit(Lit::Bool(Bool { span, value: false }))
                }),
            },
        ])
    }
}

pub struct Loader {
    pub debug: bool,
    pub cm: Lrc<SourceMap>,
    pub env: Option<AHashMap<String, String>>,
    pub vars: Option<AHashMap<String, String>>,
    pub typeofs: Option<AHashMap<String, String>>,
}

impl Load for Loader {
    fn load(&self, f: &FileName) -> std::result::Result<ModuleData, anyhow::Error> {
        let (fm, is_jsx, is_ts, is_tsx) = match f {
            FileName::Real(path) => {
                let is_jsx = path
                    .extension()
                    .is_some_and(|e| e.to_ascii_lowercase() == "jsx");
                let is_ts = path
                    .extension()
                    .is_some_and(|e| e.to_ascii_lowercase() == "ts");
                let is_tsx = path
                    .extension()
                    .is_some_and(|e| e.to_ascii_lowercase() == "tsx");

                (self.cm.load_file(path)?, is_jsx, is_ts, is_tsx)
            }
            _ => unreachable!(),
        };

        let config = if is_ts || is_tsx {
            Syntax::Typescript(TsConfig {
                tsx: is_tsx,
                decorators: true,
                ..Default::default()
            })
        } else {
            Syntax::Es(EsConfig {
                jsx: is_jsx,
                decorators: true,
                decorators_before_export: true,
                import_assertions: true,
                ..Default::default()
            })
        };

        let handler =
            Handler::with_tty_emitter(ColorConfig::Always, false, false, Some(self.cm.clone()));

        let mut module = parse_file_as_module(&fm, config, EsVersion::latest(), None, &mut vec![])
            .unwrap_or_else(|err| {
                err.into_diagnostic(&handler).emit();
                std::process::exit(0);
            });

        if is_jsx || is_tsx {
            let mut jsx_folder = swc_ecma_transforms_react::jsx::<SingleThreadedComments>(
                Default::default(),
                None,
                swc_ecma_transforms_react::Options {
                    development: Some(self.debug),
                    ..Default::default()
                },
                Mark::new(),
                Mark::new(),
            );
            module = module.fold_with(&mut jsx_folder);
        }

        // Do env var stuff

        let node_env = if self.debug {
            "'development'"
        } else {
            "'production'"
        };

        let mut variables = Variables::new();
        variables.add_env("NODE_ENV", node_env);
        variables.add_var("__DEBUG__", if self.debug { "true" } else { "false" });

        if let Some(envs) = &self.env {
            variables.add_envs(envs);
        }

        if let Some(var) = &self.vars {
            variables.add_vars(var);
        }

        if let Some(typeof_) = &self.typeofs {
            variables.add_typeofs(typeof_);
        }

        let mut global_pass = variables.build(&handler);
        module = module.fold_with(&mut global_pass);

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}
