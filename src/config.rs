use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LinkMode {
    Symlink,
    Copy,
}

impl LinkMode {
    pub fn label(&self) -> &'static str {
        match self {
            LinkMode::Symlink => "symlink",
            LinkMode::Copy => "copy",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentEntry {
    pub name: String,
    pub source: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_worktree_dir")]
    pub worktree_dir: String,
    #[serde(default = "default_link_mode")]
    pub link_mode: LinkMode,
    #[serde(default)]
    pub agents: Vec<AgentEntry>,
}

fn default_worktree_dir() -> String {
    ".worktree".to_string()
}

fn default_link_mode() -> LinkMode {
    LinkMode::Symlink
}

impl Default for Config {
    fn default() -> Self {
        Config {
            worktree_dir: default_worktree_dir(),
            link_mode: default_link_mode(),
            agents: Vec::new(),
        }
    }
}

pub fn config_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("wt").join("config.json");
        }
    }
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config").join("wt").join("config.json")
}

pub fn load() -> io::Result<Option<Config>> {
    let path = config_path();
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path)?;
    let cfg: Config = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{path:?}: {e}")))?;
    Ok(Some(cfg))
}

pub fn load_or_default() -> Config {
    load().ok().flatten().unwrap_or_default()
}

pub fn save(cfg: &Config) -> io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, data)
}

/// Expand a leading `~` to $HOME. Does not touch `~user` forms.
pub fn expand_tilde(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest);
        }
    } else if p == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(p)
}
