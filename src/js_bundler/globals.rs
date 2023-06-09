use swc::config::{GlobalInliningPassEnvs, GlobalPassOption};
use swc_atoms::JsWord;
use swc_common::{collections::AHashMap, errors::Handler};
use swc_ecma_visit::Fold;

#[derive(Debug, Default)]
pub struct Variables {
    vars: AHashMap<JsWord, JsWord>,
    envs: AHashMap<JsWord, JsWord>,
    typeofs: AHashMap<JsWord, JsWord>,
}

impl Variables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_var<S: AsRef<str>>(&mut self, name: S, value: S) -> &mut Self {
        self.vars.insert(
            JsWord::from(name.as_ref().to_string()),
            JsWord::from(value.as_ref().to_string()),
        );
        self
    }

    pub fn add_env<S: AsRef<str>>(&mut self, name: S, value: S) -> &mut Self {
        self.envs.insert(
            JsWord::from(name.as_ref().to_string()),
            JsWord::from(value.as_ref().to_string()),
        );
        self
    }

    #[allow(unused)]
    pub fn add_typeof<S: AsRef<str>>(&mut self, name: S, value: S) -> &mut Self {
        self.typeofs.insert(
            JsWord::from(name.as_ref().to_string()),
            JsWord::from(value.as_ref().to_string()),
        );
        self
    }

    pub fn add_vars(&mut self, map: &AHashMap<String, String>) -> &mut Self {
        self.vars.extend(
            map.iter()
                .map(|(name, value)| (JsWord::from(name.clone()), JsWord::from(value.clone()))),
        );
        self
    }

    pub fn add_envs(&mut self, map: &AHashMap<String, String>) -> &mut Self {
        self.envs.extend(
            map.iter()
                .map(|(name, value)| (JsWord::from(name.clone()), JsWord::from(value.clone()))),
        );
        self
    }

    pub fn add_typeofs(&mut self, map: &AHashMap<String, String>) -> &mut Self {
        self.typeofs.extend(
            map.iter()
                .map(|(name, value)| (JsWord::from(name.clone()), JsWord::from(value.clone()))),
        );
        self
    }

    pub fn build(self, handler: &Handler) -> impl Fold {
        let mut global_pass = GlobalPassOption::default();

        let envs = GlobalInliningPassEnvs::Map(self.envs);
        global_pass.envs = envs;

        global_pass.vars.extend(self.vars);

        global_pass.typeofs = self.typeofs;

        global_pass.build(&Default::default(), handler)
    }
}
