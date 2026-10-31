//! Colors, symbols and the layout of the lines of the prompt.
use console::{style, Style, StyledObject};

use crate::utils::{get_computer_name, link_with_label, pretty_path};

use super::{ansi, Entry};

/// A colorful theme
#[allow(dead_code)]
pub struct Theme {
    /// The style for default values
    pub defaults_style: Style,
    /// The style for prompt
    pub prompt_style: Style,
    /// Prompt prefix value and style
    pub prompt_prefix: StyledObject<String>,
    /// Prompt suffix value and style
    pub prompt_suffix: StyledObject<String>,
    /// Prompt on success prefix value and style
    pub success_prefix: StyledObject<String>,
    /// Prompt on success suffix value and style
    pub success_suffix: StyledObject<String>,
    /// Error prefix value and style
    pub error_prefix: StyledObject<String>,
    /// The style for error message
    pub error_style: Style,
    /// The style for hints
    pub hint_style: Style,
    /// The style for values on prompt success
    pub values_style: Style,
    /// The style for active items
    pub active_item_style: Style,
    /// The style for inactive items
    pub inactive_item_style: Style,
    /// Active item in select prefix value and style
    pub active_item_prefix: StyledObject<String>,
    /// Inctive item in select prefix value and style
    pub inactive_item_prefix: StyledObject<String>,
    /// Checked item in multi select prefix value and style
    pub checked_item_prefix: StyledObject<String>,
    /// Unchecked item in multi select prefix value and style
    pub unchecked_item_prefix: StyledObject<String>,
    /// Picked item in sort prefix value and style
    pub picked_item_prefix: StyledObject<String>,
    /// Unpicked item in sort prefix value and style
    pub unpicked_item_prefix: StyledObject<String>,
    /// Formats the cursor for a fuzzy select prompt
    pub fuzzy_cursor_style: Style,
    // Formats the highlighting of matched characters
    pub fuzzy_match_highlight_style: Style,
    /// Show the selections from certain prompts inline
    pub inline_selections: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            defaults_style: Style::new().for_stderr().cyan(),
            prompt_style: Style::new().for_stderr().bold(),
            prompt_prefix: style("?".to_string()).for_stderr().yellow(),
            prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
            success_prefix: style("✔".to_string()).for_stderr().green(),
            success_suffix: style("·".to_string()).for_stderr().black().bright(),
            error_prefix: style("✘".to_string()).for_stderr().red(),
            error_style: Style::new().for_stderr().red(),
            hint_style: Style::new().for_stderr().black().bright(),
            values_style: Style::new().for_stderr().green(),
            active_item_style: Style::new().for_stderr().cyan(),
            inactive_item_style: Style::new().for_stderr(),
            active_item_prefix: style("❯".to_string()).for_stderr().green(),
            inactive_item_prefix: style(" ".to_string()).for_stderr(),
            checked_item_prefix: style("✔".to_string()).for_stderr().green(),
            unchecked_item_prefix: style("✔".to_string()).for_stderr().black(),
            picked_item_prefix: style("❯".to_string()).for_stderr().green(),
            unpicked_item_prefix: style(" ".to_string()).for_stderr(),
            fuzzy_cursor_style: Style::new().for_stderr().black().on_white(),
            fuzzy_match_highlight_style: Style::new().for_stderr().bold(),
            inline_selections: true,
        }
    }
}

impl Theme {
    /// The prompt line: the title (a link to the current folder), then what the user typed with
    /// the cursor on it. A title that is too long loses its start, so the search term always fits.
    pub fn prompt_line(
        &self,
        title: &str,
        term: &[String],
        cursor: usize,
        max_cols: usize,
    ) -> String {
        let term_text = self.term_text(term, cursor);

        if title.is_empty() {
            return format!("{} {term_text}", self.prompt_suffix);
        }

        let prefix = format!("{} ", self.prompt_prefix);
        let suffix = format!(" {} ", self.prompt_suffix);
        let fixed = ansi::display_width(&prefix)
            + ansi::display_width(&suffix)
            + ansi::display_width(&term_text);
        let budget = max_cols.saturating_sub(fixed).max(8);

        let path = ansi::printable(title);
        let label = ansi::tail_fit(&path, budget);
        let target = if cfg!(windows) && title == get_computer_name() {
            "shell:MyComputerFolder"
        } else {
            &path
        };
        let title = self.prompt_style.apply_to(link_with_label(target, &label));

        format!("{prefix}{title}{suffix}{term_text}")
    }

    fn term_text(&self, term: &[String], cursor: usize) -> String {
        let term: Vec<String> = term.iter().map(|g| ansi::printable(g)).collect();

        term.get(cursor).map_or_else(
            || format!("{}{}", term.concat(), self.fuzzy_cursor_style.apply_to(" ")),
            |under_cursor| {
                format!(
                    "{}{}{}",
                    term[..cursor].concat(),
                    self.fuzzy_cursor_style.apply_to(under_cursor),
                    term[cursor + 1..].concat()
                )
            },
        )
    }

    /// One row of the list. `highlights` are the character indices of the name to emphasize.
    pub fn item_line(&self, entry: &Entry, active: bool, highlights: &[usize]) -> String {
        let prefix = if active {
            &self.active_item_prefix
        } else {
            &self.inactive_item_prefix
        };

        let label = self.item_label(&ansi::printable(&entry.name), active, highlights);
        let link = link_with_label(&ansi::printable(pretty_path(&entry.path)), &label);

        format!("{prefix} {} {link}", entry.icon)
    }

    /// Check if a char is right to left aligned (arabic/hebrew unicode range). Highlighting those
    /// would split joined letters.
    fn is_rtl(c: char) -> bool {
        ('\u{0591}'..='\u{07FF}').contains(&c)
    }

    fn item_label(&self, name: &str, active: bool, highlights: &[usize]) -> String {
        let mut label = String::new();
        let mut run = String::new();
        let mut run_highlighted = false;

        // Styling runs of characters at once keeps the output small
        for (idx, c) in name.chars().enumerate() {
            let highlighted = highlights.contains(&idx) && !Self::is_rtl(c);
            if highlighted != run_highlighted {
                label.push_str(&self.style_run(&run, run_highlighted, active));
                run.clear();
                run_highlighted = highlighted;
            }
            run.push(c);
        }
        label.push_str(&self.style_run(&run, run_highlighted, active));
        label
    }

    fn style_run(&self, text: &str, highlighted: bool, active: bool) -> String {
        if text.is_empty() {
            return String::new();
        }

        match (highlighted, active) {
            (true, true) => format!(
                "{}",
                self.active_item_style
                    .apply_to(self.fuzzy_match_highlight_style.apply_to(text))
            ),
            (true, false) => format!("{}", self.fuzzy_match_highlight_style.apply_to(text)),
            (false, true) => format!("{}", self.active_item_style.apply_to(text)),
            (false, false) => text.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::{Filetype, Icons};

    fn entry(name: &str) -> Entry {
        Entry {
            name: name.to_owned(),
            path: format!("/dir/{name}"),
            icon: &Icons::TXT,
            filetype: Filetype::Unknown,
        }
    }

    fn plain(s: &str) -> String {
        ansi::strip(s)
    }

    fn term(text: &str) -> Vec<String> {
        text.chars().map(|c| c.to_string()).collect()
    }

    #[test]
    fn item_line_shows_prefix_icon_and_name() {
        let theme = Theme::default();
        let line = theme.item_line(&entry("notes.txt"), false, &[]);
        assert_eq!(plain(&line), format!("  {} notes.txt", Icons::TXT));

        let line = theme.item_line(&entry("notes.txt"), true, &[]);
        assert_eq!(plain(&line), format!("❯ {} notes.txt", Icons::TXT));
    }

    #[test]
    fn item_line_links_to_the_entry_path() {
        let line = Theme::default().item_line(&entry("a b"), false, &[]);
        assert!(line.contains("\u{1b}]8;;/dir/a b\u{1b}\\"));
        assert!(line.ends_with("\u{1b}]8;;\u{1b}\\"));
    }

    #[test]
    fn item_line_never_contains_the_escapes_of_a_hostile_file_name() {
        let theme = Theme::default();
        let hostile = entry("evil\u{1b}[2J\u{1b}]8;;http://x\u{7}\nname");
        let line = theme.item_line(&hostile, false, &[]);

        assert!(!line.contains("[2J\u{1b}") && !line.contains("\u{1b}[2J"));
        assert!(!line.contains('\n') && !line.contains('\u{7}'));
        assert_eq!(
            ansi::display_width(&line),
            ansi::display_width(&plain(&line))
        );
    }

    #[test]
    fn highlights_are_applied_to_the_matched_characters_only() {
        console::set_colors_enabled_stderr(true);
        let theme = Theme::default();

        let line = theme.item_line(&entry("abcdef"), false, &[1, 2, 4]);
        let bold = theme.fuzzy_match_highlight_style.apply_to("bc").to_string();
        assert!(line.contains(&bold), "{line:?}");
        let bold = theme.fuzzy_match_highlight_style.apply_to("e").to_string();
        assert!(line.contains(&bold), "{line:?}");
        assert_eq!(plain(&line), format!("  {} abcdef", Icons::TXT));
    }

    #[test]
    fn active_rows_are_colored_as_a_whole() {
        console::set_colors_enabled_stderr(true);
        let theme = Theme::default();

        let line = theme.item_line(&entry("abc"), true, &[]);
        let cyan = theme.active_item_style.apply_to("abc").to_string();
        assert!(line.contains(&cyan), "{line:?}");
    }

    #[test]
    fn right_to_left_letters_are_not_highlighted() {
        console::set_colors_enabled_stderr(true);
        let theme = Theme::default();

        let line = theme.item_line(&entry("שלום"), false, &[0, 1, 2, 3]);
        assert!(!line.contains("\u{1b}[1m"), "{line:?}");
        assert!(plain(&line).ends_with("שלום"));
    }

    #[test]
    fn prompt_line_shows_the_title_and_the_term_with_a_cursor() {
        let theme = Theme::default();
        let line = theme.prompt_line("/home/me", &term("ab"), 2, 80);
        assert_eq!(plain(&line), "? /home/me › ab ");

        let line = theme.prompt_line("/home/me", &term("ab"), 0, 80);
        assert_eq!(plain(&line), "? /home/me › ab");

        let line = theme.prompt_line("/home/me", &[], 0, 80);
        assert_eq!(plain(&line), "? /home/me ›  ");
    }

    #[test]
    fn the_title_links_to_the_full_path() {
        let line = Theme::default().prompt_line("/home/me", &[], 0, 80);
        assert!(line.contains("\u{1b}]8;;/home/me\u{1b}\\"));
    }

    #[test]
    fn a_long_title_loses_its_start_and_keeps_the_term_visible() {
        let theme = Theme::default();
        let title = format!("/{}/leaf", "very-long-directory-name/".repeat(10));
        let line = theme.prompt_line(&title, &term("query"), 5, 40);

        let text = plain(&line);
        assert!(text.ends_with("› query "), "{text:?}");
        assert!(text.contains("…") && text.contains("leaf"));
        assert!(
            ansi::display_width(&line) <= 40,
            "{}",
            ansi::display_width(&line)
        );
        // the link still points at the real folder
        assert!(line.contains(&format!("\u{1b}]8;;{title}\u{1b}\\")));
    }

    #[test]
    fn prompt_line_without_a_title_has_no_prefix() {
        let line = Theme::default().prompt_line("", &term("x"), 1, 80);
        assert_eq!(plain(&line), "› x ");
    }
}
