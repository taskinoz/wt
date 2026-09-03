# wt

A minimal git worktree helper. Creates worktrees fast, under a standard
`.worktree/<name>` layout, and can pre-populate each new worktree with one or
more "agent" files (e.g. `CLAUDE.md` / `AGENTS.md` notes you keep in Obsidian
and symlink into projects).

Single static-ish Rust binary, no runtime dependencies beyond `git` itself.

## Install

```bash
cargo install --path .
```

This installs `wt` to `~/.cargo/bin/wt` (make sure that's on your `PATH`).

Or just build and copy the binary yourself:

```bash
cargo build --release
cp target/release/wt /usr/local/bin/wt
```

### Shell completion

```bash
# zsh — put the generated file somewhere on your $fpath, e.g.:
mkdir -p ~/.zsh/completions
wt completion zsh > ~/.zsh/completions/_wt
# then make sure ~/.zsh/completions is on fpath, before compinit, in ~/.zshrc:
#   fpath=(~/.zsh/completions $fpath)
#   autoload -Uz compinit && compinit

# bash
wt completion bash >> ~/.bashrc
```

This gives you tab-completion for branch names on `wt new <TAB>` and for
worktree names on `wt rm <TAB>`.

## First use

The first time you run `wt new`, it walks you through a one-off setup wizard
(worktree folder name, default symlink-vs-copy, and any agent files you want
to register). You can re-run it any time:

```bash
wt config            # re-run the full wizard
wt config show       # print current config + its file path
wt config add-agent  # add one more agent file entry (prompts, or pass args)
wt config remove-agent <name>
```

Config lives at `$XDG_CONFIG_HOME/wt/config.json` (or `~/.config/wt/config.json`).

Each agent entry is `{ name, source, target }`: `source` is the file to link
or copy (your Obsidian note, or wherever you keep it), `target` is the
filename it becomes inside the new worktree (e.g. `CLAUDE.md`).

## Usage

```bash
# Create a worktree for a NEW branch off HEAD, under .worktree/<name>
wt new -b feature/login-page

# Check out an EXISTING branch into a new worktree (tab-completable)
wt new feature/login-page

# Base the new branch on something other than HEAD
wt new -b feature/x --from main

# Override the folder name (default: last path segment of the branch)
wt new -b feature/x -n custom-folder-name

# Use a fully custom path instead of .worktree/<name>
wt new -b feature/x -p ../scratch/feature-x

# Choose agent files explicitly (skips the interactive picker)
wt new -b feature/x -a backend,frontend
wt new -b feature/x -a all
wt new -b feature/x -a none

# Force copy or symlink for this run, overriding the configured default
wt new -b feature/x --copy
wt new -b feature/x --link

# Non-interactive (agent files are skipped unless -a is also given)
wt new -b feature/x -y

wt list              # thin wrapper around `git worktree list`
wt rm <name>         # remove a worktree by its short folder name (or full path)
wt rm <name> --force # ...even if it has uncommitted/untracked changes
```

If you run `wt new` without a branch and without `-b`, it tells you to
either tab-complete an existing branch or pass `-b <name>` to create one.

## Design notes

- Default worktree location: `<repo-root>/<worktree_dir>/<short-name>`,
  where `worktree_dir` defaults to `.worktree` and `short-name` defaults to
  the branch name's last `/`-segment (`feature/login-page` -> `login-page`).
- `-p/--path` bypasses that scheme entirely for a one-off custom location.
- Agent files are never invented or guessed — you register the exact
  source path and target filename for each one via `wt config`.
