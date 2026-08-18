mod bounce;
mod typewriter;
mod wave;

use crate::art::Art;
use crate::config::{AnimationKind, ColorMode};
use crate::frame::Frame;

/// `t` is normalized progress through the animation (`0.0` at start, `1.0` at
/// end), driven by real elapsed time rather than a frame counter — see
/// render.rs — so a slow/loaded terminal drops frames instead of stretching
/// the animation out.
pub trait Animation {
    fn render(&self, art: &Art, t: f32, out: &mut Frame, color_mode: ColorMode);

    /// Extra rows below the art this animation needs (e.g. room to bounce into).
    fn headroom(&self) -> usize {
        0
    }
}

pub fn from_kind(kind: AnimationKind) -> Box<dyn Animation> {
    match kind {
        AnimationKind::Typewriter => Box::new(typewriter::Typewriter),
        AnimationKind::Wave => Box::new(wave::Wave),
        AnimationKind::Bounce => Box::new(bounce::Bounce),
    }
}
