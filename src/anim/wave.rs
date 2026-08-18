use super::Animation;
use crate::art::Art;
use crate::config::ColorMode;
use crate::frame::{Cell, Frame};
use crossterm::style::Color;

pub struct Wave;

impl Animation for Wave {
    fn render(&self, art: &Art, t: f32, out: &mut Frame, color_mode: ColorMode) {
        for (y, line) in art.lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    out.set(x, y, Cell::plain(ch));
                    continue;
                }
                let color = downgrade(color_at(x, y, t), color_mode);
                out.set(
                    x,
                    y,
                    Cell {
                        ch,
                        color: Some(color),
                    },
                );
            }
        }
    }
}

// TODO(you): the rainbow-sweep formula — given a cell's column/row and the
// animation's normalized progress `t` (0.0..=1.0), return its color.
//
// Things you get to decide: wavelength and sweep speed (how many color cycles
// fit across the banner, and how fast they scroll as `t` advances); direction
// (mixing `row` into the phase makes the sweep diagonal instead of purely
// horizontal); and whether it's a full HSV rainbow cycle or an interpolation
// between two fixed colors. Return `Color::Rgb` — `downgrade()` below already
// handles terminals without truecolor (e.g. Terminal.app), so this function
// only needs to worry about how it looks, not compatibility.
fn color_at(_col: usize, _row: usize, _t: f32) -> Color {
    // Placeholder: solid white, no wave yet. Replace this with your own formula.
    Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    }
}

/// Terminal.app and some others don't support 24-bit color — SGR 38;2;r;g;b
/// either gets ignored or renders wrong there, so anything RGB gets mapped
/// onto the 6x6x6 ANSI 256 color cube when the terminal can't do truecolor.
fn downgrade(color: Color, mode: ColorMode) -> Color {
    match (color, mode) {
        (Color::Rgb { r, g, b }, ColorMode::Ansi256) => Color::AnsiValue(rgb_to_ansi256(r, g, b)),
        (c, _) => c,
    }
}

fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let to_cube = |v: u8| (v as u16 * 5 / 255) as u8;
    16 + 36 * to_cube(r) + 6 * to_cube(g) + to_cube(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truecolor_passes_through_unchanged() {
        let c = Color::Rgb {
            r: 10,
            g: 20,
            b: 30,
        };
        assert_eq!(downgrade(c, ColorMode::Truecolor), c);
    }

    #[test]
    fn ansi256_downgrades_rgb() {
        let c = Color::Rgb { r: 255, g: 0, b: 0 };
        assert_eq!(downgrade(c, ColorMode::Ansi256), Color::AnsiValue(196));
    }

    #[test]
    fn ansi256_black_and_white_corners() {
        assert_eq!(
            downgrade(Color::Rgb { r: 0, g: 0, b: 0 }, ColorMode::Ansi256),
            Color::AnsiValue(16)
        );
        assert_eq!(
            downgrade(
                Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255
                },
                ColorMode::Ansi256
            ),
            Color::AnsiValue(231)
        );
    }
}
