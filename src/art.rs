//! Turns a name into a greeting rendered as big block-letter ASCII art via the
//! standard FIGlet font, with plain-text fallbacks for anything the font or the
//! terminal can't handle.

use figlet_rs::FIGfont;

pub struct Art {
    pub lines: Vec<String>,
    pub width: usize,
    pub height: usize,
}

/// `term_width` of `None` skips the width check (used by tests that don't care
/// about a real terminal size).
pub fn build(name: &str, term_width: Option<u16>) -> Art {
    let text = format!("Welcome, {name}!");

    // figlet's standard font only covers ASCII + a handful of German characters —
    // a Chinese/Thai/etc. name would otherwise silently drop every unmapped char.
    if !text.is_ascii() {
        return plain(&text);
    }

    let Some(lines) = figlet_lines(&text) else {
        return plain(&text);
    };

    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    if let Some(term_width) = term_width
        && width > term_width as usize
    {
        return plain(&text);
    }

    let height = lines.len();
    Art {
        lines,
        width,
        height,
    }
}

fn figlet_lines(text: &str) -> Option<Vec<String>> {
    let font = FIGfont::standard().ok()?;
    let figure = font.convert(text)?;
    let rendered = figure.to_string();

    let mut lines: Vec<String> = rendered.lines().map(|l| l.trim_end().to_string()).collect();

    // Fonts pad every character to a fixed glyph height, so short text often ends
    // with one or more fully blank rows (headroom for descenders like g/p that
    // this text doesn't use). Trim those, but never trim down to nothing.
    while lines.len() > 1 && lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }

    if lines.is_empty() { None } else { Some(lines) }
}

fn plain(text: &str) -> Art {
    Art {
        lines: vec![text.to_string()],
        width: text.chars().count(),
        height: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_name_renders_as_figlet_block_art() {
        let art = build("Chai", None);
        assert!(
            art.height > 1,
            "figlet output should be multi-row block art"
        );
        assert!(art.lines.iter().any(|l| !l.trim().is_empty()));
    }

    #[test]
    fn non_ascii_name_falls_back_to_plain_text() {
        let art = build("柴由民", None);
        assert_eq!(art.height, 1);
        assert_eq!(art.lines[0], "Welcome, 柴由民!");
    }

    #[test]
    fn width_overflow_falls_back_to_plain_text() {
        let wide = build("Chai", None).width;
        let art = build("Chai", Some((wide - 1) as u16));
        assert_eq!(art.height, 1);
        assert_eq!(art.lines[0], "Welcome, Chai!");
    }

    #[test]
    fn width_within_terminal_keeps_figlet_art() {
        let wide = build("Chai", None).width;
        let art = build("Chai", Some(wide as u16));
        assert!(art.height > 1);
    }

    #[test]
    fn trailing_blank_figlet_rows_are_trimmed_but_not_all_of_them() {
        let art = build("Chai", None);
        let last = art.lines.last().unwrap();
        assert!(!last.trim().is_empty(), "should not trim every row away");
    }
}
