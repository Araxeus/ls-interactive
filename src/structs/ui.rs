//! The interactive prompt: reads keys, updates the [`Selector`] and keeps the [`Screen`] in sync.
//!
//! One [`Ui`] lives for the whole session. Moving into another folder just calls
//! [`Ui::choose`] again, so the new list is drawn over the old one without the region ever being
//! cleared in between.

use std::{
    io::{self, Stderr, Write},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyModifiers},
    terminal,
};

use super::{
    screen::Screen,
    selector::{Outcome, Selector},
    theme::Theme,
    Entry,
};

/// The terminal side of the prompt: its size, raw mode and input events.
pub trait Backend {
    fn size(&self) -> io::Result<(u16, u16)>;
    fn set_raw_mode(&mut self, enabled: bool) -> io::Result<()>;
    /// Blocks until the next event.
    fn read(&mut self) -> io::Result<Event>;
    /// Whether [`read`](Self::read) would return without waiting.
    fn has_pending(&mut self) -> io::Result<bool>;
}

/// The real terminal.
pub struct Tty;

impl Backend for Tty {
    fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    fn set_raw_mode(&mut self, enabled: bool) -> io::Result<()> {
        if enabled {
            terminal::enable_raw_mode()
        } else {
            terminal::disable_raw_mode()
        }
    }

    fn read(&mut self) -> io::Result<Event> {
        event::read()
    }

    fn has_pending(&mut self) -> io::Result<bool> {
        event::poll(Duration::ZERO)
    }
}

/// Windows consoles only understand escape sequences once virtual terminal processing is turned
/// on for the handle that is written to. The prompt draws on stderr, since stdout carries the
/// result.
#[cfg(windows)]
fn enable_ansi_output() {
    use windows::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE,
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_ERROR_HANDLE,
    };

    // SAFETY: plain Win32 calls on the process's own stderr handle
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_ERROR_HANDLE) {
            let mut mode = CONSOLE_MODE::default();
            if GetConsoleMode(handle, &raw mut mode).is_ok() {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

/// Puts the terminal back in a usable state. For panic hooks, where the [`Ui`] cannot do it.
pub fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let mut stderr = io::stderr();
    let _ = stderr.write_all(b"\x1b[?25h");
    let _ = stderr.flush();
}

pub struct Ui<W: Write = Stderr, B: Backend = Tty> {
    screen: Screen<W>,
    backend: B,
    theme: Theme,
    /// Whether raw mode is on and a region may be on screen
    active: bool,
}

impl Ui {
    pub fn new() -> Self {
        #[cfg(windows)]
        enable_ansi_output();
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
            screen: Screen::new(out, 0),
            backend,
            theme: Theme::default(),
            active: false,
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
        let raw_mode = self.backend.set_raw_mode(false);
        cleared.and(raw_mode)
    }

    fn begin(&mut self) -> io::Result<u16> {
        let (cols, rows) = self.backend.size()?;
        if self.active {
            self.screen.resize(cols);
        } else {
            self.backend.set_raw_mode(true)?;
            self.screen.reset(cols);
            self.active = true;
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
                match event {
                    Event::Key(key) => match selector.handle_key(key) {
                        Outcome::Pending => {}
                        Outcome::Quit => return Ok(None),
                        Outcome::Choose { index, modifiers } => {
                            return Ok(Some((index, modifiers)))
                        }
                    },
                    Event::Resize(cols, rows) => {
                        self.screen.resize(cols);
                        selector.set_rows(list_rows(rows));
                    }
                    _ => {}
                }

                if self.backend.has_pending()? {
                    next = Some(self.backend.read()?);
                }
            }
        }
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

    use crossterm::event::{KeyCode, KeyEvent};

    use super::*;
    use crate::structs::{
        ansi,
        testing::{nz, Sink},
        Filetype, Icons,
    };

    /// What a scripted terminal did, shared with the test.
    #[derive(Default)]
    struct State {
        events: VecDeque<Event>,
        size: (u16, u16),
        raw_mode: Vec<bool>,
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
            let state = Rc::new(RefCell::new(State {
                size: (cols, rows),
                ..State::default()
            }));
            let sink = Sink::default();
            Self {
                ui: Ui::with(sink.clone(), Scripted(state.clone())),
                state,
                sink,
                parser: vt100::Parser::new(nz(rows), nz(cols), 0),
            }
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
