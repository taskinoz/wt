use crate::config::{self, AgentEntry, Config, LinkMode};
use crate::git;
use crate::prompt;
use clap::Args;
use std::fs;
use std::path::PathBuf;

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

    /// Copy agent files instead of symlinking (overrides config default)
    #[arg(long, conflicts_with = "link")]
    copy: bool,

    /// Symlink agent files instead of copying (overrides config default)
    #[arg(long)]
    link: bool,

    /// Skip interactive prompts (agent selection defaults to none unless -a is given)
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

    populate_agents(&cfg, &dest, &args.agents, args.yes, args.copy, args.link);
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

pub fn populate_agents(
    cfg: &Config,
    dest: &PathBuf,
    agents_arg: &Option<Vec<String>>,
    yes: bool,
    copy: bool,
    link: bool,
) {
    let selected = select_agents(cfg, agents_arg, yes);
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

    for agent in selected {
        let source = config::expand_tilde(&agent.source);
        let source = match fs::canonicalize(&source) {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "warning: skipping agent '{}': cannot read source {}: {e}",
                    agent.name,
                    source.display()
                );
                continue;
            }
        };
        let target_path = dest.join(&agent.target);
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "warning: skipping agent '{}': cannot create {}: {e}",
                    agent.name,
                    parent.display()
                );
                continue;
            }
        }
        let op_result = match link_mode {
            LinkMode::Symlink => std::os::unix::fs::symlink(&source, &target_path),
            LinkMode::Copy => fs::copy(&source, &target_path).map(|_| ()),
        };
        match op_result {
            Ok(()) => println!(
                "  {} {} -> {}",
                match link_mode {
                    LinkMode::Symlink => "linked",
                    LinkMode::Copy => "copied",
                },
                agent.name,
                target_path.display()
            ),
            Err(e) => eprintln!("warning: failed to set up agent '{}': {e}", agent.name),
        }
    }
}

fn select_agents(cfg: &Config, agents_arg: &Option<Vec<String>>, yes: bool) -> Vec<AgentEntry> {
    if cfg.agents.is_empty() {
        return Vec::new();
    }

    if let Some(names) = agents_arg {
        if names.iter().any(|n| n.eq_ignore_ascii_case("none")) {
            return Vec::new();
        }
        if names.iter().any(|n| n.eq_ignore_ascii_case("all")) {
            return cfg.agents.clone();
        }
        let mut out = Vec::new();
        for n in names {
            match cfg.agents.iter().find(|a| &a.name == n) {
                Some(a) => out.push(a.clone()),
                None => eprintln!("warning: no configured agent named '{n}', skipping"),
            }
        }
        return out;
    }

    if yes {
        return Vec::new();
    }

    let labels: Vec<String> = cfg
        .agents
        .iter()
        .map(|a| format!("{} ({})", a.name, a.target))
        .collect();
    let picks = prompt::multi_select("\nWhich agent files should this worktree get?", &labels);
    picks.into_iter().map(|i| cfg.agents[i].clone()).collect()
}
