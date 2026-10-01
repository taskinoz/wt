use crate::config::{self, AgentEntry, Config, LinkMode};
use crate::git;
use crate::prompt;
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(source, target)
}

#[derive(Args)]
pub struct NewArgs {
    /// Existing branch to check out into the new worktree (tab-completable).
    /// Omit this when creating a new branch with -b.
    branch: Option<String>,

    /// Create a new branch with this name instead of checking out an existing one
    #[arg(short = 'b', long = "branch")]
    new_branch: Option<String>,

    /// Base ref for the new branch (only used with -b). Defaults to HEAD.
    #[arg(long)]
    from: Option<String>,

    /// Explicit full path for the worktree, bypassing the default
    /// <worktree_dir>/<name> layout
    #[arg(short = 'p', long)]
    path: Option<String>,

    /// Override the short folder name (default: derived from the branch name)
    #[arg(short = 'n', long)]
    name: Option<String>,

    /// Comma-separated agent names to pre-populate, or "all" / "none"
    #[arg(short = 'a', long, value_delimiter = ',')]
    agents: Option<Vec<String>>,

    /// Comma-separated env file names to pre-populate, or "all" / "none"
    #[arg(short = 'e', long, value_delimiter = ',')]
    envs: Option<Vec<String>>,

    /// Copy agent/env files instead of symlinking (overrides config default)
    #[arg(long, conflicts_with = "link")]
    copy: bool,

    /// Symlink agent/env files instead of copying (overrides config default)
    #[arg(long)]
    link: bool,

    /// Skip interactive prompts (agent/env selection defaults to none unless -a/-e is given)
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: NewArgs) {
    let cfg = match config::load() {
        Ok(Some(c)) => c,
        Ok(None) => {
            println!("No config found yet — let's set one up first.\n");
            crate::cmd::config_cmd::wizard();
            config::load_or_default()
        }
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            std::process::exit(1);
        }
    };

    let repo_root = match git::repo_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let (branch_name, is_new_branch, from_ref) = match resolve_branch(&args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let short_name = args
        .name
        .clone()
        .unwrap_or_else(|| derive_short_name(&branch_name));

    let dest = match &args.path {
        Some(p) => config::expand_tilde(p),
        None => repo_root.join(&cfg.worktree_dir).join(&short_name),
    };

    if dest.exists() {
        eprintln!("error: destination already exists: {}", dest.display());
        std::process::exit(1);
    }

    if let Some(parent) = dest.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("error: failed to create {}: {e}", parent.display());
            std::process::exit(1);
        }
    }

    let result = if is_new_branch {
        git::worktree_add_new_branch(&dest, &branch_name, &from_ref)
    } else {
        git::worktree_add_existing(&dest, &branch_name)
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
    println!("Created worktree at {} (branch {})", dest.display(), branch_name);

    populate_entries(&cfg, &dest, &cfg.agents, "agent", &args.agents, args.yes, args.copy, args.link);
    populate_entries(&cfg, &dest, &cfg.envs, "env", &args.envs, args.yes, args.copy, args.link);
}

/// Returns (branch_name, is_new_branch, from_ref)
fn resolve_branch(args: &NewArgs) -> Result<(String, bool, String), String> {
    if let Some(new_branch) = &args.new_branch {
        if git::branch_exists(new_branch) {
            return Err(format!(
                "branch '{new_branch}' already exists; omit -b to check it out instead"
            ));
        }
        let from = args.from.clone().unwrap_or_else(|| "HEAD".to_string());
        return Ok((new_branch.clone(), true, from));
    }
    match &args.branch {
        Some(b) => Ok((b.clone(), false, String::new())),
        None => Err(
            "a branch name is required (tab-complete an existing branch, or pass -b <name> to create one)"
                .to_string(),
        ),
    }
}

/// Default short folder name: the last path segment of the branch,
/// e.g. "feature/login-page" -> "login-page".
fn derive_short_name(branch: &str) -> String {
    branch.rsplit('/').next().unwrap_or(branch).to_string()
}

/// Links or copies the selected entries (agent files, env files, ...) from
/// `entries` into `dest`. `kind` is just a noun for messages ("agent", "env").
pub fn populate_entries(
    cfg: &Config,
    dest: &PathBuf,
    entries: &[AgentEntry],
    kind: &str,
    selected_arg: &Option<Vec<String>>,
    yes: bool,
    copy: bool,
    link: bool,
) {
    let selected = select_entries(entries, kind, selected_arg, yes);
    if selected.is_empty() {
        return;
    }

    let link_mode = if copy {
        LinkMode::Copy
    } else if link {
        LinkMode::Symlink
    } else {
        cfg.link_mode
    };

    for entry in selected {
        let source = config::expand_tilde(&entry.source);
        let source = match fs::canonicalize(&source) {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "warning: skipping {kind} '{}': cannot read source {}: {e}",
                    entry.name,
                    source.display()
                );
                continue;
            }
        };
        let target_path = dest.join(&entry.target);
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "warning: skipping {kind} '{}': cannot create {}: {e}",
                    entry.name,
                    parent.display()
                );
                continue;
            }
        }
        // Remove any existing entry (stale symlink or old copy) first. Writing a
        // copy straight onto an existing symlink would follow it and overwrite
        // whatever it points at — e.g. the real source file, if re-running this
        // after a previous symlink run.
        if target_path.symlink_metadata().is_ok() {
            if let Err(e) = fs::remove_file(&target_path) {
                eprintln!(
                    "warning: skipping {kind} '{}': cannot replace existing {}: {e}",
                    entry.name,
                    target_path.display()
                );
                continue;
            }
        }
        let op_result = match link_mode {
            LinkMode::Symlink => symlink(&source, &target_path),
            LinkMode::Copy => fs::copy(&source, &target_path).map(|_| ()),
        };
        match op_result {
            Ok(()) => println!(
                "  {} {} -> {}",
                match link_mode {
                    LinkMode::Symlink => "linked",
                    LinkMode::Copy => "copied",
                },
                entry.name,
                target_path.display()
            ),
            Err(e) => eprintln!("warning: failed to set up {kind} '{}': {e}", entry.name),
        }
    }
}

fn select_entries(
    entries: &[AgentEntry],
    kind: &str,
    selected_arg: &Option<Vec<String>>,
    yes: bool,
) -> Vec<AgentEntry> {
    if entries.is_empty() {
        return Vec::new();
    }

    if let Some(names) = selected_arg {
        if names.iter().any(|n| n.eq_ignore_ascii_case("none")) {
            return Vec::new();
        }
        if names.iter().any(|n| n.eq_ignore_ascii_case("all")) {
            return entries.to_vec();
        }
        let mut out = Vec::new();
        for n in names {
            match entries.iter().find(|a| &a.name == n) {
                Some(a) => out.push(a.clone()),
                None => eprintln!("warning: no configured {kind} named '{n}', skipping"),
            }
        }
        return out;
    }

    if yes {
        return Vec::new();
    }

    let labels: Vec<String> = entries
        .iter()
        .map(|a| format!("{} ({})", a.name, a.target))
        .collect();
    let picks = prompt::multi_select(
        &format!("\nWhich {kind} files should this worktree get?"),
        &labels,
    );
    picks.into_iter().map(|i| entries[i].clone()).collect()
}
