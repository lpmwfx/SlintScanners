use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub enabled: bool,
    pub deny: bool,
    pub check_tokens: bool,
    pub check_strings: bool,
    pub check_structure: bool,
    pub check_events: bool,
    pub check_mother_child: bool,
    pub check_string_states: bool,
    pub check_architecture: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            deny: false,
            check_tokens: true,
            check_strings: true,
            check_structure: true,
            check_events: true,
            check_mother_child: true,
            check_string_states: true,
            check_architecture: true,
        }
    }
}

#[derive(Deserialize, Default)]
struct TomlRoot {
    #[serde(default)]
    slintscanners: Option<TomlScanners>,
}

#[derive(Deserialize, Default)]
struct TomlScanners {
    enabled: Option<bool>,
    deny: Option<bool>,
    tokens: Option<bool>,
    strings: Option<bool>,
    structure: Option<bool>,
    events: Option<bool>,
    mother_child: Option<bool>,
    string_states: Option<bool>,
    architecture: Option<bool>,
}

impl Config {
    pub fn load(project_root: &Path) -> Self {
        let toml_path = project_root.join("proj").join("rulestools.toml");
        let mut cfg = Self::default();

        if let Ok(content) = std::fs::read_to_string(&toml_path) {
            if let Ok(parsed) = toml::from_str::<TomlRoot>(&content) {
                if let Some(s) = parsed.slintscanners {
                    if let Some(v) = s.enabled { cfg.enabled = v; }
                    if let Some(v) = s.deny { cfg.deny = v; }
                    if let Some(v) = s.tokens { cfg.check_tokens = v; }
                    if let Some(v) = s.strings { cfg.check_strings = v; }
                    if let Some(v) = s.structure { cfg.check_structure = v; }
                    if let Some(v) = s.events { cfg.check_events = v; }
                    if let Some(v) = s.mother_child { cfg.check_mother_child = v; }
                    if let Some(v) = s.string_states { cfg.check_string_states = v; }
                    if let Some(v) = s.architecture { cfg.check_architecture = v; }
                }
            }
        }

        cfg
    }
}
