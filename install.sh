#!/usr/bin/env bash
# Builds twp and installs it, but only ever *shows* the ~/.zshrc snippet by
# default — it does not edit your real ~/.zshrc unless you pass --auto, and
# even then only when it can find a Powerlevel10k instant-prompt block to
# insert before (and it backs up ~/.zshrc first either way).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_ROOT="${TWP_INSTALL_ROOT:-$HOME/.local}"
BIN_PATH="$INSTALL_ROOT/bin/twp"
SHARE_DIR="$HOME/.local/share/terminal-welcome-page"
HOOK_PATH="$SHARE_DIR/twp.zsh"
CONFIG_DIR="$HOME/.config/terminal-welcome-page"
CONFIG_PATH="$CONFIG_DIR/config.env"
ZSHRC="${TWP_ZSHRC:-$HOME/.zshrc}"
MARKER_START="# >>> terminal-welcome-page >>>"
MARKER_END="# <<< terminal-welcome-page <<<"

AUTO=0
for arg in "$@"; do
  case "$arg" in
    --auto) AUTO=1 ;;
  esac
done

# This script runs as a plain bash process (not your login zsh), so it never
# sources ~/.zshrc — load cargo's own env file directly instead of assuming
# it's already on PATH.
if ! command -v cargo >/dev/null 2>&1 && [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> cargo not found. Install Rust first: https://rustup.rs" >&2
  exit 1
fi

echo "==> Building and installing the twp binary to $BIN_PATH"
(cd "$REPO_ROOT" && cargo install --path . --root "$INSTALL_ROOT" --force)

echo "==> Installing shell hook to $HOOK_PATH"
mkdir -p "$SHARE_DIR"
cp "$REPO_ROOT/shell/twp.zsh" "$HOOK_PATH"

if [[ -f "$CONFIG_PATH" ]]; then
  echo "==> Config already exists at $CONFIG_PATH, leaving it untouched"
else
  echo "==> Writing default config to $CONFIG_PATH"
  mkdir -p "$CONFIG_DIR"
  cp "$REPO_ROOT/config.env.example" "$CONFIG_PATH"
fi

SOURCE_LINE="source \"$HOOK_PATH\""
BLOCK_FILE="$(mktemp)"
trap 'rm -f "$BLOCK_FILE"' EXIT
printf '%s\n' "$MARKER_START" "$SOURCE_LINE" "$MARKER_END" > "$BLOCK_FILE"

if [[ -f "$ZSHRC" ]] && grep -qF "$MARKER_START" "$ZSHRC"; then
  echo "==> $ZSHRC already has the terminal-welcome-page block, nothing to do."
  exit 0
fi

if [[ "$AUTO" -eq 1 ]]; then
  if [[ -f "$ZSHRC" ]] && LINE_NO="$(grep -n 'p10k-instant-prompt' "$ZSHRC" | head -1 | cut -d: -f1)" && [[ -n "$LINE_NO" ]]; then
    BACKUP="$HOME/.zshrc.twp.bak.$(date +%Y%m%d%H%M%S)"
    cp "$ZSHRC" "$BACKUP"
    echo "==> Backed up $ZSHRC to $BACKUP"

    awk -v line="$LINE_NO" -v blockfile="$BLOCK_FILE" '
      NR == line { while ((getline l < blockfile) > 0) print l }
      { print }
    ' "$ZSHRC" > "$ZSHRC.tmp" && mv "$ZSHRC.tmp" "$ZSHRC"

    echo "==> Inserted the hook into $ZSHRC, right before the Powerlevel10k instant-prompt block."
    echo "==> Open a new terminal to see it."
    exit 0
  else
    echo "==> --auto could not find a Powerlevel10k instant-prompt block in $ZSHRC."
    echo "==> Refusing to guess where to insert it. Add this block manually instead"
    echo "    (near the top of ~/.zshrc, above anything that prints console output"
    echo "    during shell startup):"
    echo
    cat "$BLOCK_FILE"
    exit 1
  fi
fi

echo
echo "==> Add this block to your ~/.zshrc yourself — near the top, and above any"
echo "    'instant prompt' block your prompt theme (e.g. Powerlevel10k) uses:"
echo
cat "$BLOCK_FILE"
echo
echo "==> Or re-run as '$0 --auto' to have this script insert it automatically"
echo "    (only if it can find a Powerlevel10k instant-prompt block; backs up"
echo "    ~/.zshrc first either way)."
