use crate::cmd::new::populate_entries;
use crate::config;
use crate::git;
use clap::Args;

#[derive(Args)]
pub struct EnvsArgs {
    /// Worktree short folder name (or full path) to add env files to
    name: String,

    /// Comma-separated env names to add, or "all" / "none" (skips the interactive picker)
    #[arg(short = 'e', long, value_delimiter = ',')]
    envs: Option<Vec<String>>,

    /// Copy env files instead of symlinking (overrides config default)
    #[arg(long, conflicts_with = "link")]
    copy: bool,

    /// Symlink env files instead of copying (overrides config default)
    #[arg(long)]
    link: bool,
}

pub fn run(args: EnvsArgs) {
    let (path, _branch) = match git::resolve_worktree(&args.name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };
    let cfg = config::load_or_default();
    populate_entries(&cfg, &path, &cfg.envs, "env", &args.envs, false, args.copy, args.link);
}
