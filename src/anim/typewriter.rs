use super::Animation;
use crate::art::Art;
use crate::config::ColorMode;
use crate::frame::{Cell, Frame};

/// Reference animation: reveals the art left-to-right as `t` advances.
pub struct Typewriter;

impl Animation for Typewriter {
    fn render(&self, art: &Art, t: f32, out: &mut Frame, _color_mode: ColorMode) {
        let revealed_cols = ((t * art.width as f32).ceil() as usize).min(art.width);
        for (y, line) in art.lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if x < revealed_cols {
                    out.set(x, y, Cell::plain(ch));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::build;

    #[test]
    fn nothing_revealed_at_t_zero() {
        let art = build("Chai", None);
        let mut frame = Frame::blank(art.width, art.height);
        Typewriter.render(&art, 0.0, &mut frame, ColorMode::Ansi256);
        assert!(frame.plain_rows().iter().all(|row| row.trim().is_empty()));
    }

    #[test]
    fn everything_revealed_at_t_one() {
        let art = build("Chai", None);
        let mut frame = Frame::blank(art.width, art.height);
        Typewriter.render(&art, 1.0, &mut frame, ColorMode::Ansi256);
        for (y, line) in art.lines.iter().enumerate() {
            let mut expected = line.clone();
            expected.push_str(&" ".repeat(art.width.saturating_sub(line.chars().count())));
            assert_eq!(frame.plain_row(y), expected);
        }
    }

    #[test]
    fn reveal_grows_monotonically_with_t() {
        let art = build("Chai", None);
        let count_revealed = |t: f32| {
            let mut frame = Frame::blank(art.width, art.height);
            Typewriter.render(&art, t, &mut frame, ColorMode::Ansi256);
            frame
                .plain_rows()
                .iter()
                .map(|r| r.trim_end().len())
                .sum::<usize>()
        };
        assert!(count_revealed(0.25) <= count_revealed(0.5));
        assert!(count_revealed(0.5) <= count_revealed(0.75));
    }
}
