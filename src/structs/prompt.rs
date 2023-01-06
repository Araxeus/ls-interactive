use super::{
    prompt_renderer::{TermRenderer, Theme},
    Entry, Filetype,
};
use console::Term;
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use fuzzy_matcher::FuzzyMatcher;
use std::io;
use unicode_segmentation::UnicodeSegmentation;

/// Renders a selection menu that user can fuzzy match to reduce set.
///
/// User can use fuzzy search to limit selectable items.
/// Interaction returns index of an item selected in the order they appear in `item` invocation or `items` slice.
pub struct Prompt<'a> {
    default: usize,
    items: Vec<Entry>,
    title: String,
    report: bool,
    clear: bool,
    highlight_matches: bool,
    theme: &'a Theme,
}

impl Prompt<'_> {
    /// Adds multiple items to the fuzzy selector.
    pub fn items(&mut self, items: &[Entry]) -> &mut Self {
        let its = items.to_owned();
        for item in its {
            self.items.push(item);
        }
        self
    }

    /// Prefaces the menu with a prompt.
    ///
    /// When a prompt is set the system also prints out a confirmation after
    /// the fuzzy selection.
    pub fn title<S: Into<String>>(&mut self, title: S) -> &mut Self {
        self.title = title.into();
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// The user can select the items using 'Enter' and the index of selected item will be returned.
    /// The dialog is rendered on stderr.
    /// Result contains `Some(index)` if user hit 'Enter' or `None` if user cancelled with 'Esc' or 'q'.
    #[inline]
    pub fn run(&self) -> io::Result<Option<(usize, KeyModifiers)>> {
        self._run(&Term::stderr(), true)
    }

    /// Like `interact` but allows a specific terminal to be set.
    /// Ignore `clippy::too-many-lines`
    #[allow(clippy::too_many_lines)] // TODO: refactor
    fn _run(&self, term: &Term, allow_quit: bool) -> io::Result<Option<(usize, KeyModifiers)>> {
        // This cursor iterates over the graphemes vec rather than the search term
        let mut cursor_pos = 0;
        let mut search_term: Vec<String> = Vec::new();

        let mut render = TermRenderer::new(term, self.theme);
        let mut sel = self.default;

        let mut size_vec = Vec::new();
        for item in self.items.iter().as_slice() {
            let size = &item.name.len() + 2;
            size_vec.push(size);
        }

        // Fuzzy matcher
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        // Subtract -2 because we need space to render the prompt.
        let visible_term_rows = (term.size().0 as usize).max(3) - 2;
        // Variable used to determine if we need to scroll through the list.
        let mut starting_row = 0;

        term.hide_cursor()?;

        loop {
            let concatted_search_term = search_term.concat();
            search_term = concatted_search_term
                .graphemes(true)
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();

            render.clear()?;
            render.fuzzy_select_prompt(self.title.as_str(), &search_term, cursor_pos)?;

            // Maps all items to a tuple of item and its match score.
            let mut filtered_list = self
                .items
                .iter()
                .map(|item| (item, matcher.fuzzy_match(&item.name, &search_term.concat())))
                .filter_map(|(item, score)| score.map(|s| (item, s)))
                .collect::<Vec<_>>();

            // Renders all matching items, from best match to worst.
            filtered_list.sort_unstable_by(|(_, s1), (_, s2)| s2.cmp(s1));

            for (idx, (item, _)) in filtered_list
                .iter()
                .enumerate()
                .skip(starting_row)
                .take(visible_term_rows)
            {
                render.fuzzy_select_prompt_item(
                    item,
                    idx == sel,
                    self.highlight_matches,
                    &matcher,
                    &search_term.concat(),
                )?;
                term.flush()?;
            }

            terminal::enable_raw_mode()?;

            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = read().unwrap()
            {
                match code {
                    KeyCode::Esc if allow_quit => {
                        if self.clear {
                            render.clear()?;
                            term.flush()?;
                        }
                        terminal::disable_raw_mode()?;
                        term.show_cursor()?;
                        return Ok(None);
                    }
                    KeyCode::Up | KeyCode::BackTab if !filtered_list.is_empty() => {
                        if sel == 0 {
                            starting_row =
                                filtered_list.len().max(visible_term_rows) - visible_term_rows;
                        } else if sel == starting_row {
                            starting_row -= 1;
                        }

                        sel = (sel + filtered_list.len() - 1) % filtered_list.len();
                        term.flush()?;
                    }
                    KeyCode::Down | KeyCode::Tab if !filtered_list.is_empty() => {
                        sel = (sel + filtered_list.len() + 1) % filtered_list.len();
                        if sel == visible_term_rows + starting_row {
                            starting_row += 1;
                        } else if sel == 0 {
                            starting_row = 0;
                        }
                        term.flush()?;
                    }
                    KeyCode::Left => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            term.flush()?;
                        } else if search_term.is_empty() && self.items[0].name.ends_with("..") {
                            if self.clear {
                                render.clear()?;
                            }

                            terminal::disable_raw_mode()?;

                            term.show_cursor()?;

                            return Ok(Some((0, KeyModifiers::NONE)));
                        }
                    }
                    KeyCode::Right => {
                        if cursor_pos < search_term.len() {
                            cursor_pos += 1;
                            term.flush()?;
                        } else if search_term.is_empty()
                            && (filtered_list[sel].0.filetype == Filetype::Directory)
                            || (cfg!(windows)
                                && filtered_list[sel].0.filetype == Filetype::DriveView)
                        {
                            if self.clear {
                                render.clear()?;
                            }

                            let sel_string_pos_in_items = self
                                .items
                                .iter()
                                .position(|item| item.name.eq(&filtered_list[sel].0.name))
                                .unwrap();

                            terminal::disable_raw_mode()?;

                            term.show_cursor()?;

                            return Ok(Some((sel_string_pos_in_items, KeyModifiers::NONE)));
                        }
                    }
                    KeyCode::Enter if !filtered_list.is_empty() => {
                        if self.clear {
                            render.clear()?;
                        }

                        if self.report {
                            render.input_prompt_selection(
                                self.title.as_str(),
                                &filtered_list[sel].0.name,
                            )?;
                        }

                        let sel_string_pos_in_items = self
                            .items
                            .iter()
                            .position(|item| item.name.eq(&filtered_list[sel].0.name))
                            .unwrap();

                        terminal::disable_raw_mode()?;

                        term.show_cursor()?;

                        return Ok(Some((sel_string_pos_in_items, modifiers)));
                    }
                    KeyCode::Backspace if cursor_pos > 0 => {
                        cursor_pos -= 1;
                        search_term.remove(cursor_pos);
                        term.flush()?;
                    }
                    KeyCode::Char(chr) if !chr.is_ascii_control() => {
                        search_term.insert(cursor_pos, chr.to_string());
                        cursor_pos += 1;
                        term.flush()?;
                        sel = 0;
                        starting_row = 0;
                    }

                    KeyCode::Null => render.error("Unrecognized Key").unwrap(),

                    _ => {}
                }
            }

            terminal::disable_raw_mode()?;
            render.clear_preserve_prompt(&size_vec)?;
        }
    }
}

impl<'a> Prompt<'a> {
    /// Same as `new` but with a specific theme.
    pub const fn with_theme(theme: &'a Theme) -> Self {
        Self {
            default: 0,
            items: vec![],
            title: String::new(),
            report: false,
            clear: true,
            highlight_matches: true,
            theme,
        }
    }
}
