use super::Animation;
use crate::art::Art;
use crate::config::ColorMode;
use crate::frame::{Cell, Frame};

const HEADROOM: u16 = 2;

pub struct Bounce;

impl Animation for Bounce {
    fn render(&self, art: &Art, t: f32, out: &mut Frame, _color_mode: ColorMode) {
        let offset = vertical_offset(t, HEADROOM) as usize;
        for (y, line) in art.lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                out.set(x, y + offset, Cell::plain(ch));
            }
        }
    }

    fn headroom(&self) -> usize {
        HEADROOM as usize
    }
}

// TODO(you): this is the animation's personality — how many rows to push the
// whole banner down at normalized progress `t` (0.0 at the start of the
// animation, 1.0 at the end), given `headroom` extra rows reserved below the
// art to bounce into.
//
// Some directions to try: a single decaying sine wave `(1.0 - t) * sin(k * PI
// * t)`; a piecewise ease-out-bounce with 2-3 diminishing hops; a gravity +
// restitution simulation; or an anticipation-then-overshoot curve. Keep it a
// pure `f32 -> f32`-ish function (no I/O) so it's easy to unit test — see
// tests/frames.rs for a starting point once you've picked a curve.
//
// Constraints the render loop assumes: the return value must stay within
// `0..=headroom` (values outside that range get silently clamped by
// Frame::set, i.e. they just disappear off the bottom rather than panicking,
// but that'll look like a bug).
fn vertical_offset(_t: f32, _headroom: u16) -> u16 {
    // Placeholder: no bounce yet. Replace this with your own motion curve.
    0
}
