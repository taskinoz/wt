use std::path::PathBuf;
use std::process::Command;

pub fn repo_root() -> Result<PathBuf, String> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !out.status.success() {
        return Err("not inside a git repository".to_string());
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(PathBuf::from(s))
}

pub fn branch_exists(name: &str) -> bool {
    Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{name}")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn worktree_add_new_branch(
    path: &std::path::Path,
    branch: &str,
    from: &str,
) -> Result<(), String> {
    run_worktree_add(&["-b", branch, path.to_str().unwrap(), from])
}

pub fn worktree_add_existing(path: &std::path::Path, branch: &str) -> Result<(), String> {
    run_worktree_add(&[path.to_str().unwrap(), branch])
}

fn run_worktree_add(args: &[&str]) -> Result<(), String> {
    let mut full = vec!["worktree", "add"];
    full.extend_from_slice(args);
    let status = Command::new("git")
        .args(&full)
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !status.success() {
        return Err("git worktree add failed".to_string());
    }
    Ok(())
}

pub fn worktree_remove(path: &std::path::Path, force: bool) -> Result<(), String> {
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    let path_str = path.to_str().unwrap();
    args.push(path_str);
    let status = Command::new("git")
        .args(&args)
        .status()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !status.success() {
        return Err("git worktree remove failed".to_string());
    }
    Ok(())
}

pub fn worktree_list_raw() -> Result<String, String> {
    let out = Command::new("git")
        .args(["worktree", "list"])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !out.status.success() {
        return Err("git worktree list failed".to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Returns (path, branch) pairs from `git worktree list --porcelain`.
pub fn worktree_list_parsed() -> Result<Vec<(PathBuf, Option<String>)>, String> {
    let out = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !out.status.success() {
        return Err("git worktree list failed".to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut result = Vec::new();
    let mut cur_path: Option<PathBuf> = None;
    let mut cur_branch: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("worktree ") {
            if let Some(p) = cur_path.take() {
                result.push((p, cur_branch.take()));
            }
            cur_path = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("branch refs/heads/") {
            cur_branch = Some(rest.to_string());
        } else if line.is_empty() {
            if let Some(p) = cur_path.take() {
                result.push((p, cur_branch.take()));
            }
        }
    }
    if let Some(p) = cur_path.take() {
        result.push((p, cur_branch.take()));
    }
    Ok(result)
}
