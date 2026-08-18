# terminal-welcome-page

Every time you open a new interactive zsh terminal, prints a short animated
ASCII greeting ("Welcome, `${name}`!") rendered as big block letters, then
gets out of the way before your prompt.

Written in Rust (`twp`), configured entirely through shell variables — no
JSON/YAML, no config parser. See [`config.env.example`](config.env.example).

## Install

```bash
./install.sh
```

This builds the binary (`cargo install --path . --root ~/.local`), installs
the shell hook to `~/.local/share/terminal-welcome-page/twp.zsh`, and writes a
default config to `~/.config/terminal-welcome-page/config.env` if you don't
already have one.

It will **not** touch your `~/.zshrc`. Instead it prints a 3-line block and
asks you to paste it in yourself — ideally near the top of the file, and
**above** any "instant prompt" block your prompt theme uses (e.g.
Powerlevel10k), since anything that prints console output during shell
startup has to go above that block or you'll get a warning and a garbled
first prompt.

If you're confident about your setup, `./install.sh --auto` will insert it
for you automatically — but only if it can find a Powerlevel10k
instant-prompt block to insert before; otherwise it refuses to guess. Either
way it backs up your `~/.zshrc` first, to `~/.zshrc.twp.bak.<timestamp>`
(in your home directory, never inside this repo).

## Configure

Edit `~/.config/terminal-welcome-page/config.env`:

| Variable | Default | Notes |
|---|---|---|
| `TWP_NAME` | `$USER` | Truncated to 16 characters. Non-ASCII names (e.g. Chinese) fall back to plain text — figlet's standard font only covers ASCII. |
| `TWP_ANIMATION` | `typewriter` | One of `typewriter`, `wave`, `bounce`. Unknown values silently fall back to the default. |
| `TWP_DURATION_MS` | `700` | Clamped to `0..1500`. Kept short on purpose so it doesn't cancel out an instant-prompt-style setup. |
| `TWP_FPS` | `30` | Clamped to `5..60`. |

Other environment variables, meant to be set outside `config.env`:

- `TWP_DISABLE=1` — skip the animation entirely without touching `~/.zshrc` or your config.
- `TWP_DEBUG=1` — print the resolved config to stderr for troubleshooting.

## Uninstall

```bash
./uninstall.sh
```

Removes the `~/.zshrc` block (backing it up first), the binary, and the shell
hook. Asks before deleting your config.

## How it decides whether to greet you

`shell/twp.zsh` only runs the animation when **all** of these hold:

- the shell is interactive
- stdout is a real terminal (not a pipe, script, or task runner)
- `TWP_DISABLE` isn't set
- this exact terminal (`$TTY`) hasn't already been greeted

The last one is why it's keyed on `$TTY` rather than a plain "already shown"
flag: a plain flag leaks into tmux server / long-lived parent process
lifetimes and ends up suppressing panes that should be greeted. `$TTY` is
guaranteed different for a new window/tab and identical for a nested shell in
the same one, so opening a subshell inside an already-greeted terminal won't
replay the animation, but a new window will. New tmux panes get their own
pty, so they *do* get greeted — if you don't want that, add `-z $TMUX` to the
condition in `shell/twp.zsh`.

## Known limitations

- **No skip-on-keypress.** Reading a key to let you skip the animation would
  also swallow whatever you type right after opening the terminal (macOS has
  no way to push those bytes back to the shell). Instead the animation is just
  kept short.
- **Terminal.app has no truecolor.** The `wave` animation downgrades RGB to
  the nearest 256-color-cube value there automatically; it looks better in
  iTerm2 or VS Code's terminal, which do support truecolor.
- **No alternate screen.** By design — an alt-screen leak from a killed
  process leaves the terminal with no scrollback and nothing fixes it. This
  project only uses relative cursor motion, so worst case a leaked cursor
  position is harmless.
- **`config.env` is arbitrary shell code**, sourced on every new terminal.
  That's fine for a file you own, but don't put anything in it you wouldn't
  put directly in `.zshrc`.

## Development

```bash
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`tests/shell_guard.rs` and `tests/pty.rs` drive real `zsh`/the real binary
through a pty (via macOS's `script`) and never touch your real `~/.zshrc`.

### Writing your own animation feel

Two files are intentionally left as starting points rather than finished
animations:

- [`src/anim/bounce.rs`](src/anim/bounce.rs) — `vertical_offset(t, headroom)`
  decides how many rows to push the banner down at progress `t`. Currently a
  no-op (always `0`).
- [`src/anim/wave.rs`](src/anim/wave.rs) — `color_at(col, row, t)` decides
  each cell's color. Currently solid white.

Both are pure functions with no I/O, so you can iterate on them purely
through `cargo test` before ever looking at a real terminal.
