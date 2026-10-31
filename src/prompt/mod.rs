//! The interactive prompt: reads keys, updates the [`Selector`] and keeps the [`Screen`] in sync.
//!
//! One [`Ui`] lives for the whole session. Moving into another folder just calls
//! [`Ui::choose`] again, so the new list is drawn over the old one without the region ever being
//! cleared in between.

mod selector;
mod terminal;
mod theme;

use std::{
    io::{self, Stderr, Write},
    time::Instant,
};

use crossterm::event::{Event, KeyModifiers, MouseEventKind};

use crate::{entry::Entry, render::Screen};
use selector::{Outcome, Selector};
pub use terminal::restore_terminal;
use terminal::{Backend, Tty};
use theme::Theme;

pub struct Ui<W: Write = Stderr, B: Backend = Tty> {
    screen: Screen<W>,
    backend: B,
    theme: Theme,
    /// Whether raw mode is on and a region may be on screen
    active: bool,
    /// Whether the terminal ignored a cursor position query this session
    cursor_unanswered: bool,
}

impl Ui {
    pub fn new() -> Self {
        #[cfg(windows)]
        terminal::enable_ansi_output();
        Self::with(io::stderr(), Tty)
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Write, B: Backend> Ui<W, B> {
    pub fn with(out: W, backend: B) -> Self {
        Self {
            screen: Screen::new(out, 0, 0),
            backend,
            theme: Theme::default(),
            active: false,
            cursor_unanswered: false,
        }
    }

    /// Lets the user pick one of `items`. Returns its index and the modifiers held, or `None` if
    /// the user quit.
    ///
    /// The list stays on screen when an item is chosen, so the next call can draw over it. Call
    /// [`finish`](Self::finish) before printing anything else. Quitting and errors clean up on
    /// their own.
    pub fn choose(
        &mut self,
        items: &[Entry],
        title: &str,
    ) -> io::Result<Option<(usize, KeyModifiers)>> {
        match self.run(items, title) {
            Ok(None) => {
                self.finish()?;
                Ok(None)
            }
            Err(err) => {
                let _ = self.finish();
                Err(err)
            }
            chosen => chosen,
        }
    }

    /// Removes the prompt from the screen and gives the terminal back.
    pub fn finish(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;

        let cleared = self.screen.clear();
        let mouse = self.backend.set_mouse_capture(false);
        let raw_mode = self.backend.set_raw_mode(false);
        cleared.and(mouse).and(raw_mode)
    }

    fn begin(&mut self) -> io::Result<u16> {
        let (cols, rows) = self.backend.size()?;
        if self.active {
            self.screen.resize(cols, rows);
        } else {
            self.backend.set_raw_mode(true)?;
            self.screen.reset(cols, rows);
            self.active = true;
            self.cursor_unanswered = false;
            self.backend.set_mouse_capture(true)?;
        }
        Ok(rows)
    }

    fn run(&mut self, items: &[Entry], title: &str) -> io::Result<Option<(usize, KeyModifiers)>> {
        let rows = self.begin()?;
        let mut selector = Selector::new(items, list_rows(rows));

        loop {
            let lines = frame(&self.theme, &selector, title, self.screen.width());
            self.screen.draw(&lines)?;

            // Everything that is already waiting is handled before drawing again, so typing or
            // holding a key down on a slow connection costs one frame per burst, not per key.
            let mut next = Some(self.backend.read()?);
            while let Some(event) = next.take() {
                let outcome = match event {
                    Event::Key(key) => selector.handle_key(key),
                    Event::Mouse(mouse) => {
                        let row = if matches!(mouse.kind, MouseEventKind::Down(_)) {
                            self.list_row(mouse.row)?
                        } else {
                            None
                        };
                        selector.handle_mouse(mouse, row, Instant::now())
                    }
                    Event::Resize(cols, rows) => {
                        self.screen.resize(cols, rows);
                        selector.set_rows(list_rows(rows));
                        Outcome::Pending
                    }
                    _ => Outcome::Pending,
                };
                match outcome {
                    Outcome::Pending => {}
                    Outcome::Quit => return Ok(None),
                    Outcome::Choose { index, modifiers } => return Ok(Some((index, modifiers))),
                }

                if self.backend.has_pending()? {
                    next = Some(self.backend.read()?);
                }
            }
        }
    }

    /// The list row shown on screen row `row`. Clicks come in screen rows, but the region only
    /// knows where it is relative to the cursor, so the first click asks the terminal where the
    /// cursor is. Starting the prompt never waits for that answer.
    fn list_row(&mut self, row: u16) -> io::Result<Option<usize>> {
        if !self.screen.is_anchored() && !self.cursor_unanswered {
            match self.backend.cursor_position()? {
                Some((_, cursor_row)) => self.screen.anchor(cursor_row),
                None => self.cursor_unanswered = true,
            }
        }
        // The prompt is on the first row of the region
        Ok(self
            .screen
            .region_row(row)
            .and_then(|row| row.checked_sub(1)))
    }
}

impl<W: Write, B: Backend> Drop for Ui<W, B> {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

/// How many list rows fit: the prompt takes one row and one more is left for the cursor.
fn list_rows(terminal_rows: u16) -> usize {
    usize::from(terminal_rows).max(3) - 2
}

/// The lines of one frame: the prompt, then the visible part of the list.
fn frame(theme: &Theme, selector: &Selector, title: &str, width: usize) -> Vec<String> {
    let mut lines = vec![theme.prompt_line(
        title,
        selector.term(),
        selector.cursor(),
        width.saturating_sub(1),
    )];
    lines.extend(
        selector
            .visible()
            .map(|row| theme.item_line(row.entry, row.active, &selector.highlights(row.item))),
    );
    lines
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent};

    use super::*;
    use crate::{
        entry::{Filetype, Icons},
        render::{
            ansi,
            testing::{nz, Sink},
        },
    };

    /// What a scripted terminal did, shared with the test.
    #[derive(Default)]
    struct State {
        events: VecDeque<Event>,
        size: (u16, u16),
        raw_mode: Vec<bool>,
        mouse_capture: Vec<bool>,
        /// Whether cursor queries get an answer
        answers: bool,
        cursor_queries: usize,
        /// What the terminal showed before the prompt started
        shell: Vec<u8>,
        /// Everything the prompt wrote, to answer cursor queries from
        output: Sink,
    }

    #[derive(Clone)]
    struct Scripted(Rc<RefCell<State>>);

    impl Backend for Scripted {
        fn size(&self) -> io::Result<(u16, u16)> {
            Ok(self.0.borrow().size)
        }

        fn set_raw_mode(&mut self, enabled: bool) -> io::Result<()> {
            self.0.borrow_mut().raw_mode.push(enabled);
            Ok(())
        }

        fn set_mouse_capture(&mut self, enabled: bool) -> io::Result<()> {
            self.0.borrow_mut().mouse_capture.push(enabled);
            Ok(())
        }

        fn cursor_position(&mut self) -> io::Result<Option<(u16, u16)>> {
            let mut state = self.0.borrow_mut();
            state.cursor_queries += 1;
            if !state.answers {
                return Ok(None);
            }
            let (cols, rows) = state.size;
            let mut terminal = vt100::Parser::new(nz(rows), nz(cols), 0);
            terminal.process(&state.shell);
            terminal.process(&state.output.log());
            let (row, column) = terminal.screen().cursor_position();
            Ok(Some((column, row)))
        }

        fn read(&mut self) -> io::Result<Event> {
            self.0
                .borrow_mut()
                .events
                .pop_front()
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "script ran out"))
        }

        fn has_pending(&mut self) -> io::Result<bool> {
            Ok(!self.0.borrow().events.is_empty())
        }
    }

    struct Harness {
        ui: Ui<Sink, Scripted>,
        state: Rc<RefCell<State>>,
        sink: Sink,
        parser: vt100::Parser,
    }

    impl Harness {
        fn new(rows: u16, cols: u16) -> Self {
            let sink = Sink::default();
            let state = Rc::new(RefCell::new(State {
                size: (cols, rows),
                answers: true,
                output: sink.clone(),
                ..State::default()
            }));
            Self {
                ui: Ui::with(sink.clone(), Scripted(state.clone())),
                state,
                sink,
                parser: vt100::Parser::new(nz(rows), nz(cols), 0),
            }
        }

        /// Shows `text` before the prompt starts, as the shell would.
        fn shell(&mut self, text: &[u8]) {
            self.parser.process(text);
            self.state.borrow_mut().shell.extend_from_slice(text);
        }

        fn queue(&self, events: impl IntoIterator<Item = Event>) {
            self.state.borrow_mut().events.extend(events);
        }

        /// Runs `choose` and feeds what it drew to the emulator. Returns the result and the number
        /// of writes it took.
        fn choose(
            &mut self,
            items: &[Entry],
            title: &str,
        ) -> (io::Result<Option<(usize, KeyModifiers)>>, usize) {
            let result = self.ui.choose(items, title);
            let writes = self.sink.take();
            for write in &writes {
                self.parser.process(write);
            }
            (result, writes.len())
        }

        fn flush_output(&mut self) -> usize {
            let writes = self.sink.take();
            for write in &writes {
                self.parser.process(write);
            }
            writes.len()
        }

        fn screen(&self) -> Vec<String> {
            let (rows, cols) = self.parser.screen().size();
            self.parser
                .screen()
                .rows(0, cols.get())
                .take(usize::from(rows.get()))
                .map(|row| ansi::strip(&row).trim_end().to_owned())
                .collect()
        }

        fn raw_mode(&self) -> Vec<bool> {
            self.state.borrow().raw_mode.clone()
        }
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn chars(text: &str) -> Vec<Event> {
        text.chars().map(|c| key(KeyCode::Char(c))).collect()
    }

    fn entry(name: &str, filetype: Filetype) -> Entry {
        Entry {
            name: name.to_owned(),
            path: format!("/dir/{name}"),
            icon: &Icons::TXT,
            filetype,
        }
    }

    fn entries(names: &[&str]) -> Vec<Entry> {
        names
            .iter()
            .map(|name| entry(name, Filetype::Unknown))
            .collect()
    }

    fn row_text(row: &str) -> String {
        row.replace(&Icons::TXT.to_string(), "I")
            .replace(&Icons::DIR.to_string(), "D")
    }

    #[test]
    fn draws_the_prompt_and_the_list_then_returns_the_choice() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Down), key(KeyCode::Enter)]);

        let (result, _) = term.choose(&entries(&["alpha", "beta"]), "/tmp");
        assert_eq!(result.unwrap(), Some((1, KeyModifiers::NONE)));

        // Down and Enter arrive as one burst, so only the first frame was drawn
        let screen: Vec<_> = term.screen().iter().map(|r| row_text(r)).collect();
        assert_eq!(screen[0], "? /tmp ›");
        assert_eq!(screen[1], "❯ I alpha");
        assert_eq!(screen[2], "  I beta");
    }

    #[test]
    fn a_burst_of_keys_costs_one_frame() {
        let mut term = Harness::new(10, 40);
        let mut script = chars("alph");
        script.push(key(KeyCode::Enter));
        term.queue(script);

        let (result, writes) = term.choose(&entries(&["alpha", "beta"]), "/tmp");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
        // the initial frame, and the one for the whole burst that ends with Enter
        assert_eq!(
            writes, 1,
            "keys that are already waiting must not be drawn one by one"
        );
    }

    #[test]
    fn keys_that_arrive_one_at_a_time_are_drawn_one_at_a_time() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Down)]);

        // run the first iteration by hand: the script runs dry after Down
        let items = entries(&["a", "b", "c"]);
        let result = term.ui.choose(&items, "/tmp");
        assert_eq!(
            result.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof,
            "the scripted terminal ran out of events"
        );
        // initial frame + the frame after Down
        assert_eq!(
            term.flush_output(),
            2 + 1 /* cleanup after the error */
        );
    }

    #[test]
    fn typing_filters_the_visible_list() {
        let mut term = Harness::new(10, 40);
        term.queue(chars("be"));
        let items = entries(&["alpha", "beta", "gamma"]);

        // the script runs dry after the burst, so the last write is the cleanup
        let _ = term.ui.choose(&items, "/tmp");
        let mut writes = term.sink.take();
        assert_eq!(writes.len(), 3, "initial frame, filtered frame, cleanup");
        writes.pop();
        for write in &writes {
            term.parser.process(write);
        }

        let screen: Vec<_> = term.screen().iter().map(|r| row_text(r)).collect();
        assert_eq!(screen[..3], ["? /tmp › be", "❯ I beta", ""]);
    }

    #[test]
    fn the_filtered_frame_shows_only_matches() {
        let theme = Theme::default();
        let items = entries(&["alpha", "beta", "gamma"]);
        let mut selector = Selector::new(&items, 10);
        for c in "be".chars() {
            selector.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        let lines = frame(&theme, &selector, "/tmp", 40);
        let text: Vec<_> = lines.iter().map(|l| row_text(&ansi::strip(l))).collect();
        assert_eq!(text, ["? /tmp › be ", "❯ I beta"]);
    }

    #[test]
    fn quitting_clears_the_prompt_and_restores_the_terminal() {
        let mut term = Harness::new(10, 40);
        term.parser.process(b"shell prompt\r\n");
        term.queue([key(KeyCode::Char('x')), key(KeyCode::Esc)]);

        let (result, _) = term.choose(&entries(&["a", "b"]), "/tmp");
        assert_eq!(result.unwrap(), None);

        assert_eq!(term.screen()[0], "shell prompt");
        assert!(term.screen()[1..].iter().all(String::is_empty));
        assert!(!term.parser.screen().hide_cursor());
        assert_eq!(term.raw_mode(), [true, false]);
    }

    #[test]
    fn a_chosen_folder_stays_on_screen_for_the_next_list() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Enter)]);
        let first = vec![
            entry("one", Filetype::Directory),
            entry("two", Filetype::Unknown),
        ];
        let (result, _) = term.choose(&first, "/a");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));

        // still drawn, still raw, cursor still hidden: nothing was torn down
        assert_eq!(row_text(&term.screen()[0]), "? /a ›");
        assert_eq!(term.raw_mode(), [true]);
        assert!(term.parser.screen().hide_cursor());

        term.queue([key(KeyCode::Enter)]);
        let second = entries(&["x", "y", "z"]);
        let (result, writes) = term.choose(&second, "/a/one");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
        assert_eq!(writes, 1);

        let screen: Vec<_> = term.screen().iter().map(|r| row_text(r)).collect();
        assert_eq!(screen[..4], ["? /a/one ›", "❯ I x", "  I y", "  I z"]);
        assert_eq!(term.raw_mode(), [true], "raw mode stays on between folders");
    }

    #[test]
    fn switching_folders_never_erases_the_screen_before_drawing() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Enter)]);
        let first = entries(&["a", "b", "c"]);
        term.choose(&first, "/a").0.unwrap();
        term.sink.take();

        term.queue([key(KeyCode::Enter)]);
        let second = entries(&["x", "y", "z"]);
        let _ = term.ui.choose(&second, "/b");
        let writes = term.sink.take();
        let bytes = String::from_utf8(writes.concat()).unwrap();
        assert!(
            !bytes.contains("\u{1b}[J"),
            "the region was erased: {bytes:?}"
        );
        assert!(!bytes.contains("\u{1b}[2K"), "a row was blanked: {bytes:?}");
    }

    #[test]
    fn finish_removes_the_prompt_and_is_safe_to_repeat() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Enter)]);
        term.choose(&entries(&["a", "b"]), "/a").0.unwrap();

        term.ui.finish().unwrap();
        assert_eq!(term.flush_output(), 1);
        assert!(term.screen().iter().all(String::is_empty));
        assert!(!term.parser.screen().hide_cursor());

        term.ui.finish().unwrap();
        assert_eq!(term.flush_output(), 0);
        assert_eq!(term.raw_mode(), [true, false]);
    }

    #[test]
    fn the_ui_can_be_used_again_after_finish() {
        let mut term = Harness::new(10, 40);
        term.parser.process(b"shell\r\n");
        term.queue([key(KeyCode::Enter)]);
        term.choose(&entries(&["a"]), "/a").0.unwrap();
        term.ui.finish().unwrap();
        term.flush_output();

        term.queue([key(KeyCode::Enter)]);
        term.choose(&entries(&["b"]), "/b").0.unwrap();
        assert_eq!(term.screen()[0], "shell");
        assert_eq!(row_text(&term.screen()[1]), "? /b ›");
        assert_eq!(term.raw_mode(), [true, false, true]);
    }

    #[test]
    fn dropping_the_ui_cleans_up() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Enter)]);
        term.choose(&entries(&["a"]), "/a").0.unwrap();

        let Harness {
            ui,
            state,
            sink,
            mut parser,
        } = term;
        drop(ui);
        for write in sink.take() {
            parser.process(&write);
        }
        assert!(!parser.screen().hide_cursor());
        assert_eq!(state.borrow().raw_mode, [true, false]);
    }

    #[test]
    fn errors_clean_up_too() {
        let mut term = Harness::new(10, 40);
        let (result, _) = term.choose(&entries(&["a"]), "/a");
        assert!(result.is_err());
        assert!(!term.parser.screen().hide_cursor());
        assert_eq!(term.raw_mode(), [true, false]);
    }

    #[test]
    fn resizing_shrinks_the_list_to_the_new_height() {
        let mut term = Harness::new(12, 40);
        let items: Vec<_> = (0..20)
            .map(|i| entry(&format!("f{i:02}"), Filetype::Unknown))
            .collect();
        term.queue([Event::Resize(40, 6), key(KeyCode::Enter)]);

        let (result, _) = term.choose(&items, "/a");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
        let drawn = term.screen().iter().filter(|row| !row.is_empty()).count();
        // 12 rows at the start: prompt + 10 items; the resize came before the next frame was drawn
        assert_eq!(drawn, 11);
    }

    #[test]
    fn a_resize_repaints_inside_one_frame() {
        let items = entries(&["a", "b", "c"]);
        let mut term = Harness::new(10, 40);
        term.queue([
            key(KeyCode::Down),
            Event::Resize(20, 10),
            key(KeyCode::Enter),
        ]);
        let (result, writes) = term.choose(&items, "/a");
        assert_eq!(result.unwrap(), Some((1, KeyModifiers::NONE)));
        assert_eq!(writes, 1, "Down and the resize arrive together: one frame");
    }

    fn click(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn a_click_selects_the_row_under_the_pointer() {
        let mut term = Harness::new(10, 40);
        term.shell(b"$ cd\r\n$ lsi\r\n");
        // the region starts on row 2: the prompt, then a, b and c
        term.queue([click(4, 5), key(KeyCode::Enter)]);
        let (result, _) = term.choose(&entries(&["a", "b", "c"]), "/tmp");
        assert_eq!(result.unwrap(), Some((2, KeyModifiers::NONE)));
    }

    #[test]
    fn a_double_click_chooses_the_row() {
        let mut term = Harness::new(10, 40);
        term.shell(b"$ lsi\r\n");
        term.queue([click(4, 3), click(4, 3)]);
        let (result, _) = term.choose(&entries(&["a", "b", "c"]), "/tmp");
        assert_eq!(result.unwrap(), Some((1, KeyModifiers::NONE)));
    }

    #[test]
    fn clicks_follow_the_region_when_the_shell_scrolled() {
        let mut term = Harness::new(10, 40);
        for i in 0..9 {
            term.shell(format!("sh {i}\r\n").as_bytes());
        }
        // the region scrolled up to rows 6..10, the list is on rows 7, 8 and 9
        term.queue([click(4, 8), click(4, 8)]);
        let (result, _) = term.choose(&entries(&["a", "b", "c"]), "/tmp");
        assert_eq!(result.unwrap(), Some((1, KeyModifiers::NONE)));
    }

    #[test]
    fn the_cursor_is_asked_for_on_the_first_click_only() {
        let mut term = Harness::new(10, 40);
        term.shell(b"$ lsi\r\n");
        term.queue([key(KeyCode::Down), key(KeyCode::Enter)]);
        term.choose(&entries(&["a", "b"]), "/a").0.unwrap();
        assert_eq!(
            term.state.borrow().cursor_queries,
            0,
            "no click, no question"
        );

        term.queue([click(4, 2), click(4, 3), click(4, 3)]);
        let (result, _) = term.choose(&entries(&["x", "y"]), "/b");
        assert_eq!(result.unwrap(), Some((1, KeyModifiers::NONE)));
        assert_eq!(term.state.borrow().cursor_queries, 1);
    }

    #[test]
    fn clicks_do_nothing_when_the_terminal_does_not_answer() {
        let mut term = Harness::new(10, 40);
        term.state.borrow_mut().answers = false;
        term.queue([click(4, 2), click(4, 2), key(KeyCode::Enter)]);
        let (result, _) = term.choose(&entries(&["a", "b"]), "/tmp");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
        assert_eq!(
            term.state.borrow().cursor_queries,
            1,
            "asked once, not per click"
        );
    }

    #[test]
    fn clicks_on_the_prompt_or_the_shell_are_ignored() {
        let mut term = Harness::new(10, 40);
        term.shell(b"$ lsi\r\n");
        term.queue([click(4, 0), click(4, 1), click(4, 1), key(KeyCode::Enter)]);
        let (result, _) = term.choose(&entries(&["a", "b"]), "/tmp");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
    }

    #[test]
    fn the_mouse_is_captured_while_the_prompt_is_open() {
        let mut term = Harness::new(10, 40);
        term.queue([key(KeyCode::Enter)]);
        term.choose(&entries(&["a"]), "/a").0.unwrap();
        assert_eq!(term.state.borrow().mouse_capture, [true]);

        term.queue([key(KeyCode::Enter)]);
        term.choose(&entries(&["b"]), "/b").0.unwrap();
        assert_eq!(
            term.state.borrow().mouse_capture,
            [true],
            "kept between folders"
        );

        term.ui.finish().unwrap();
        assert_eq!(term.state.borrow().mouse_capture, [true, false]);
    }

    #[test]
    fn non_key_events_are_ignored() {
        let mut term = Harness::new(10, 40);
        term.queue([Event::FocusGained, Event::FocusLost, key(KeyCode::Enter)]);
        let (result, writes) = term.choose(&entries(&["a"]), "/a");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
        assert_eq!(writes, 1);
    }

    #[test]
    fn a_tiny_terminal_still_works() {
        let mut term = Harness::new(2, 5);
        term.queue([key(KeyCode::Enter)]);
        let items = entries(&["abcdefghij", "b"]);
        let (result, _) = term.choose(&items, "/a/long/path");
        assert_eq!(result.unwrap(), Some((0, KeyModifiers::NONE)));
    }

    #[test]
    fn list_rows_leaves_room_for_the_prompt_and_the_cursor() {
        assert_eq!(list_rows(24), 22);
        assert_eq!(list_rows(3), 1);
        assert_eq!(list_rows(0), 1);
    }
}
