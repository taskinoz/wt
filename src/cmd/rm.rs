use crate::git;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct RmArgs {
    /// Short worktree folder name (or full path) to remove
    name: String,

    /// Force removal even if the worktree has uncommitted changes
    #[arg(short = 'f', long)]
    force: bool,
}

pub fn run(args: RmArgs) {
    let worktrees = match git::worktree_list_parsed() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let as_path = PathBuf::from(&args.name);
    let matches: Vec<&(PathBuf, Option<String>)> = worktrees
        .iter()
        .skip(1) // first entry is always the main worktree; never targeted by short name
        .filter(|(p, _)| p == &as_path || p.file_name().map(|f| f == args.name.as_str()).unwrap_or(false))
        .collect();

    let target = match matches.as_slice() {
        [] => {
            eprintln!("error: no worktree matches '{}'", args.name);
            eprintln!("Run `wt list` to see current worktrees.");
            std::process::exit(1);
        }
        [one] => one,
        many => {
            eprintln!("error: '{}' matches multiple worktrees:", args.name);
            for (p, _) in many {
                eprintln!("  {}", p.display());
            }
            eprintln!("Pass the full path to disambiguate.");
            std::process::exit(1);
        }
    };

    let (path, branch) = target;
    if let Err(e) = git::worktree_remove(path, args.force) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
    println!("Removed worktree {}", path.display());
    if let Some(b) = branch {
        println!("Branch '{b}' still exists. Delete it with: git branch -d {b}");
    }
}
