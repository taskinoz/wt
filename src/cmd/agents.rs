use crate::cmd::new::populate_entries;
use crate::config;
use crate::git;
use clap::Args;

#[derive(Args)]
pub struct AgentsArgs {
    /// Worktree short folder name (or full path) to add agent files to
    name: String,

    /// Comma-separated agent names to add, or "all" / "none" (skips the interactive picker)
    #[arg(short = 'a', long, value_delimiter = ',')]
    agents: Option<Vec<String>>,

    /// Copy agent files instead of symlinking (overrides config default)
    #[arg(long, conflicts_with = "link")]
    copy: bool,

    /// Symlink agent files instead of copying (overrides config default)
    #[arg(long)]
    link: bool,
}

pub fn run(args: AgentsArgs) {
    let (path, _branch) = match git::resolve_worktree(&args.name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let cfg = config::load_or_default();
    populate_entries(&cfg, &path, &cfg.agents, "agent", &args.agents, false, args.copy, args.link);
}
