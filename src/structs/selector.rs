//! Selection state of the prompt: search term, fuzzy filtering, selection and scrolling.
//!
//! Nothing in here draws anything, which keeps the key handling easy to test.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use unicode_segmentation::UnicodeSegmentation;

use super::{Entry, Filetype};

/// What a key press did to the selector.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Keep prompting
    Pending,
    /// The user asked to leave without choosing anything
    Quit,
    /// The user chose the item at `index` of the item list
    Choose {
        index: usize,
        modifiers: KeyModifiers,
    },
}

/// One visible row of the list.
pub struct Row<'s> {
    pub item: usize,
    pub entry: &'s Entry,
    pub active: bool,
}

pub struct Selector<'a> {
    items: &'a [Entry],
    matcher: SkimMatcherV2,
    /// The search term as grapheme clusters, so the cursor never lands inside a character
    term: Vec<String>,
    /// `term` joined, the pattern the cached `matches` were computed for
    pattern: String,
    /// Cursor position inside `term`
    cursor: usize,
    /// Indices into `items` with their score, best match first. Only recomputed when the pattern
    /// changes, moving the selection or the cursor never touches it.
    matches: Vec<(usize, i64)>,
    /// Position inside `matches`
    selected: usize,
    /// First visible position inside `matches`
    top: usize,
    /// How many matches fit on screen
    rows: usize,
    #[cfg(test)]
    recomputes: usize,
}

impl<'a> Selector<'a> {
    pub fn new(items: &'a [Entry], rows: usize) -> Self {
        let mut selector = Self {
            items,
            matcher: SkimMatcherV2::default(),
            term: Vec::new(),
            pattern: String::new(),
            cursor: 0,
            matches: Vec::new(),
            selected: 0,
            top: 0,
            rows: rows.max(1),
            #[cfg(test)]
            recomputes: 0,
        };
        selector.recompute();
        selector
    }

    pub fn term(&self) -> &[String] {
        &self.term
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// Changes how many matches fit on screen, keeping the selection visible.
    pub fn set_rows(&mut self, rows: usize) {
        self.rows = rows.max(1);
        self.scroll_into_view();
    }

    /// The matches that are currently on screen.
    pub fn visible(&self) -> impl Iterator<Item = Row<'_>> {
        self.matches
            .iter()
            .enumerate()
            .skip(self.top)
            .take(self.rows)
            .map(|(position, &(item, _))| Row {
                item,
                entry: &self.items[item],
                active: position == self.selected,
            })
    }

    /// Character indices of `item`'s name that the search term matched.
    pub fn highlights(&self, item: usize) -> Vec<usize> {
        self.matcher
            .fuzzy_indices(&self.items[item].name, &self.pattern)
            .map(|(_, indices)| indices)
            .unwrap_or_default()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return Outcome::Pending;
        }

        let has_matches = !self.matches.is_empty();
        match key.code {
            KeyCode::Esc => return Outcome::Quit,
            KeyCode::Up | KeyCode::BackTab if has_matches => self.step_back(),
            KeyCode::Down | KeyCode::Tab if has_matches => self.step_forward(),
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                } else if self.term.is_empty() && self.starts_with_parent_entry() {
                    return Outcome::Choose {
                        index: 0,
                        modifiers: KeyModifiers::NONE,
                    };
                }
            }
            KeyCode::Right => {
                if self.cursor < self.term.len() {
                    self.cursor += 1;
                } else if self.term.is_empty() && self.selected_is_enterable() {
                    return Outcome::Choose {
                        index: self.matches[self.selected].0,
                        modifiers: KeyModifiers::NONE,
                    };
                }
            }
            KeyCode::Enter if has_matches => {
                return Outcome::Choose {
                    index: self.matches[self.selected].0,
                    modifiers: key.modifiers,
                };
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.term.remove(self.cursor);
                self.term_changed();
            }
            KeyCode::Char(c) if !c.is_ascii_control() => {
                self.term.insert(self.cursor, c.to_string());
                self.cursor += 1;
                self.selected = 0;
                self.top = 0;
                self.term_changed();
            }
            _ => {}
        }

        Outcome::Pending
    }

    fn starts_with_parent_entry(&self) -> bool {
        self.items
            .first()
            .is_some_and(|item| item.name.ends_with(".."))
    }

    fn selected_is_enterable(&self) -> bool {
        self.matches.get(self.selected).is_some_and(|&(item, _)| {
            let filetype = &self.items[item].filetype;
            *filetype == Filetype::Directory || (cfg!(windows) && *filetype == Filetype::DriveView)
        })
    }

    fn step_back(&mut self) {
        self.selected = (self.selected + self.matches.len() - 1) % self.matches.len();
        self.scroll_into_view();
    }

    fn step_forward(&mut self) {
        self.selected = (self.selected + 1) % self.matches.len();
        self.scroll_into_view();
    }

    /// Re-segments the term (a typed combining mark merges into the previous grapheme) and
    /// refreshes the matches if the pattern really changed.
    fn term_changed(&mut self) {
        let pattern = self.term.concat();
        self.term = pattern.graphemes(true).map(str::to_owned).collect();
        self.cursor = self.cursor.min(self.term.len());

        if pattern != self.pattern {
            self.pattern = pattern;
            self.recompute();
        }
    }

    fn recompute(&mut self) {
        #[cfg(test)]
        {
            self.recomputes += 1;
        }

        self.matches = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                self.matcher
                    .fuzzy_match(&item.name, &self.pattern)
                    .map(|score| (index, score))
            })
            .collect();
        // Stable, so equal scores keep the directory order and ".." stays first
        self.matches.sort_by(|(_, a), (_, b)| b.cmp(a));
        self.scroll_into_view();
    }

    fn scroll_into_view(&mut self) {
        self.selected = self.selected.min(self.matches.len().saturating_sub(1));
        self.top = self.top.min(self.matches.len().saturating_sub(self.rows));

        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + self.rows {
            self.top = self.selected + 1 - self.rows;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::Icons;

    fn entry(name: &str, filetype: Filetype) -> Entry {
        Entry {
            name: name.to_owned(),
            path: format!("/dir/{name}"),
            icon: &Icons::TXT,
            filetype,
        }
    }

    fn files(names: &[&str]) -> Vec<Entry> {
        names
            .iter()
            .map(|name| entry(name, Filetype::Unknown))
            .collect()
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press(selector: &mut Selector, code: KeyCode) -> Outcome {
        selector.handle_key(key(code))
    }

    fn type_str(selector: &mut Selector, text: &str) {
        for c in text.chars() {
            press(selector, KeyCode::Char(c));
        }
    }

    fn names(selector: &Selector) -> Vec<String> {
        selector
            .visible()
            .map(|row| row.entry.name.clone())
            .collect()
    }

    fn active(selector: &Selector) -> Vec<String> {
        selector
            .visible()
            .filter(|row| row.active)
            .map(|row| row.entry.name.clone())
            .collect()
    }

    fn numbered(count: usize) -> Vec<Entry> {
        (0..count)
            .map(|i| entry(&format!("f{i:02}"), Filetype::Unknown))
            .collect()
    }

    #[test]
    fn starts_with_everything_and_the_first_item_selected() {
        let items = files(&["..", "a", "b"]);
        let selector = Selector::new(&items, 10);
        assert_eq!(names(&selector), ["..", "a", "b"]);
        assert_eq!(active(&selector), [".."]);
    }

    #[test]
    fn down_and_up_wrap_around() {
        let items = files(&["a", "b", "c"]);
        let mut selector = Selector::new(&items, 10);

        press(&mut selector, KeyCode::Up);
        assert_eq!(active(&selector), ["c"]);
        press(&mut selector, KeyCode::Down);
        assert_eq!(active(&selector), ["a"]);
        press(&mut selector, KeyCode::Tab);
        assert_eq!(active(&selector), ["b"]);
        press(&mut selector, KeyCode::BackTab);
        assert_eq!(active(&selector), ["a"]);
    }

    #[test]
    fn list_scrolls_one_row_at_a_time() {
        let items = numbered(10);
        let mut selector = Selector::new(&items, 3);
        assert_eq!(names(&selector), ["f00", "f01", "f02"]);

        press(&mut selector, KeyCode::Down);
        press(&mut selector, KeyCode::Down);
        assert_eq!(names(&selector), ["f00", "f01", "f02"]);
        press(&mut selector, KeyCode::Down);
        assert_eq!(names(&selector), ["f01", "f02", "f03"]);
        assert_eq!(active(&selector), ["f03"]);

        press(&mut selector, KeyCode::Up);
        press(&mut selector, KeyCode::Up);
        press(&mut selector, KeyCode::Up);
        assert_eq!(names(&selector), ["f00", "f01", "f02"]);
    }

    #[test]
    fn wrapping_jumps_the_viewport_to_the_other_end() {
        let items = numbered(10);
        let mut selector = Selector::new(&items, 3);

        press(&mut selector, KeyCode::Up);
        assert_eq!(names(&selector), ["f07", "f08", "f09"]);
        assert_eq!(active(&selector), ["f09"]);

        press(&mut selector, KeyCode::Down);
        assert_eq!(names(&selector), ["f00", "f01", "f02"]);
        assert_eq!(active(&selector), ["f00"]);
    }

    #[test]
    fn typing_filters_and_resets_the_selection() {
        let items = files(&["alpha", "beta", "gamma", "delta"]);
        let mut selector = Selector::new(&items, 10);
        press(&mut selector, KeyCode::Down);
        press(&mut selector, KeyCode::Down);

        type_str(&mut selector, "ta");
        assert_eq!(selector.term(), ["t", "a"]);
        assert_eq!(selector.cursor(), 2);
        assert_eq!(names(&selector).len(), 2);
        assert!(names(&selector).contains(&"beta".to_owned()));
        assert!(names(&selector).contains(&"delta".to_owned()));
        assert_eq!(selector.visible().next().map(|row| row.active), Some(true));
    }

    #[test]
    fn best_match_comes_first() {
        let items = files(&["xxaxxbxx", "ab", "a_b"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "ab");
        assert_eq!(names(&selector)[0], "ab");
    }

    #[test]
    fn backspace_widens_the_filter_and_keeps_the_selected_row() {
        let items = files(&["apple", "apricot", "banana"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "apr");
        assert_eq!(names(&selector), ["apricot"]);

        press(&mut selector, KeyCode::Backspace);
        assert_eq!(selector.term(), ["a", "p"]);
        assert_eq!(names(&selector).len(), 2);
        assert!(names(&selector).iter().all(|name| name.starts_with("ap")));
    }

    #[test]
    fn backspace_with_empty_term_does_nothing() {
        let items = files(&["a"]);
        let mut selector = Selector::new(&items, 10);
        assert_eq!(press(&mut selector, KeyCode::Backspace), Outcome::Pending);
        assert_eq!(selector.term(), [] as [String; 0]);
    }

    #[test]
    fn cursor_moves_and_edits_in_the_middle() {
        let items = files(&["abc", "adc", "abd"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "ac");
        press(&mut selector, KeyCode::Left);
        assert_eq!(selector.cursor(), 1);

        press(&mut selector, KeyCode::Char('b'));
        assert_eq!(selector.term(), ["a", "b", "c"]);
        assert_eq!(selector.cursor(), 2);

        press(&mut selector, KeyCode::Backspace);
        assert_eq!(selector.term(), ["a", "c"]);
        assert_eq!(selector.cursor(), 1);

        press(&mut selector, KeyCode::Right);
        press(&mut selector, KeyCode::Right);
        assert_eq!(selector.cursor(), 2);
    }

    #[test]
    fn combining_marks_merge_into_the_previous_grapheme() {
        let items = files(&["e\u{301}clair", "eclair"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "e\u{301}");
        assert_eq!(selector.term(), ["e\u{301}"]);
        assert_eq!(selector.cursor(), 1);
    }

    #[test]
    fn control_characters_are_not_typed() {
        let items = files(&["a"]);
        let mut selector = Selector::new(&items, 10);
        press(&mut selector, KeyCode::Char('\u{7}'));
        assert_eq!(selector.term(), [] as [String; 0]);
    }

    #[test]
    fn enter_chooses_with_modifiers() {
        let items = files(&["a", "b"]);
        let mut selector = Selector::new(&items, 10);
        press(&mut selector, KeyCode::Down);

        let outcome = selector.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT));
        assert_eq!(
            outcome,
            Outcome::Choose {
                index: 1,
                modifiers: KeyModifiers::SHIFT
            }
        );
    }

    #[test]
    fn enter_maps_back_to_the_item_index_after_filtering() {
        let items = files(&["zzz", "target", "other"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "targ");
        assert_eq!(
            press(&mut selector, KeyCode::Enter),
            Outcome::Choose {
                index: 1,
                modifiers: KeyModifiers::NONE
            }
        );
    }

    #[test]
    fn enter_without_matches_does_nothing() {
        let items = files(&["a"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "zzz");
        assert_eq!(names(&selector), [] as [String; 0]);
        assert_eq!(press(&mut selector, KeyCode::Enter), Outcome::Pending);
        assert_eq!(press(&mut selector, KeyCode::Down), Outcome::Pending);
        assert_eq!(press(&mut selector, KeyCode::Up), Outcome::Pending);
        assert_eq!(press(&mut selector, KeyCode::Right), Outcome::Pending);
    }

    #[test]
    fn escape_quits() {
        let items = files(&["a"]);
        let mut selector = Selector::new(&items, 10);
        assert_eq!(press(&mut selector, KeyCode::Esc), Outcome::Quit);
    }

    #[test]
    fn left_on_an_empty_term_goes_to_the_parent() {
        let items = vec![
            entry("..", Filetype::Directory),
            entry("a", Filetype::Directory),
        ];
        let mut selector = Selector::new(&items, 10);
        press(&mut selector, KeyCode::Down);
        assert_eq!(
            press(&mut selector, KeyCode::Left),
            Outcome::Choose {
                index: 0,
                modifiers: KeyModifiers::NONE
            }
        );
    }

    #[test]
    fn left_does_not_go_up_while_searching_or_without_a_parent() {
        let items = vec![
            entry("..", Filetype::Directory),
            entry("a", Filetype::Directory),
        ];
        let mut selector = Selector::new(&items, 10);
        press(&mut selector, KeyCode::Char('a'));
        assert_eq!(press(&mut selector, KeyCode::Left), Outcome::Pending);
        assert_eq!(selector.cursor(), 0);
        assert_eq!(press(&mut selector, KeyCode::Left), Outcome::Pending);

        let no_parent = files(&["a"]);
        let mut selector = Selector::new(&no_parent, 10);
        assert_eq!(press(&mut selector, KeyCode::Left), Outcome::Pending);
    }

    #[test]
    fn right_enters_the_selected_directory_only() {
        let items = vec![
            entry("file", Filetype::Unknown),
            entry("dir", Filetype::Directory),
        ];
        let mut selector = Selector::new(&items, 10);
        assert_eq!(press(&mut selector, KeyCode::Right), Outcome::Pending);

        press(&mut selector, KeyCode::Down);
        assert_eq!(
            press(&mut selector, KeyCode::Right),
            Outcome::Choose {
                index: 1,
                modifiers: KeyModifiers::NONE
            }
        );
    }

    #[test]
    fn right_moves_the_cursor_while_searching() {
        let items = vec![entry("dir", Filetype::Directory)];
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "di");
        press(&mut selector, KeyCode::Left);
        assert_eq!(press(&mut selector, KeyCode::Right), Outcome::Pending);
        assert_eq!(selector.cursor(), 2);
        // at the end of a non empty term Right must not navigate
        assert_eq!(press(&mut selector, KeyCode::Right), Outcome::Pending);
    }

    #[cfg(windows)]
    #[test]
    fn right_enters_the_drive_view_on_windows() {
        let items = vec![entry("PC", Filetype::DriveView)];
        let mut selector = Selector::new(&items, 10);
        assert!(matches!(
            press(&mut selector, KeyCode::Right),
            Outcome::Choose { index: 0, .. }
        ));
    }

    #[test]
    fn key_releases_are_ignored() {
        let items = files(&["a"]);
        let mut selector = Selector::new(&items, 10);
        let mut release = key(KeyCode::Char('x'));
        release.kind = KeyEventKind::Release;
        assert_eq!(selector.handle_key(release), Outcome::Pending);
        assert_eq!(selector.term(), [] as [String; 0]);

        let mut repeat = key(KeyCode::Char('x'));
        repeat.kind = KeyEventKind::Repeat;
        selector.handle_key(repeat);
        assert_eq!(selector.term(), ["x"]);
    }

    #[test]
    fn equal_scores_keep_directory_order_even_in_big_directories() {
        let mut items = vec![entry("..", Filetype::Directory)];
        items.extend((0..500).map(|i| entry(&format!("file{i}"), Filetype::Unknown)));
        let selector = Selector::new(&items, 600);

        let order: Vec<usize> = selector.visible().map(|row| row.item).collect();
        assert_eq!(order, (0..501).collect::<Vec<_>>());
    }

    #[test]
    fn matches_are_only_recomputed_when_the_pattern_changes() {
        let items = numbered(50);
        let mut selector = Selector::new(&items, 5);
        assert_eq!(selector.recomputes, 1);

        for code in [KeyCode::Down, KeyCode::Up, KeyCode::Left, KeyCode::Right] {
            press(&mut selector, code);
        }
        assert_eq!(selector.recomputes, 1);

        press(&mut selector, KeyCode::Char('f'));
        assert_eq!(selector.recomputes, 2);
        press(&mut selector, KeyCode::Left);
        press(&mut selector, KeyCode::Right);
        press(&mut selector, KeyCode::Down);
        assert_eq!(selector.recomputes, 2);

        press(&mut selector, KeyCode::Backspace);
        assert_eq!(selector.recomputes, 3);
    }

    #[test]
    fn shrinking_the_viewport_keeps_the_selection_visible() {
        let items = numbered(20);
        let mut selector = Selector::new(&items, 10);
        for _ in 0..9 {
            press(&mut selector, KeyCode::Down);
        }
        assert_eq!(active(&selector), ["f09"]);

        selector.set_rows(3);
        assert_eq!(names(&selector), ["f07", "f08", "f09"]);
        assert_eq!(active(&selector), ["f09"]);

        selector.set_rows(30);
        assert_eq!(names(&selector).len(), 20);
        assert_eq!(
            selector.visible().next().map(|row| row.entry.name.clone()),
            Some("f00".into())
        );
    }

    #[test]
    fn zero_rows_is_treated_as_one() {
        let items = numbered(3);
        let mut selector = Selector::new(&items, 0);
        assert_eq!(names(&selector).len(), 1);
        selector.set_rows(0);
        assert_eq!(names(&selector).len(), 1);
    }

    #[test]
    fn highlights_report_matched_characters() {
        let items = files(&["abcdef"]);
        let mut selector = Selector::new(&items, 10);
        type_str(&mut selector, "bd");
        assert_eq!(selector.highlights(0), [1, 3]);
    }

    #[test]
    fn highlights_are_empty_without_a_term() {
        let items = files(&["abc"]);
        let selector = Selector::new(&items, 10);
        assert_eq!(selector.highlights(0), [] as [usize; 0]);
    }
}
