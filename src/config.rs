use serde::Deserialize;
use std::path::Path;

/// Project topology — controls which files the scanner collects.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Topology {
    /// Single crate: scan own `ui/` or `src/` only. Default.
    #[default]
    Flat,
    /// Workspace: scan all `apps/*/ui/` and `apps/*/src/` from workspace root.
    Workspace,
}

const TOPOLOGY_WORKSPACE: &str = "workspace";
const CONFIG_DIR: &str = "proj";
const CONFIG_FILENAME: &str = "rulestools.toml";

impl Topology {
    fn from_str(s: &str) -> Self {
        if s.trim().to_lowercase() == TOPOLOGY_WORKSPACE { Self::Workspace } else { Self::Flat }
    }
}

/// Scanner configuration — controls which checks are enabled and whether violations block the build.
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
    pub exclude: Vec<String>,
    pub topology: Topology,
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
            exclude: Vec::new(),
            topology: Topology::Flat,
        }
    }
}

#[derive(Deserialize, Default)]
struct TomlRoot {
    #[serde(default)]
    slintscanners: Option<TomlScanners>,
    #[serde(default)]
    project: Option<TomlProject>,
}

#[derive(Deserialize, Default)]
struct TomlProject {
    topology: Option<String>,
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
    exclude: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from `proj/rulestools.toml`.
    /// Checks `manifest_dir` first (per-crate override), falls back to workspace `root`.
    pub fn load(root: &Path, manifest_dir: &Path) -> Self {
        let local = manifest_dir.join(CONFIG_DIR).join(CONFIG_FILENAME);
        let fallback = root.join(CONFIG_DIR).join(CONFIG_FILENAME);
        let toml_path = if local.is_file() { local } else { fallback };
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
                    if let Some(v) = s.exclude { cfg.exclude = v; }
                }
                if let Some(p) = parsed.project {
                    if let Some(t) = p.topology { cfg.topology = Topology::from_str(&t); }
                }
            }
        }

        cfg
    }
}
