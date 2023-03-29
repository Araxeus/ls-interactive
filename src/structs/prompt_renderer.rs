//! Customizes the rendering of the elements.
use std::{env, fmt, io};

use console::{style, Style, StyledObject, Term};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::utils::{link, link_with_label, pretty_path};

use super::Entry;

/// A colorful theme
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
    /// Formats an error
    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        write!(
            f,
            "{} {}",
            &self.error_prefix,
            self.error_style.apply_to(err)
        )
    }

    /// Formats an input prompt after selection.
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        write!(
            f,
            "{} {}",
            &self.success_suffix,
            self.values_style.apply_to(sel)
        )
    }

    /// Formats a fuzzy select prompt item.
    fn format_fuzzy_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        entry: &Entry,
        active: bool,
        highlight_matches: bool,
        matcher: &SkimMatcherV2,
        search_term: &str,
    ) -> fmt::Result {
        //check if char is right to left aligned (arabic/hebrew unicode range)
        fn is_rtl(c: char) -> bool {
            ('\u{0591}'..='\u{07FF}').contains(&c) // c >= '\u{0591}' && c <= '\u{07FF}'
        }

        let mut output = String::new();

        write!(
            f,
            "{}",
            if active {
                format!("{} {} ", &self.active_item_prefix, entry.icon,)
            } else {
                format!("{} {} ", &self.inactive_item_prefix, entry.icon)
            },
        )?;

        if highlight_matches {
            if let Some((_score, indices)) = matcher.fuzzy_indices(&entry.name, search_term) {
                for (idx, c) in entry.name.chars().enumerate() {
                    let char;

                    if indices.contains(&idx) && !is_rtl(c) {
                        if active {
                            char = format!(
                                "{}",
                                self.active_item_style
                                    .apply_to(self.fuzzy_match_highlight_style.apply_to(c))
                            );
                        } else {
                            char = format!("{}", self.fuzzy_match_highlight_style.apply_to(c));
                        }
                    } else if active {
                        char = format!("{}", self.active_item_style.apply_to(c));
                    } else {
                        char = format!("{c}");
                    };
                    output.push_str(&char);
                }
                write!(f, "{}", link_with_label(pretty_path(&entry.path), &output))?;
                return Ok(());
            }
        }

        write!(f, "{}", entry.name)
    }

    /// Formats a fuzzy-selectprompt after selection.
    fn format_fuzzy_select_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        search_term: &[String], // This should be Vec<str>
        cursor_pos: usize,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            let link_text = if cfg!(windows)
                && prompt == env::var("COMPUTERNAME").unwrap_or("My Computer".to_string())
            {
                link_with_label("shell:MyComputerFolder", prompt)
            } else {
                link(prompt)
            };
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(link_text)
            )?;
        }

        if cursor_pos < search_term.len() {
            let split = search_term.split_at(cursor_pos);
            let head = split.0.concat();
            let cursor = self.fuzzy_cursor_style.apply_to(split.1.get(0).unwrap());
            let tail = split.1[1..].concat();

            write!(f, "{} {head}{cursor}{tail}", &self.prompt_suffix)
        } else {
            let cursor = self.fuzzy_cursor_style.apply_to(" ");
            write!(
                f,
                "{} {}{}",
                &self.prompt_suffix,
                search_term.concat(),
                cursor
            )
        }
    }
}

/// Helper struct to conveniently render a theme ot a term.
pub struct TermRenderer<'a> {
    term: &'a Term,
    theme: &'a Theme,
    height: usize,
    prompt_height: usize,
    prompts_reset_height: bool,
}

impl<'a> TermRenderer<'a> {
    pub const fn new(term: &'a Term, theme: &'a Theme) -> TermRenderer<'a> {
        TermRenderer {
            term,
            theme,
            height: 0,
            prompt_height: 0,
            prompts_reset_height: true,
        }
    }

    pub fn error(&mut self, err: &str) -> io::Result<()> {
        self.write_formatted_line(|this, buf| this.theme.format_error(buf, err))
    }

    fn write_formatted_line<F: FnOnce(&mut TermRenderer, &mut dyn fmt::Write) -> fmt::Result>(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        let mut buf = String::new();
        f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        self.height += buf.chars().filter(|&x| x == '\n').count() + 1;
        self.term.write_line(&buf)
    }

    fn write_formatted_prompt<F: FnOnce(&mut TermRenderer, &mut dyn fmt::Write) -> fmt::Result>(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        self.write_formatted_line(f)?;
        if self.prompts_reset_height {
            self.prompt_height = self.height;
            self.height = 0;
        }
        Ok(())
    }

    pub fn input_prompt_selection(&mut self, prompt: &str, sel: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme.format_input_prompt_selection(buf, prompt, sel)
        })
    }

    pub fn fuzzy_select_prompt(
        &mut self,
        prompt: &str,
        search_term: &[String],
        cursor_pos: usize,
    ) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| {
            this.theme
                .format_fuzzy_select_prompt(buf, prompt, search_term, cursor_pos)
        })
    }

    pub fn fuzzy_select_prompt_item(
        &mut self,
        entry: &Entry,
        active: bool,
        highlight: bool,
        matcher: &SkimMatcherV2,
        search_term: &str,
    ) -> io::Result<()> {
        self.write_formatted_line(|this, buf| {
            this.theme.format_fuzzy_select_prompt_item(
                buf,
                entry,
                active,
                highlight,
                matcher,
                search_term,
            )
        })
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.term
            .clear_last_lines(self.height + self.prompt_height)?;
        self.height = 0;
        Ok(())
    }

    pub fn clear_preserve_prompt(&mut self, size_vec: &[usize]) -> io::Result<()> {
        let mut new_height = self.height;
        //Check each item size, increment on finding an overflow
        for size in size_vec {
            if *size > self.term.size().1 as usize {
                new_height += 1;
            }
        }

        self.term.clear_last_lines(new_height)?;
        self.height = 0;
        Ok(())
    }
}
