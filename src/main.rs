mod anim;
mod art;
mod config;
mod frame;
mod guard;
mod render;

use config::Config;
use std::io::IsTerminal;

fn main() {
    // A stderr line on every new terminal is worse than no animation at all —
    // every failure path below is silent unless TWP_DEBUG=1.
    if std::env::var("TWP_DISABLE").as_deref() == Ok("1") {
        return;
    }
    if !std::io::stdout().is_terminal() {
        return;
    }

    let config = Config::from_process_env();
    if config.debug {
        eprintln!("twp: config = {:?}", config);
    }

    let term_width = crossterm::terminal::size().ok().map(|(cols, _)| cols);
    let art = art::build(&config.name, term_width);

    render::play(&config, &art);
}
