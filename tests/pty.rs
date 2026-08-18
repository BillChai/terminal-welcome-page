//! Runs the real binary through a pty (via macOS `script`) so raw ANSI bytes
//! can be inspected directly: proves there's no alt-screen entry, the cursor
//! gets shown again before exit, and the animation respects its time budget.

use std::process::Command;
use std::time::Instant;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_twp")
}

#[test]
fn clean_pty_output_no_alt_screen_and_cursor_restored() {
    let start = Instant::now();
    let output = Command::new("script")
        .arg("-q")
        .arg("/dev/null")
        .arg(bin())
        .env("TWP_DURATION_MS", "50")
        .env("TWP_FPS", "30")
        .env_remove("TWP_DISABLE")
        .output()
        .expect("failed to run twp under a pty");
    let elapsed = start.elapsed();

    let text = String::from_utf8_lossy(&output.stdout);

    assert!(
        text.contains('\u{1b}'),
        "expected ANSI escape sequences in output: {text:?}"
    );
    assert!(
        !text.contains("\u{1b}[?1049h"),
        "must never enter the alternate screen: {text:?}"
    );
    assert!(
        text.contains("\u{1b}[?25h"),
        "cursor must be shown again before exit: {text:?}"
    );
    assert!(
        elapsed.as_millis() < 1500,
        "animation exceeded its time budget: {elapsed:?}"
    );
}

#[test]
fn non_tty_invocation_produces_no_output() {
    let output = Command::new(bin())
        .env("TWP_DURATION_MS", "50")
        .output()
        .expect("failed to run twp without a pty");
    assert!(output.stdout.is_empty());
    assert!(output.status.success());
}

#[test]
fn twp_disable_produces_no_output_even_with_a_pty() {
    let output = Command::new("script")
        .arg("-q")
        .arg("/dev/null")
        .arg(bin())
        .env("TWP_DISABLE", "1")
        .output()
        .expect("failed to run twp under a pty");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        !text.contains('\u{1b}'),
        "TWP_DISABLE should suppress all output: {text:?}"
    );
}
