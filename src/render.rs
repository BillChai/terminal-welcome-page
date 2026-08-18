//! The animation loop, and the only module that emits ANSI. Deliberately does
//! NOT use the alternate screen: if the process dies mid-animation, a leaked
//! alt screen leaves the terminal with no scrollback and nothing fixes it —
//! whereas everything here uses plain relative cursor motion, so a leaked
//! cursor position is harmless (the next prompt just prints wherever it is).

use crate::anim;
use crate::art::Art;
use crate::config::Config;
use crate::frame::Frame;
use crate::guard::TerminalGuard;
use crossterm::cursor::{MoveDown, MoveToNextLine, MoveUp};
use crossterm::queue;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use std::io::{Stdout, Write, stdout};
use std::time::{Duration, Instant};

pub fn play(config: &Config, art: &Art) {
    let Ok(_guard) = TerminalGuard::new() else {
        return;
    };

    let animation = anim::from_kind(config.animation);
    let width = art.width.max(1);
    let height = art.height + animation.headroom();

    let mut out = stdout();
    reserve_rows(&mut out, height);

    let start = Instant::now();
    let frame_interval = Duration::from_secs_f64(1.0 / config.fps as f64);
    let duration_secs = config.duration.as_secs_f32().max(f32::EPSILON);
    let mut frame_count: u32 = 0;

    loop {
        let t = (start.elapsed().as_secs_f32() / duration_secs).min(1.0);

        let mut frame = Frame::blank(width, height);
        animation.render(art, t, &mut frame, config.color_mode);
        draw_frame(&mut out, &frame);

        if t >= 1.0 {
            break;
        }

        frame_count += 1;
        let deadline = start + frame_interval * frame_count;
        let now = Instant::now();
        if deadline > now {
            std::thread::sleep(deadline - now);
        }
    }

    finish(&mut out, height);
}

/// Forces a scroll (if we're at the bottom of the visible window) before the
/// loop starts, so the whole `height`-row banner is guaranteed on-screen, then
/// moves back up to the origin the per-frame redraw expects.
fn reserve_rows(out: &mut Stdout, height: usize) {
    for _ in 0..height {
        let _ = writeln!(out);
    }
    let _ = queue!(out, MoveUp(height as u16));
    let _ = out.flush();
}

/// Redraws every row from the origin and returns the cursor there, clearing
/// only the current line each time (never `Clear(ClearType::All)`, which would
/// wipe the user's scrollback) and flushing once per frame to avoid tearing.
fn draw_frame(out: &mut Stdout, frame: &Frame) {
    for y in 0..frame.height {
        let _ = queue!(out, Clear(ClearType::CurrentLine));

        let mut current_color: Option<Color> = None;
        for cell in frame.row(y) {
            if cell.color != current_color {
                let _ = match cell.color {
                    Some(c) => queue!(out, SetForegroundColor(c)),
                    None => queue!(out, ResetColor),
                };
                current_color = cell.color;
            }
            let mut buf = [0u8; 4];
            let _ = out.write_all(cell.ch.encode_utf8(&mut buf).as_bytes());
        }

        let _ = queue!(out, ResetColor, MoveToNextLine(1));
    }
    let _ = queue!(out, MoveUp(frame.height as u16));
    let _ = out.flush();
}

/// Leaves the final frame in scrollback above the next prompt, rather than
/// erasing it — a welcome banner that persists reads better than one that
/// flashes and disappears.
fn finish(out: &mut Stdout, height: usize) {
    let _ = queue!(out, MoveDown(height as u16));
    let _ = out.flush();
}
