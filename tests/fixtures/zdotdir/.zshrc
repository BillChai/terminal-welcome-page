# Isolated .zshrc used only by tests/shell_guard.rs (via ZDOTDIR). Never touches
# the user's real ~/.zshrc.
export TWP_BIN="$TWP_TEST_REPO_ROOT/tests/fixtures/fake-twp.sh"
export TWP_CONFIG_FILE="/nonexistent-config-for-test.env"
source "$TWP_TEST_REPO_ROOT/shell/twp.zsh"
