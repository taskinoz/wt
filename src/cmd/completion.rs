use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
}

pub fn run(shell: Shell) {
    match shell {
        Shell::Bash => print!("{BASH}"),
        Shell::Zsh => print!("{ZSH}"),
    }
}

const BASH: &str = r#"# wt completion for bash
# Install: wt completion bash >> ~/.bashrc   (or source it from a completions dir)
_wt_complete() {
  local cur prev
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"

  if [[ $COMP_CWORD -eq 1 ]]; then
    COMPREPLY=($(compgen -W "new config list rm completion" -- "$cur"))
    return
  fi

  case "${COMP_WORDS[1]}" in
    new)
      case "$prev" in
        -b|--branch|-n|--name|-p|--path|--from|-a|--agents)
          return
          ;;
      esac
      if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "-b --branch -p --path -n --name -a --agents --copy --link -y --yes --from" -- "$cur"))
        return
      fi
      local branches
      branches=$( { git branch --format='%(refname:short)' 2>/dev/null; git branch -r --format='%(refname:short)' 2>/dev/null; } )
      COMPREPLY=($(compgen -W "$branches" -- "$cur"))
      ;;
    rm)
      local names
      names=$(git worktree list --porcelain 2>/dev/null | awk '/^worktree /{print $2}' | tail -n +2 | xargs -I{} basename {})
      COMPREPLY=($(compgen -W "$names" -- "$cur"))
      ;;
    config)
      if [[ $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=($(compgen -W "show add-agent remove-agent" -- "$cur"))
      fi
      ;;
    completion)
      if [[ $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=($(compgen -W "bash zsh" -- "$cur"))
      fi
      ;;
  esac
}
complete -F _wt_complete wt
"#;

const ZSH: &str = r#"#compdef wt
# wt completion for zsh
# Install: wt completion zsh > ~/.zsh/completions/_wt   (somewhere on your $fpath)

_wt() {
  local -a subcmds
  subcmds=(new config list rm completion)

  if (( CURRENT == 2 )); then
    _describe 'command' subcmds
    return
  fi

  case ${words[2]} in
    new)
      _wt_new
      ;;
    rm)
      _wt_rm
      ;;
    config)
      if (( CURRENT == 3 )); then
        local -a actions
        actions=(show add-agent remove-agent)
        _describe 'action' actions
      fi
      ;;
    completion)
      if (( CURRENT == 3 )); then
        local -a shells
        shells=(bash zsh)
        _describe 'shell' shells
      fi
      ;;
  esac
}

_wt_new() {
  case ${words[CURRENT-1]} in
    -b|--branch|-n|--name|-p|--path|--from|-a|--agents)
      return
      ;;
  esac
  if [[ ${words[CURRENT]} == -* ]]; then
    local -a flags
    flags=(-b --branch -p --path -n --name -a --agents --copy --link -y --yes --from)
    _describe 'flag' flags
    return
  fi
  local -a branches
  branches=(${(f)"$(git branch --format='%(refname:short)' 2>/dev/null; git branch -r --format='%(refname:short)' 2>/dev/null)"})
  _describe 'branch' branches
}

_wt_rm() {
  local -a names
  names=(${(f)"$(git worktree list --porcelain 2>/dev/null | awk '/^worktree /{print \$2}' | tail -n +2 | xargs -I{} basename {})"})
  _describe 'worktree' names
}

_wt "$@"
"#;
