//! The only module allowed to touch global terminal state.
//!
//! Deliberately does NOT enable raw mode: reading a keypress to let the user
//! skip the animation would also swallow whatever they start typing right after
//! opening the terminal (macOS has no way to push those bytes back to the shell).
//! Instead the animation just stays short (see config::MAX_DURATION_MS). Cursor
//! hiding is the only state that needs restoring, and unlike raw mode, zsh's
//! line editor does NOT self-heal a leaked hidden cursor — hence the RAII guard
//! and the panic hook below.

use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use std::io::Write;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

pub struct TerminalGuard {
    _private: (),
}

impl TerminalGuard {
    pub fn new() -> std::io::Result<Self> {
        install_panic_hook();
        let mut out = std::io::stdout();
        execute!(out, Hide)?;
        out.flush()?;
        Ok(TerminalGuard { _private: () })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = execute!(out, Show);
        let _ = out.flush();
    }
}

/// Restores the cursor *before* delegating to the previous (default) panic
/// hook — the other order would print the panic message into a still-hidden
/// cursor / half-restored terminal.
fn install_panic_hook() {
    if HOOK_INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let mut out = std::io::stdout();
        let _ = execute!(out, Show);
        let _ = out.flush();
        previous(info);
    }));
}
