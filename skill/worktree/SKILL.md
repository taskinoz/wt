---
name: worktree
description: Create, list, and remove git worktrees on the user's behalf using the `wt` CLI (a minimal git-worktree helper that places worktrees under a `.worktree/<name>` folder and can pre-populate each one with a per-project agent/instructions file such as CLAUDE.md or AGENTS.md). Use this whenever the user wants to work on something in an isolated checkout without disturbing their current one — spinning up a new branch for a feature or fix, checking out an existing branch into its own folder, wanting to "try something in a separate worktree/folder", or asking to list or clean up worktrees — even if they never say the word "worktree". Also use this when the user wants to register, view, or change which agent/instructions file(s) get copied or symlinked into new worktrees for a given project in a monorepo.
---

# Worktree helper (`wt`)

`wt` wraps `git worktree` so that creating an isolated working copy is a
single short command instead of a multi-step `git worktree add` +
"remember to copy the instructions file" ritual. It always creates
worktrees under `<repo-root>/.worktree/<short-name>` by default, and can
symlink or copy one or more registered "agent" files (project-specific
`CLAUDE.md` / `AGENTS.md` notes) into the new worktree automatically.

Read this whole file before running any `wt` command — the flag semantics
around `-b` are easy to get backwards.

## 0. Locate the binary

```bash
command -v wt
```

If that fails, `wt` isn't installed yet. Build it from source rather than
giving up:

```bash
cd /Users/tristan/Documents/leaf/wt && cargo build --release
```

Then use `/Users/tristan/Documents/leaf/wt/target/release/wt` in place of
`wt` for the rest of this skill (or tell the user to run
`cargo install --path .` from that directory, and point them at its
`README.md` for shell-completion setup, if they'd rather install it
properly first).

## 1. Confirm you're in the right repo

`wt` operates on whatever git repo contains the current directory
(`git rev-parse --show-toplevel` under the hood). If the user's request
doesn't make it obvious which repo/project they mean — especially in a
monorepo with several sub-projects — ask, or `cd` into the right place
first. Don't guess at a repo the user didn't mention.

## 2. Check existing configuration before creating anything

```bash
wt config show
```

This prints the config file path and its JSON: `worktree_dir` (default
`.worktree`), `link_mode` (`symlink` or `copy`), and `agents` — a list of
`{name, source, target}` entries already registered for this machine
(e.g. `{"name": "backend", "source": "/Users/.../Obsidian/.../Backend.md",
"target": "CLAUDE.md"}`).

- **If a config already exists**, use its `agents` list to figure out
  which entry (if any) matches the project/sub-project the user is working
  in. In a monorepo each sub-project typically has its own entry.
- **If no config exists yet**, do *not* let `wt new` fall into its
  interactive first-run wizard unattended — that wizard reads from stdin
  and will hang or fail non-interactively. Instead:
  - If the user just wants a worktree right now and hasn't mentioned agent
    files, run `wt new ... -y` (skips prompts, creates the worktree with no
    agent file — that's a safe, reversible default).
  - If the user wants agent files wired up, ask them for the source path(s)
    and desired target filename(s), then register them non-interactively:
    ```bash
    wt config add-agent <name> <source-path> <target-filename>
    ```
    You can call this once per agent file. It's additive and safe to run
    any time — it doesn't touch existing entries with different names.

## 3. Create the worktree

The core command:

```bash
wt new -b <new-branch-name>            # brand-new branch off HEAD
wt new <existing-branch-name>          # checkout an existing branch
```

**These two forms are mutually exclusive and easy to mix up:**
`-b` *creates* a branch and must be a name that does not exist yet. Without
`-b`, the positional argument must already exist as a branch — `wt` will
refuse (a new branch off HEAD) rather than silently doing the wrong thing,
but don't rely on that; pick the right form up front. If you're not sure
whether a branch exists, check first (`git branch --list <name>`) or just
ask the user whether they want a new branch or to resume an existing one.

Useful flags:

| Flag | Meaning |
|---|---|
| `--from <ref>` | base the new branch on `<ref>` instead of `HEAD` (only with `-b`) |
| `-n <name>` | override the worktree's short folder name (default: the branch's last `/`-segment, e.g. `feature/login-page` → `login-page`) |
| `-p <path>` | put the worktree at an explicit full path instead of `.worktree/<name>` |
| `-a <names>` | comma-separated agent names to pre-populate, or `all` / `none` |
| `--copy` / `--link` | force copy or symlink for this run, overriding the configured default |
| `-y` | skip interactive prompts (agent selection then defaults to none unless `-a` is also given) |

**Deciding the branch name**: if the user gave you one, use it verbatim.
If they described a task instead ("let's fix the login bug"), propose a
short kebab-case branch name derived from it (e.g. `fix/login-bug`) and
say what you're about to run before running it if there's any ambiguity —
creating a branch is cheap and reversible, but confirm rather than
guessing when the user's intent isn't clear.

**Deciding agent files**: prefer `-a <name>` naming the specific
config entry that matches the project being worked on over `-a all`,
especially in a monorepo where an unrelated project's agent file would
just be noise. If nothing matches, don't force one in — pass `-y` (or
`-a none`) rather than defaulting to every registered agent.

Example, always run non-interactively (`-y` plus explicit `-a`) so the
command completes without you needing to answer prompts on the user's
behalf:

```bash
wt new -b feature/login-page -a backend -y
```

After it runs, `wt new` prints the created path, branch, and which agent
files it linked/copied — relay that back to the user along with the `cd`
command to get into it.

## 4. Listing and removing worktrees

```bash
wt list                 # thin wrapper around `git worktree list`
wt rm <short-name>       # remove by folder name (or a full path)
```

`wt rm` will refuse if the worktree has uncommitted or untracked changes,
asking for `--force`. **Never pass `--force` on the user's behalf without
their explicit confirmation** — that discards real work. Surface the
refusal to them and let them decide.

## Notes

- `wt`'s own first-run wizard and `wt config` (no subcommand) are
  interactive and read from stdin — never invoke either without a
  subcommand/flags in an automated context; use `wt config show` /
  `wt config add-agent` / `wt config remove-agent` instead, which are
  fully non-interactive.
- The full `README.md` next to this skill (in the `wt` project) has
  install and shell-completion instructions if the user wants to use `wt`
  directly themselves going forward.
