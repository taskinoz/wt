mod cmd;
mod config;
mod git;
mod prompt;

use clap::{Parser, Subcommand};
use cmd::completion::Shell;
use cmd::config_cmd::ConfigArgs;
use cmd::new::NewArgs;
use cmd::rm::RmArgs;

#[derive(Parser)]
#[command(name = "wt", version, about = "Fast git worktree helper with agent-file pre-population")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new worktree
    New(NewArgs),
    /// View or (re)run the setup wizard for wt's configuration
    Config(ConfigArgs),
    /// List worktrees for the current repo
    List,
    /// Remove a worktree
    Rm(RmArgs),
    /// Print a shell completion script
    Completion { shell: Shell },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New(args) => cmd::new::run(args),
        Commands::Config(args) => cmd::config_cmd::run(args),
        Commands::List => cmd::list::run(),
        Commands::Rm(args) => cmd::rm::run(args),
        Commands::Completion { shell } => cmd::completion::run(shell),
    }
}
