use json_patch::Patch;
use k8s_openapi::api::core::v1::Pod;
use serde_json::Value;

pub struct PodTemplate {
    pub(crate) image: String,
    pub(crate) command: Vec<String>,
    pub(crate) args: Vec<String>,
    pub(crate) static_env: Vec<StaticEnvTemplate>,
    pub(crate) secret_env: Vec<SecretEnvTemplate>,
}

pub struct StaticEnvTemplate {
    pub(crate) key: String,
    pub(crate) value: String,
}

pub struct SecretEnvTemplate {
    pub(crate) secret: String,
    pub(crate) key: String,
}

pub struct PodBuilder {
    pub(crate) base: Value,
    pub(crate) patches: Patch,
}

impl PodBuilder {
    pub fn new(base: Value) -> Self {
        Self {
            base,
            patches: Patch::new(),
        }
    }

    pub fn add_patch(&mut self, patch: Patch) -> &mut Self {
        self.patches.0.extend(patch.0);
        self
    }

    pub fn build(&self) -> Result<Pod, serde_json::Error> {
        let mut pod = self.pod_template.base.clone();
        json_patch::patch(&mut pod, &self.patches).unwrap();
        serde_json::from_value(pod)
    }
}
