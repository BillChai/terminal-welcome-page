# terminal-welcome-page zsh hook.
#
# ~/.zshrc should only ever need one line:
#   source "$HOME/.local/share/terminal-welcome-page/twp.zsh"
# Any future change to the playback logic goes here, never back into .zshrc.
#
# Guards, in order: interactive shell, real tty, not explicitly disabled, and not
# already shown in this exact tty. The last guard is keyed on $TTY (not a bare
# TWP_SHOWN=1 flag and not $SHLVL) because a bare flag leaks into tmux server
# lifetimes and long-lived parent processes, silently suppressing panes/shells
# that should be greeted, whereas $TTY is guaranteed different for a new
# window/tab and identical for a nested shell in the same window.

_twp_hook() {
  emulate -L zsh

  [[ -o interactive ]] || return
  [[ -t 1 ]] || return
  [[ -z "$TWP_DISABLE" ]] || return
  [[ "$TWP_SHOWN_TTY" != "$TTY" ]] || return

  local bin="${TWP_BIN:-$HOME/.local/bin/twp}"
  local cfg="${TWP_CONFIG_FILE:-$HOME/.config/terminal-welcome-page/config.env}"

  [[ -x "$bin" ]] || return

  export TWP_SHOWN_TTY="$TTY"

  # Subshell: config.env is sourced (arbitrary shell code, hence a subshell so
  # its TWP_* vars die when the subshell exits) then the binary replaces it via
  # exec, saving a fork. Never `exec` at the top level of .zshrc — that would
  # replace the interactive shell itself and the terminal would close when the
  # animation ends.
  (
    [[ -r "$cfg" ]] && source "$cfg" 2>/dev/null
    exec "$bin"
  ) 2>/dev/null
}

_twp_hook
unfunction _twp_hook
