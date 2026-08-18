#!/usr/bin/env bash
# Reverses install.sh: removes the ~/.zshrc block (exact-match delete, not a
# regex over the whole file), the binary, and the shell hook. Backs up
# ~/.zshrc first. Asks before touching your config (name/animation settings).
set -euo pipefail

INSTALL_ROOT="${TWP_INSTALL_ROOT:-$HOME/.local}"
BIN_PATH="$INSTALL_ROOT/bin/twp"
SHARE_DIR="$HOME/.local/share/terminal-welcome-page"
HOOK_PATH="$SHARE_DIR/twp.zsh"
CONFIG_DIR="$HOME/.config/terminal-welcome-page"
ZSHRC="${TWP_ZSHRC:-$HOME/.zshrc}"
MARKER_START="# >>> terminal-welcome-page >>>"
MARKER_END="# <<< terminal-welcome-page <<<"

if [[ -f "$ZSHRC" ]] && grep -qF "$MARKER_START" "$ZSHRC"; then
  BACKUP="$HOME/.zshrc.twp.bak.$(date +%Y%m%d%H%M%S)"
  cp "$ZSHRC" "$BACKUP"
  echo "==> Backed up $ZSHRC to $BACKUP"

  awk -v start="$MARKER_START" -v end="$MARKER_END" '
    $0 == start { skip = 1; next }
    $0 == end { skip = 0; next }
    skip { next }
    { print }
  ' "$ZSHRC" > "$ZSHRC.tmp" && mv "$ZSHRC.tmp" "$ZSHRC"

  echo "==> Removed the terminal-welcome-page block from $ZSHRC"
else
  echo "==> No terminal-welcome-page block found in $ZSHRC, nothing to remove there."
fi

echo "==> Removing $BIN_PATH"
rm -f "$BIN_PATH"

echo "==> Removing $HOOK_PATH"
rm -f "$HOOK_PATH"
rmdir "$SHARE_DIR" 2>/dev/null || true

read -r -p "Also remove config at $CONFIG_DIR (your name/animation settings)? [y/N] " REPLY
case "$REPLY" in
  [yY]*)
    rm -rf "$CONFIG_DIR"
    echo "==> Removed $CONFIG_DIR"
    ;;
  *)
    echo "==> Left $CONFIG_DIR in place"
    ;;
esac

echo "==> Done. Open a new terminal to confirm the greeting is gone."
