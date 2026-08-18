//! Exercises shell/twp.zsh's guard logic through a real pty (via macOS `script`),
//! using an isolated ZDOTDIR fixture so the user's real ~/.zshrc is never touched.

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_zsh(command: &str, extra_env: &[(&str, &str)]) -> String {
    let root = repo_root();
    let zdotdir = root.join("tests/fixtures/zdotdir");

    let mut cmd = Command::new("script");
    cmd.arg("-q")
        .arg("/dev/null")
        .arg("zsh")
        .arg("-i")
        .arg("-c")
        .arg(command)
        .env("ZDOTDIR", &zdotdir)
        .env("TWP_TEST_REPO_ROOT", &root)
        .env_remove("TWP_SHOWN_TTY")
        .env_remove("TWP_DISABLE");

    for (k, v) in extra_env {
        cmd.env(k, v);
    }

    let output = cmd.output().expect("failed to spawn zsh via script");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn greets_once_in_the_outer_interactive_shell() {
    let out = run_zsh("exit", &[]);
    assert_eq!(out.matches("GREETED").count(), 1, "output was: {out:?}");
}

#[test]
fn does_not_replay_in_a_nested_interactive_shell() {
    // Same pty/$TTY as the outer shell, so the second greeting must be suppressed.
    let out = run_zsh("zsh -i -c exit", &[]);
    assert_eq!(out.matches("GREETED").count(), 1, "output was: {out:?}");
}

#[test]
fn twp_disable_suppresses_the_greeting() {
    let out = run_zsh("exit", &[("TWP_DISABLE", "1")]);
    assert_eq!(out.matches("GREETED").count(), 0, "output was: {out:?}");
}
