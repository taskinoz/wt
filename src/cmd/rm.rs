use crate::git;
use clap::Args;

#[derive(Args)]
pub struct RmArgs {
    /// Short worktree folder name (or full path) to remove
    name: String,

    /// Force removal even if the worktree has uncommitted changes
    #[arg(short = 'f', long)]
    force: bool,
}

pub fn run(args: RmArgs) {
    let (path, branch) = match git::resolve_worktree(&args.name) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = git::worktree_remove(&path, args.force) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
    println!("Removed worktree {}", path.display());
    if let Some(b) = branch {
        println!("Branch '{b}' still exists. Delete it with: git branch -d {b}");
    }
}
