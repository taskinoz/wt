use crate::git;

pub fn run() {
    match git::worktree_list_raw() {
        Ok(text) => print!("{text}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
