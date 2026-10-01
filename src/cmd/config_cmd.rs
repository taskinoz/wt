use crate::config::{self, AgentEntry, Config, LinkMode};
use crate::prompt;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the current configuration and its file path
    Show,
    /// Add an agent file entry
    AddAgent {
        /// Short name used to select this agent (e.g. "backend")
        name: Option<String>,
        /// Path to the source agent file (e.g. your Obsidian note)
        source: Option<String>,
        /// Filename to create in the worktree (e.g. "CLAUDE.md")
        target: Option<String>,
    },
    /// Remove an agent file entry by name
    RemoveAgent { name: String },
    /// Add an env file entry
    AddEnv {
        /// Short name used to select this env file (e.g. "backend")
        name: Option<String>,
        /// Path to the source env file
        source: Option<String>,
        /// Filename to create in the worktree (e.g. ".env")
        target: Option<String>,
    },
    /// Remove an env file entry by name
    RemoveEnv { name: String },
}

pub fn run(args: ConfigArgs) {
    match args.action {
        None => wizard(),
        Some(ConfigAction::Show) => show(),
        Some(ConfigAction::AddAgent { name, source, target }) => {
            with_saved_config(|cfg| add_entry(&mut cfg.agents, "agent", name, source, target, "AGENTS.md"))
        }
        Some(ConfigAction::RemoveAgent { name }) => {
            with_saved_config(|cfg| remove_entry(&mut cfg.agents, "agent", &name))
        }
        Some(ConfigAction::AddEnv { name, source, target }) => {
            with_saved_config(|cfg| add_entry(&mut cfg.envs, "env", name, source, target, ".env"))
        }
        Some(ConfigAction::RemoveEnv { name }) => {
            with_saved_config(|cfg| remove_entry(&mut cfg.envs, "env", &name))
        }
    }
}

/// Runs on first use, and any time the user runs `wt config` with no
/// subcommand to reconfigure.
pub fn wizard() {
    let existing = config::load().ok().flatten();
    if existing.is_some() {
        println!("Reconfiguring wt (existing settings shown as defaults).\n");
    } else {
        println!("First time setup for wt.\n");
    }
    let mut cfg = existing.unwrap_or_default();

    cfg.worktree_dir = prompt::ask(
        "Default worktree folder (relative to repo root)",
        Some(&cfg.worktree_dir),
    );

    let use_symlink = prompt::ask_bool(
        "Symlink agent/env files by default? (no = copy)",
        cfg.link_mode.label() == "symlink",
    );
    cfg.link_mode = if use_symlink { LinkMode::Symlink } else { LinkMode::Copy };

    wizard_entries(&mut cfg.agents, "agent", "AGENTS.md");
    wizard_entries(&mut cfg.envs, "env", ".env");

    match config::save(&cfg) {
        Ok(()) => println!("\nSaved config to {:?}", config::config_path()),
        Err(e) => eprintln!("Failed to save config: {e}"),
    }
}

fn wizard_entries(entries: &mut Vec<AgentEntry>, kind: &str, default_target_fallback: &str) {
    println!();
    if !entries.is_empty() {
        println!("Existing {kind} files:");
        for e in entries.iter() {
            println!("  - {} -> {} (source: {})", e.name, e.target, e.source);
        }
    }
    let mut adding = prompt::ask_bool(&format!("Add an {kind} file now?"), entries.is_empty());
    while adding {
        if let Some(entry) = prompt_new_entry(kind, default_target_fallback) {
            entries.retain(|e| e.name != entry.name);
            entries.push(entry);
        }
        adding = prompt::ask_bool(&format!("Add another {kind} file?"), false);
    }
}

fn prompt_new_entry(kind: &str, default_target_fallback: &str) -> Option<AgentEntry> {
    let name = prompt::ask(&format!("  {kind} name (short identifier)"), None);
    if name.trim().is_empty() {
        return None;
    }
    let source = prompt::ask("  Source file path", None);
    if source.trim().is_empty() {
        return None;
    }
    let target = prompt::ask(
        "  Filename to create in the new worktree",
        Some(&default_target_name(&source, default_target_fallback)),
    );
    Some(AgentEntry { name, source, target })
}

fn default_target_name(source: &str, fallback: &str) -> String {
    std::path::Path::new(source)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn show() {
    let path = config::config_path();
    match config::load() {
        Ok(Some(cfg)) => {
            println!("Config file: {path:?}\n");
            println!("{}", serde_json::to_string_pretty(&cfg).unwrap());
        }
        Ok(None) => {
            println!("No config found at {path:?}. Run `wt config` to set one up.");
        }
        Err(e) => eprintln!("Failed to read config: {e}"),
    }
}

fn with_saved_config(f: impl FnOnce(&mut Config)) {
    let mut cfg = config::load_or_default();
    f(&mut cfg);
    if let Err(e) = config::save(&cfg) {
        eprintln!("Failed to save config: {e}");
    }
}

fn add_entry(
    entries: &mut Vec<AgentEntry>,
    kind: &str,
    name: Option<String>,
    source: Option<String>,
    target: Option<String>,
    default_target_fallback: &str,
) {
    let name = name.unwrap_or_else(|| prompt::ask(&format!("{kind} name (short identifier)"), None));
    let source = source.unwrap_or_else(|| prompt::ask("Source file path", None));
    let target = target.unwrap_or_else(|| {
        prompt::ask(
            "Filename to create in the new worktree",
            Some(&default_target_name(&source, default_target_fallback)),
        )
    });

    entries.retain(|e| e.name != name);
    entries.push(AgentEntry { name: name.clone(), source, target });
    println!("Added {kind} '{name}'.");
}

fn remove_entry(entries: &mut Vec<AgentEntry>, kind: &str, name: &str) {
    let before = entries.len();
    entries.retain(|e| e.name != name);
    if entries.len() == before {
        eprintln!("No {kind} named '{name}' found.");
        return;
    }
    println!("Removed {kind} '{name}'.");
}
