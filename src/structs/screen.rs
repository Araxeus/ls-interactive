//! A small retained mode renderer for an inline region of the terminal.
//!
//! The region is the group of rows starting at the cursor. [`Screen::draw`] compares the new
//! frame with the one on screen and only rewrites the rows that changed:
//!
//! * a row is overwritten in place and the rest of the row is erased *afterwards*, so the terminal
//!   never shows a blank row in between,
//! * the whole frame goes out as one `write` wrapped in a synchronized update (DEC mode 2026), so
//!   terminals that support it paint the frame atomically and, over SSH, it travels as a single
//!   packet instead of dribbling in line by line,
//! * all cursor movement is relative to the region, so no round trip to ask the terminal where the
//!   cursor is.
//!
//! This is the same idea as the differential renderer described by the Claude Code team in
//! <https://github.com/anthropics/claude-code/issues/769#issuecomment-3667315590>, reduced to
//! whole rows because the region always fits on screen and never touches the scrollback.

use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveDown, MoveUp, Show},
    terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate},
    Command,
};

use super::ansi;

/// Appends the escape sequence of `command` to `out`.
fn emit(out: &mut String, command: impl Command) {
    // Writing to a String cannot fail
    let _ = command.write_ansi(out);
}

fn move_up(out: &mut String, rows: usize) {
    if rows > 0 {
        emit(out, MoveUp(u16::try_from(rows).unwrap_or(u16::MAX)));
    }
}

fn move_down(out: &mut String, rows: usize) {
    if rows > 0 {
        emit(out, MoveDown(u16::try_from(rows).unwrap_or(u16::MAX)));
    }
}

pub struct Screen<W: Write> {
    out: W,
    width: usize,
    /// What each row of the region currently shows, `None` while it is unknown. The region is
    /// always at least one row: the one the cursor was on when drawing started.
    rows: Vec<Option<String>>,
    /// The row of the region the cursor is on
    cursor_row: usize,
    /// Whether anything was drawn yet. Until then the region is not ours to clear.
    drawn: bool,
    cursor_hidden: bool,
    /// The terminal was resized, so it may have rewrapped what is on screen
    repaint: bool,
}

impl<W: Write> Screen<W> {
    pub fn new(out: W, width: u16) -> Self {
        Self {
            out,
            width: usize::from(width),
            rows: vec![None],
            cursor_row: 0,
            drawn: false,
            cursor_hidden: false,
            repaint: false,
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    /// Forgets everything about the screen and starts a new region at the cursor.
    pub fn reset(&mut self, width: u16) {
        self.width = usize::from(width);
        self.rows = vec![None];
        self.cursor_row = 0;
        self.drawn = false;
        self.cursor_hidden = false;
        self.repaint = false;
    }

    /// Tells the screen the terminal changed its width. Nothing is written until the next
    /// [`draw`](Self::draw), which then repaints the region inside the same synchronized update.
    pub fn resize(&mut self, width: u16) {
        let width = usize::from(width);
        if width != self.width {
            self.width = width;
            self.repaint = true;
        }
    }

    /// Makes the screen show `lines`, one per row, changing as little as possible.
    ///
    /// Lines may contain styles and hyperlinks. They are cut to the terminal width, since a
    /// wrapped line would break the one line = one row bookkeeping.
    pub fn draw(&mut self, lines: &[String]) -> io::Result<()> {
        // One column is kept free: writing into the last column leaves the cursor in the
        // "pending wrap" state, where erasing the rest of the row would also erase that column.
        let max_cols = self.width.saturating_sub(1);
        let lines: Vec<String> = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines.iter().map(|line| ansi::fit(line, max_cols)).collect()
        };

        let mut body = String::new();

        if !self.cursor_hidden {
            emit(&mut body, Hide);
            self.cursor_hidden = true;
        }
        if self.repaint {
            self.start_over(&mut body);
        }

        self.grow(&mut body, lines.len());

        for (row, line) in lines.iter().enumerate() {
            if self.rows[row].as_deref() != Some(line.as_str()) {
                self.move_to(&mut body, row);
                body.push_str(line);
                emit(&mut body, Clear(ClearType::UntilNewLine));
                self.rows[row] = Some(line.clone());
            }
        }

        self.shrink(&mut body, lines.len());
        self.drawn = true;

        // Park the cursor at the start of its row. Terminals that rewrap on resize then leave it
        // on the first piece of that row, which is what `rows_above_cursor_after_reflow` assumes.
        if !body.is_empty() {
            body.push('\r');
        }

        self.send(&body)
    }

    /// Erases the region, leaves the cursor where it started and shows it again.
    pub fn clear(&mut self) -> io::Result<()> {
        let mut body = String::new();

        if self.drawn {
            if self.repaint {
                self.start_over(&mut body);
            } else {
                body.push('\r');
                move_up(&mut body, self.cursor_row);
                emit(&mut body, Clear(ClearType::FromCursorDown));
            }
        }
        if self.cursor_hidden {
            emit(&mut body, Show);
        }

        self.rows = vec![None];
        self.cursor_row = 0;
        self.drawn = false;
        self.cursor_hidden = false;
        self.repaint = false;

        self.send(&body)
    }

    /// Writes one frame, as a single write. Nothing is sent for an empty frame.
    fn send(&mut self, body: &str) -> io::Result<()> {
        if body.is_empty() {
            return Ok(());
        }

        let mut frame = String::with_capacity(body.len() + 16);
        emit(&mut frame, BeginSynchronizedUpdate);
        frame.push_str(body);
        emit(&mut frame, EndSynchronizedUpdate);

        self.out.write_all(frame.as_bytes())?;
        self.out.flush()
    }

    /// Moves the cursor to the start of `row`.
    fn move_to(&mut self, body: &mut String, row: usize) {
        if row < self.cursor_row {
            move_up(body, self.cursor_row - row);
        } else {
            move_down(body, row - self.cursor_row);
        }
        body.push('\r');
        self.cursor_row = row;
    }

    /// Adds rows at the bottom of the region until it has `target` rows.
    fn grow(&mut self, body: &mut String, target: usize) {
        if target <= self.rows.len() {
            return;
        }

        self.move_to(body, self.rows.len() - 1);
        // A line feed on the last row of the screen scrolls it, which is how the region gets room.
        // Raw mode does not turn it into a carriage return too.
        for _ in self.rows.len()..target {
            body.push_str("\r\n");
            self.cursor_row += 1;
            self.rows.push(None);
        }
    }

    /// Erases the rows below the region once it has shrunk to `target` rows.
    fn shrink(&mut self, body: &mut String, target: usize) {
        if target >= self.rows.len() {
            return;
        }

        self.move_to(body, target);
        emit(body, Clear(ClearType::FromCursorDown));
        move_up(body, 1);
        self.cursor_row = target - 1;
        self.rows.truncate(target);
    }

    /// Erases the region and starts again from its top row, for when the terminal may have
    /// rewrapped it. Never touches the scrollback.
    fn start_over(&mut self, body: &mut String) {
        self.repaint = false;
        if !self.drawn {
            return;
        }

        body.push('\r');
        move_up(body, self.rows_above_cursor_after_reflow());
        emit(body, Clear(ClearType::FromCursorDown));
        self.rows = vec![None];
        self.cursor_row = 0;
    }

    /// How many screen rows are above the cursor once the terminal rewrapped the region to the
    /// current width. Rows we drew always fit the old width, so they only wrap when the terminal
    /// became narrower than the text on them.
    ///
    /// This assumes the terminal rewraps on resize and moves the cursor along with its text, which
    /// every common terminal does. Without any wrapping (the text still fits) it is exact for
    /// terminals that don't rewrap as well.
    fn rows_above_cursor_after_reflow(&self) -> usize {
        (0..self.cursor_row)
            .map(|row| {
                self.rows
                    .get(row)
                    .and_then(Option::as_deref)
                    .map_or(1, |line| {
                        ansi::display_width(line).div_ceil(self.width.max(1)).max(1)
                    })
            })
            .sum()
    }

    #[cfg(test)]
    const fn cursor_row(&self) -> usize {
        self.cursor_row
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::testing::{nz, Sink};

    const BSU: &str = "\u{1b}[?2026h";
    const ESU: &str = "\u{1b}[?2026l";
    const EL: &str = "\u{1b}[K";
    const ED: &str = "\u{1b}[J";

    /// A screen wired to a terminal emulator.
    struct Harness {
        parser: vt100::Parser,
        sink: Sink,
        screen: Screen<Sink>,
        cols: u16,
    }

    impl Harness {
        /// An emulator that already shows `shell` and has its cursor on the line after it.
        fn new(rows: u16, cols: u16, shell: &[&str]) -> Self {
            let mut parser = vt100::Parser::new(nz(rows), nz(cols), 0);
            for line in shell {
                parser.process(line.as_bytes());
                parser.process(b"\r\n");
            }
            let sink = Sink::default();
            Self {
                parser,
                screen: Screen::new(sink.clone(), cols),
                sink,
                cols,
            }
        }

        fn blank(rows: u16, cols: u16) -> Self {
            Self::new(rows, cols, &[])
        }

        /// Draws `lines`, feeds the output to the emulator and returns it.
        fn draw(&mut self, lines: &[&str]) -> String {
            let lines: Vec<String> = lines.iter().map(|&line| line.to_owned()).collect();
            self.screen.draw(&lines).unwrap();
            self.feed()
        }

        fn clear(&mut self) -> String {
            self.screen.clear().unwrap();
            self.feed()
        }

        fn feed(&mut self) -> String {
            let writes = self.sink.take();
            assert!(writes.len() <= 1, "a frame must be a single write");
            let bytes = writes.concat();
            self.parser.process(&bytes);
            String::from_utf8(bytes).unwrap()
        }

        fn text(&self) -> Vec<String> {
            self.parser
                .screen()
                .rows(0, self.cols)
                .map(|row| row.trim_end().to_owned())
                .collect()
        }

        /// The screen row the region starts on
        fn region_top(&self) -> usize {
            usize::from(self.parser.screen().cursor_position().0) - self.screen.cursor_row()
        }

        fn region_text(&self, height: usize) -> Vec<String> {
            let top = self.region_top();
            self.text()[top..top + height].to_vec()
        }

        fn formatted_rows(&self) -> Vec<String> {
            self.parser.screen().rows_formatted(0, self.cols).collect()
        }
    }

    #[test]
    fn first_frame_is_drawn_in_a_single_synchronized_write() {
        let mut term = Harness::blank(10, 40);
        let bytes = term.draw(&["title", "one", "two"]);

        assert!(bytes.starts_with(BSU));
        assert!(bytes.ends_with(ESU));
        assert_eq!(term.text()[..4], ["title", "one", "two", ""]);
    }

    #[test]
    fn unchanged_frames_write_nothing() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["a", "b"]);
        assert_eq!(term.draw(&["a", "b"]), "");
    }

    #[test]
    fn only_the_changed_row_is_rewritten() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["title", "alpha", "beta", "gamma"]);

        let bytes = term.draw(&["title", "alpha", "BETA", "gamma"]);
        assert!(bytes.contains("BETA"));
        for unchanged in ["title", "alpha", "gamma"] {
            assert!(!bytes.contains(unchanged), "{unchanged} was rewritten");
        }
        assert_eq!(term.text()[..4], ["title", "alpha", "BETA", "gamma"]);
    }

    #[test]
    fn moving_a_selection_rewrites_exactly_two_rows() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["> one", "  two", "  three"]);

        let bytes = term.draw(&["  one", "> two", "  three"]);
        assert_eq!(bytes.matches(EL).count(), 2);
        assert!(!bytes.contains(ED));
    }

    #[test]
    fn rows_are_overwritten_before_they_are_erased() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["title", "a long first line", "second"]);

        let bytes = term.draw(&["title", "short", "second"]);
        let text_at = bytes.find("short").unwrap();
        let erase_at = bytes.find(EL).unwrap();
        assert!(text_at < erase_at, "erased before writing: {bytes:?}");
        assert!(!bytes.contains("\u{1b}[2K"), "whole row erased: {bytes:?}");
        assert_eq!(term.text()[..3], ["title", "short", "second"]);
    }

    #[test]
    fn growing_scrolls_the_screen_when_there_is_no_room() {
        let mut term = Harness::new(6, 40, &["s1", "s2", "s3", "s4", "s5"]);
        // the cursor is on the last row, a three row region has to scroll two rows
        term.draw(&["a", "b", "c"]);

        assert_eq!(term.text(), ["s3", "s4", "s5", "a", "b", "c"]);
        assert_eq!(term.region_top(), 3);
    }

    #[test]
    fn shrinking_erases_the_rows_below_and_keeps_the_ones_above() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["a", "b", "c", "d"]);
        term.draw(&["a", "b"]);

        assert_eq!(term.text()[..6], ["shell", "a", "b", "", "", ""]);
        assert_eq!(term.region_top(), 1);
        assert_eq!(term.parser.screen().cursor_position().0, 2);
    }

    #[test]
    fn growing_again_after_shrinking_works() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["a", "b", "c"]);
        term.draw(&["a"]);
        term.draw(&["a", "x", "y", "z"]);

        assert_eq!(term.text()[..5], ["shell", "a", "x", "y", "z"]);
    }

    #[test]
    fn rows_below_the_cursor_are_overwritten_not_trusted() {
        let mut term = Harness::new(10, 40, &["shell"]);
        // left over junk below the cursor, e.g. from a program that drew there earlier
        term.parser
            .process(b"\x1b[2;1Hjunk1\r\njunk2\r\njunk3\x1b[2;1H");

        term.draw(&["a", "b"]);
        assert_eq!(term.text()[..4], ["shell", "a", "b", "junk3"]);

        let mut term = Harness::new(10, 40, &["shell"]);
        term.parser.process(b"\x1b[2;1Hjunk1\r\njunk2\x1b[2;1H");
        term.draw(&["a", "b", "c"]);
        assert_eq!(term.text()[..5], ["shell", "a", "b", "c", ""]);
    }

    #[test]
    fn a_long_row_does_not_get_longer_when_the_text_before_it_changes() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["a very long first line that was drawn earlier", "x"]);
        term.draw(&["short", "x"]);
        assert_eq!(term.text()[..2], ["short", "x"]);
    }

    #[test]
    fn long_lines_are_cut_and_never_wrap() {
        let mut term = Harness::blank(10, 20);
        let long = "x".repeat(60);
        term.draw(&[&long, "after"]);

        assert_eq!(term.text()[0], format!("{}…", "x".repeat(18)));
        assert_eq!(term.text()[1], "after");
        assert!(!term.parser.screen().row_wrapped(0));
    }

    #[test]
    fn the_last_column_stays_free_so_erasing_cannot_eat_text() {
        let mut term = Harness::blank(10, 20);
        // 19 columns is the widest line that is drawn without a cut
        let full = "y".repeat(19);
        term.draw(&[&full, "z"]);
        assert_eq!(term.text()[0], full);
        term.draw(&[&full, "zz"]);
        assert_eq!(term.text()[0], full);
    }

    #[test]
    fn wide_characters_are_not_split_at_the_edge() {
        let mut term = Harness::blank(10, 11);
        term.draw(&["日本語日本語日本語"]);
        assert_eq!(term.text()[0], "日本語日…");
        assert!(!term.parser.screen().row_wrapped(0));
    }

    #[test]
    fn styles_are_kept_and_do_not_leak_into_the_next_row() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["\u{1b}[1;31mred and bold\u{1b}[0m", "plain"]);

        let cell = term.parser.screen().cell(0, 0).unwrap();
        assert!(cell.bold());
        assert_eq!(cell.fgcolor(), vt100::Color::Idx(1));
        let plain = term.parser.screen().cell(1, 0).unwrap();
        assert!(!plain.bold());
        assert_eq!(plain.fgcolor(), vt100::Color::Default);
    }

    #[test]
    fn a_cut_style_is_closed_so_the_next_row_is_plain() {
        let mut term = Harness::blank(10, 12);
        term.draw(&["\u{1b}[1;31mthis is far too long\u{1b}[0m", "plain"]);

        let plain = term.parser.screen().cell(1, 0).unwrap();
        assert!(!plain.bold());
        assert_eq!(plain.fgcolor(), vt100::Color::Default);
    }

    #[test]
    fn hyperlinks_pass_through_untouched() {
        let mut term = Harness::blank(10, 40);
        let link = "\u{1b}]8;;/some/dir\u{1b}\\name\u{1b}]8;;\u{1b}\\";
        let bytes = term.draw(&[link]);
        assert!(bytes.contains(link));
        assert_eq!(term.text()[0], "name");
    }

    #[test]
    fn control_characters_in_lines_cannot_move_the_cursor() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["a\nb\rc\u{8}d", "next"]);
        assert_eq!(term.text()[..2], ["abcd", "next"]);
    }

    #[test]
    fn an_empty_frame_still_owns_one_row() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["a", "b"]);
        term.draw(&[]);
        assert_eq!(term.text()[..3], ["shell", "", ""]);
        assert_eq!(term.region_top(), 1);
    }

    #[test]
    fn the_cursor_is_hidden_once_and_shown_again_on_clear() {
        let mut term = Harness::blank(10, 40);
        let first = term.draw(&["a"]);
        assert!(first.contains("\u{1b}[?25l"));
        let second = term.draw(&["b"]);
        assert!(!second.contains("\u{1b}[?25l"));
        assert!(term.parser.screen().hide_cursor());

        let cleared = term.clear();
        assert!(cleared.contains("\u{1b}[?25h"));
        assert!(!term.parser.screen().hide_cursor());
    }

    #[test]
    fn clear_erases_the_region_and_puts_the_cursor_back_at_its_start() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["a", "b", "c"]);
        term.clear();

        assert_eq!(term.text()[..3], ["shell", "", ""]);
        assert_eq!(term.parser.screen().cursor_position(), (1, 0));
    }

    #[test]
    fn clear_before_anything_was_drawn_leaves_the_screen_alone() {
        let mut term = Harness::new(10, 40, &["shell"]);
        let bytes = term.clear();
        assert_eq!(bytes, "");
        assert_eq!(term.text()[0], "shell");
    }

    #[test]
    fn clear_is_idempotent_and_the_screen_can_be_used_again() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["a", "b"]);
        term.clear();
        assert_eq!(term.clear(), "");

        term.draw(&["c", "d", "e"]);
        assert_eq!(term.text()[..5], ["shell", "c", "d", "e", ""]);
    }

    #[test]
    fn drawing_a_new_directory_over_the_old_one_never_blanks_the_region() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["~/a", "x", "y", "z"]);

        // same height: no row is ever erased without being overwritten first
        let bytes = term.draw(&["~/b", "p", "q", "r"]);
        assert!(!bytes.contains(ED));
        assert!(!bytes.contains("\u{1b}[2K"));
        assert_eq!(term.region_text(4), ["~/b", "p", "q", "r"]);
    }

    #[test]
    fn resizing_repaints_the_region_in_place_in_one_write() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["title", "a", "b"]);

        term.parser.set_size(nz(10), nz(60));
        term.cols = 60;
        term.screen.resize(60);
        let bytes = term.draw(&["title", "a", "b"]);

        assert!(bytes.starts_with(BSU));
        assert!(bytes.ends_with(ESU));
        assert!(bytes.contains(ED));
        assert!(!bytes.contains("\u{1b}[3J"), "scrollback must stay alone");
        assert_eq!(term.text()[..5], ["shell", "title", "a", "b", ""]);
    }

    #[test]
    fn several_resizes_before_a_draw_cost_one_repaint() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["title", "a"]);

        term.screen.resize(50);
        term.screen.resize(45);
        term.screen.resize(30);
        term.parser.set_size(nz(10), nz(30));
        term.cols = 30;
        let bytes = term.draw(&["title", "a"]);

        assert_eq!(bytes.matches(ED).count(), 1);
        assert_eq!(term.text()[..4], ["shell", "title", "a", ""]);
    }

    #[test]
    fn resizing_to_the_same_width_is_not_a_repaint() {
        let mut term = Harness::blank(10, 40);
        term.draw(&["a"]);
        term.screen.resize(40);
        assert_eq!(term.draw(&["a"]), "");
    }

    #[test]
    fn narrowing_cuts_lines_that_no_longer_fit() {
        // The emulator does not rewrap, so the region starts on the first row where the cursor
        // clamps the upward move and the outcome does not depend on rewrapping.
        let mut term = Harness::blank(10, 40);
        let name = "n".repeat(30);
        term.draw(&["title", &name]);

        term.parser.set_size(nz(10), nz(20));
        term.cols = 20;
        term.screen.resize(20);
        term.draw(&["title", &name]);
        assert_eq!(term.text()[1], format!("{}…", "n".repeat(18)));
    }

    #[test]
    fn repaint_accounts_for_rows_a_narrower_terminal_rewrapped() {
        let mut screen = Screen::new(Sink::default(), 40);
        screen.rows = vec![
            Some("title".to_owned()),
            Some("n".repeat(35)),
            Some("short".to_owned()),
        ];
        screen.cursor_row = 2;
        screen.drawn = true;

        // at width 40 nothing wraps
        assert_eq!(screen.rows_above_cursor_after_reflow(), 2);

        // at width 20 the second row takes two screen rows
        screen.width = 20;
        assert_eq!(screen.rows_above_cursor_after_reflow(), 3);

        // the cursor sits at the start of its own row, so that row never counts
        screen.cursor_row = 1;
        assert_eq!(screen.rows_above_cursor_after_reflow(), 1);
        screen.cursor_row = 0;
        assert_eq!(screen.rows_above_cursor_after_reflow(), 0);

        // unknown rows count as one
        screen.cursor_row = 2;
        screen.rows[1] = None;
        assert_eq!(screen.rows_above_cursor_after_reflow(), 2);
    }

    #[test]
    fn resizing_before_the_first_draw_does_not_touch_the_screen() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.screen.resize(50);
        term.draw(&["a"]);
        assert_eq!(term.text()[..2], ["shell", "a"]);
    }

    #[test]
    fn reset_starts_a_fresh_region_at_the_cursor() {
        let mut term = Harness::new(10, 40, &["shell"]);
        term.draw(&["a", "b"]);
        term.clear();
        term.screen.reset(40);
        term.draw(&["c"]);
        assert_eq!(term.text()[..3], ["shell", "c", ""]);
    }

    #[test]
    fn a_tiny_terminal_does_not_panic() {
        for cols in 0..4 {
            let mut screen = Screen::new(Sink::default(), cols);
            screen.draw(&["abc".to_owned(), "def".to_owned()]).unwrap();
            screen.clear().unwrap();
        }
    }

    /// Small deterministic generator, so failures can be reproduced without another dependency.
    struct Rng(u64);

    impl Rng {
        fn step(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.step() % u64::try_from(bound).unwrap()).unwrap()
        }
    }

    fn random_line(rng: &mut Rng, cols: usize) -> String {
        const WORDS: [&str; 10] = [
            "file.txt",
            "Cargo.toml",
            "日本語のファイル",
            "e\u{301}clair",
            "📁 folder",
            "שלום",
            "x",
            "",
            "with space",
            "UPPER",
        ];

        let mut line = String::new();
        for _ in 0..=rng.below(4) {
            let word = WORDS[rng.below(WORDS.len())];
            let styled = match rng.below(5) {
                0 => format!("\u{1b}[1;36m{word}\u{1b}[0m"),
                1 => format!("\u{1b}]8;;/p/{word}\u{1b}\\{word}\u{1b}]8;;\u{1b}\\"),
                _ => word.to_owned(),
            };
            line.push_str(&styled);
            line.push(' ');
        }
        if rng.below(6) == 0 {
            line.push_str(&"w".repeat(cols + rng.below(20)));
        }
        line
    }

    /// Whatever sequence of frames was drawn, the screen must look exactly as if the last frame
    /// had been drawn on a clean terminal, with the shell output above it left alone.
    fn check_frame_sequences(cols: u16, rows: u16, shell_lines: usize, seed: u64) {
        // Short enough to never wrap, even in the narrowest terminal tested
        let shell: Vec<String> = (0..shell_lines).map(|i| format!("sh {i}")).collect();
        let shell_refs: Vec<&str> = shell.iter().map(String::as_str).collect();
        let mut term = Harness::new(rows, cols, &shell_refs);
        let mut rng = Rng(seed);
        let max_height = usize::from(rows) - 1;

        for step in 0..150 {
            let height = 1 + rng.below(max_height);
            let lines: Vec<String> = (0..height)
                .map(|_| random_line(&mut rng, usize::from(cols)))
                .collect();
            let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
            term.draw(&refs);

            let mut reference = Harness::blank(rows, cols);
            reference.draw(&refs);

            let top = term.region_top();
            let context = format!("cols {cols} seed {seed} step {step} height {height}");
            assert_eq!(
                term.formatted_rows()[top..top + height],
                reference.formatted_rows()[..height],
                "region differs, {context}"
            );
            assert!(
                term.text()[top + height..].iter().all(String::is_empty),
                "rows below the region are not blank, {context}: {:?}",
                term.text()
            );
            let above = &term.text()[..top];
            let expected_above = &shell[shell.len().saturating_sub(top)..];
            assert_eq!(above, expected_above, "shell output was touched, {context}");
        }
    }

    #[test]
    fn random_frame_sequences_match_a_clean_redraw_with_room_to_spare() {
        for seed in 1..=6 {
            check_frame_sequences(40, 14, 2, seed);
        }
    }

    #[test]
    fn random_frame_sequences_match_a_clean_redraw_when_scrolling() {
        for seed in 1..=6 {
            check_frame_sequences(40, 14, 13, seed);
            check_frame_sequences(25, 9, 8, seed + 100);
        }
    }

    #[test]
    fn random_frame_sequences_match_with_a_very_narrow_terminal() {
        for seed in 1..=4 {
            check_frame_sequences(7, 10, 3, seed);
        }
    }
}
