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
}

pub fn run(args: ConfigArgs) {
    match args.action {
        None => wizard(),
        Some(ConfigAction::Show) => show(),
        Some(ConfigAction::AddAgent { name, source, target }) => add_agent(name, source, target),
        Some(ConfigAction::RemoveAgent { name }) => remove_agent(&name),
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
    let cfg = existing.unwrap_or_default();

    let worktree_dir = prompt::ask(
        "Default worktree folder (relative to repo root)",
        Some(&cfg.worktree_dir),
    );

    let default_mode_label = cfg.link_mode.label();
    let use_symlink = prompt::ask_bool(
        "Symlink agent files by default? (no = copy)",
        default_mode_label == "symlink",
    );
    let link_mode = if use_symlink { LinkMode::Symlink } else { LinkMode::Copy };

    let mut agents = cfg.agents.clone();
    println!();
    if !agents.is_empty() {
        println!("Existing agent files:");
        for a in &agents {
            println!("  - {} -> {} (source: {})", a.name, a.target, a.source);
        }
    }
    let add_more = prompt::ask_bool("Add an agent file now?", agents.is_empty());
    let mut adding = add_more;
    while adding {
        if let Some(entry) = prompt_new_agent() {
            agents.retain(|a| a.name != entry.name);
            agents.push(entry);
        }
        adding = prompt::ask_bool("Add another agent file?", false);
    }

    let new_cfg = Config { worktree_dir, link_mode, agents };
    match config::save(&new_cfg) {
        Ok(()) => println!("\nSaved config to {:?}", config::config_path()),
        Err(e) => eprintln!("Failed to save config: {e}"),
    }
}

fn prompt_new_agent() -> Option<AgentEntry> {
    let name = prompt::ask("  Agent name (short identifier)", None);
    if name.trim().is_empty() {
        return None;
    }
    let source = prompt::ask("  Source file path (e.g. your Obsidian note)", None);
    if source.trim().is_empty() {
        return None;
    }
    let default_target = std::path::Path::new(&source)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "AGENTS.md".to_string());
    let target = prompt::ask(
        "  Filename to create in the new worktree",
        Some(&default_target),
    );
    Some(AgentEntry { name, source, target })
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

fn add_agent(name: Option<String>, source: Option<String>, target: Option<String>) {
    let mut cfg = config::load_or_default();
    let name = name.unwrap_or_else(|| prompt::ask("Agent name (short identifier)", None));
    let source =
        source.unwrap_or_else(|| prompt::ask("Source file path (e.g. your Obsidian note)", None));
    let default_target = std::path::Path::new(&source)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "AGENTS.md".to_string());
    let target = target.unwrap_or_else(|| {
        prompt::ask("Filename to create in the new worktree", Some(&default_target))
    });

    cfg.agents.retain(|a| a.name != name);
    cfg.agents.push(AgentEntry { name: name.clone(), source, target });
    match config::save(&cfg) {
        Ok(()) => println!("Added agent '{name}'."),
        Err(e) => eprintln!("Failed to save config: {e}"),
    }
}

fn remove_agent(name: &str) {
    let mut cfg = config::load_or_default();
    let before = cfg.agents.len();
    cfg.agents.retain(|a| a.name != name);
    if cfg.agents.len() == before {
        eprintln!("No agent named '{name}' found.");
        return;
    }
    match config::save(&cfg) {
        Ok(()) => println!("Removed agent '{name}'."),
        Err(e) => eprintln!("Failed to save config: {e}"),
    }
}
