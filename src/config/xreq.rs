use crate::{LoadConfig, RequestProfile};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ValidateConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, RequestProfile>,
}

impl RequestConfig {
    pub fn new(profiles: HashMap<String, RequestProfile>) -> Self {
        Self { profiles }
    }
    pub fn get_profile(&self, name: &str) -> Option<&RequestProfile> {
        self.profiles.get(name)
    }
}

impl LoadConfig for RequestConfig {}
impl ValidateConfig for RequestConfig {
    fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }
        Ok(())
    }
}
