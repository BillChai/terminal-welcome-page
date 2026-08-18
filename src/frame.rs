//! Pure data structures for a single rendered frame. No I/O and no ANSI here —
//! `render.rs` is the only module that turns a `Frame` into terminal escape codes,
//! which keeps this module (and the animations that fill it) trivially unit-testable.

use crossterm::style::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub color: Option<Color>,
}

impl Cell {
    pub fn blank() -> Self {
        Cell {
            ch: ' ',
            color: None,
        }
    }

    pub fn plain(ch: char) -> Self {
        Cell { ch, color: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Frame {
    pub fn blank(width: usize, height: usize) -> Self {
        Frame {
            width,
            height,
            cells: vec![Cell::blank(); width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        if x >= self.width || y >= self.height {
            return Cell::blank();
        }
        self.cells[y * self.width + x]
    }

    /// Out-of-bounds writes are silently dropped rather than panicking — an
    /// animation's math being slightly off should never crash every new terminal.
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.cells[y * self.width + x] = cell;
    }

    pub fn row(&self, y: usize) -> &[Cell] {
        if y >= self.height {
            return &[];
        }
        &self.cells[y * self.width..(y + 1) * self.width]
    }

    /// Row text with color stripped — used by golden tests so assertions read as
    /// plain strings instead of `Cell` structs.
    pub fn plain_row(&self, y: usize) -> String {
        self.row(y).iter().map(|c| c.ch).collect()
    }

    pub fn plain_rows(&self) -> Vec<String> {
        (0..self.height).map(|y| self.plain_row(y)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_frame_is_all_spaces() {
        let f = Frame::blank(3, 2);
        assert_eq!(f.plain_rows(), vec!["   ", "   "]);
    }

    #[test]
    fn set_and_get_roundtrip() {
        let mut f = Frame::blank(3, 2);
        f.set(1, 0, Cell::plain('x'));
        assert_eq!(f.get(1, 0).ch, 'x');
        assert_eq!(f.plain_row(0), " x ");
    }

    #[test]
    fn out_of_bounds_writes_are_ignored_not_panics() {
        let mut f = Frame::blank(2, 2);
        f.set(99, 99, Cell::plain('x'));
        assert_eq!(f.get(99, 99).ch, ' ');
    }
}
