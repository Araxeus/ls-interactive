//! Measuring and fitting strings that contain ANSI escape sequences.
//!
//! The screen diffing in [`super::screen`] relies on every line occupying exactly one terminal row,
//! so lines are measured in display columns and sanitized before they are drawn.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;
const RESET: &str = "\u{1b}[0m";
const LINK_END: &str = "\u{1b}]8;;\u{1b}\\";
const ELLIPSIS: &str = "…";

/// A slice of a string that is either an escape sequence or visible text.
enum Piece<'a> {
    Escape(&'a str),
    Text(&'a str),
}

/// Splits `s` into escape sequences (CSI, OSC and two byte escapes) and the text between them.
fn pieces(s: &str) -> impl Iterator<Item = Piece<'_>> {
    let bytes = s.as_bytes();
    let mut pos = 0;

    std::iter::from_fn(move || {
        if pos >= bytes.len() {
            return None;
        }

        let start = pos;
        if bytes[pos] != ESC {
            pos = bytes[pos..]
                .iter()
                .position(|&b| b == ESC)
                .map_or(bytes.len(), |offset| pos + offset);
            return Some(Piece::Text(&s[start..pos]));
        }

        pos += 1;
        match bytes.get(pos) {
            Some(b'[') => {
                pos += 1;
                // Parameter and intermediate bytes, then one final byte in 0x40..=0x7E
                while pos < bytes.len() {
                    let b = bytes[pos];
                    pos += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
            }
            Some(b']') => {
                pos += 1;
                // Terminated by BEL or by the string terminator ESC \
                while pos < bytes.len() {
                    let b = bytes[pos];
                    pos += 1;
                    if b == BEL {
                        break;
                    }
                    if b == ESC && bytes.get(pos) == Some(&b'\\') {
                        pos += 1;
                        break;
                    }
                }
            }
            Some(_) => {
                pos += s[pos..].chars().next().map_or(1, char::len_utf8);
            }
            None => {}
        }

        Some(Piece::Escape(&s[start..pos]))
    })
}

/// Visible grapheme clusters of a text piece. Control characters are dropped, since they would
/// move the cursor and break the one line = one row guarantee.
fn visible_graphemes(text: &str) -> impl Iterator<Item = &str> {
    text.graphemes(true)
        .filter(|g| !g.chars().any(char::is_control))
}

/// Display width of one grapheme cluster.
///
/// Zero width joiner sequences are only drawn as a single glyph by some terminals, so they are
/// measured as the sum of their parts. Overestimating is safe, underestimating would wrap the line.
fn grapheme_width(grapheme: &str) -> usize {
    if grapheme.contains('\u{200D}') {
        grapheme.split('\u{200D}').map(UnicodeWidthStr::width).sum()
    } else {
        UnicodeWidthStr::width(grapheme)
    }
}

/// Number of terminal columns the visible text of `s` occupies, ignoring escape sequences.
pub fn display_width(s: &str) -> usize {
    pieces(s)
        .map(|piece| match piece {
            Piece::Text(text) => visible_graphemes(text).map(grapheme_width).sum(),
            Piece::Escape(_) => 0,
        })
        .sum()
}

/// The visible text of `s` without any escape sequences.
#[cfg(test)]
pub fn strip(s: &str) -> String {
    pieces(s)
        .filter_map(|piece| match piece {
            Piece::Text(text) => Some(visible_graphemes(text).collect::<String>()),
            Piece::Escape(_) => None,
        })
        .collect()
}

/// Replaces control characters (escape sequences, newlines, ...) with `�`, one for one.
///
/// File names can contain anything. Keeping the character count unchanged keeps match indices
/// valid.
pub fn printable(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect()
}

/// Terminal attribute state that must be closed again if a line is cut short.
#[derive(Default)]
struct OpenState {
    styled: bool,
    link: bool,
}

impl OpenState {
    fn observe(&mut self, escape: &str) {
        if let Some(params) = escape
            .strip_prefix("\u{1b}[")
            .and_then(|rest| rest.strip_suffix('m'))
        {
            self.styled = !matches!(params, "" | "0");
        } else if let Some(payload) = escape.strip_prefix("\u{1b}]") {
            let payload = payload
                .strip_suffix('\u{7}')
                .or_else(|| payload.strip_suffix("\u{1b}\\"))
                .unwrap_or(payload);
            let mut fields = payload.splitn(3, ';');
            if fields.next() == Some("8") {
                self.link = fields.nth(1).is_some_and(|uri| !uri.is_empty());
            }
        }
    }

    fn close(&self, out: &mut String) {
        if self.link {
            out.push_str(LINK_END);
        }
        if self.styled {
            out.push_str(RESET);
        }
    }
}

/// Sanitizes `s` and cuts it down to at most `max_cols` columns.
///
/// Escape sequences are kept, control characters are dropped and a trailing `…` marks a cut.
/// Styles and hyperlinks that are still open at the end are closed, so a line never leaks
/// attributes into the next one.
pub fn fit(s: &str, max_cols: usize) -> String {
    let overflows = display_width(s) > max_cols;
    let budget = if overflows {
        max_cols.saturating_sub(1)
    } else {
        max_cols
    };

    let mut out = String::with_capacity(s.len());
    let mut state = OpenState::default();
    let mut used = 0;

    'outer: for piece in pieces(s) {
        match piece {
            Piece::Escape(escape) => {
                state.observe(escape);
                out.push_str(escape);
            }
            Piece::Text(text) => {
                for grapheme in visible_graphemes(text) {
                    let width = grapheme_width(grapheme);
                    if used + width > budget {
                        break 'outer;
                    }
                    used += width;
                    out.push_str(grapheme);
                }
            }
        }
    }

    state.close(&mut out);
    if overflows && max_cols > 0 {
        out.push_str(ELLIPSIS);
    }
    out
}

/// `label` as an OSC 8 hyperlink to `target`.
pub fn hyperlink(target: &str, label: &str) -> String {
    format!("\u{1b}]8;;{target}\u{1b}\\{label}{LINK_END}")
}

/// Keeps the end of plain text `s` within `max_cols`, replacing the start with `…` if needed.
///
/// Used for paths, where the last components matter most.
pub fn tail_fit(s: &str, max_cols: usize) -> String {
    if UnicodeWidthStr::width(s) <= max_cols {
        return s.to_owned();
    }
    if max_cols == 0 {
        return String::new();
    }

    let mut used = 0;
    let mut kept = Vec::new();
    for grapheme in s.graphemes(true).rev() {
        let width = grapheme_width(grapheme);
        if used + width > max_cols - 1 {
            break;
        }
        used += width;
        kept.push(grapheme);
    }
    kept.push(ELLIPSIS);
    kept.reverse();
    kept.concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: &str = "\u{1b}[31m";

    #[test]
    fn width_ignores_escape_sequences() {
        assert_eq!(display_width("plain"), 5);
        assert_eq!(display_width(&format!("{RED}plain{RESET}")), 5);
        assert_eq!(
            display_width("\u{1b}]8;;/some/path\u{1b}\\label\u{1b}]8;;\u{1b}\\"),
            5
        );
        assert_eq!(display_width("\u{1b}]8;;x\u{7}hi\u{1b}]8;;\u{7}"), 2);
    }

    #[test]
    fn width_counts_wide_and_combining_characters() {
        assert_eq!(display_width("日本"), 4);
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(display_width("📁"), 2);
        // emoji presentation sequence
        assert_eq!(display_width("⚙\u{fe0f}"), 2);
    }

    #[test]
    fn zero_width_joiner_sequences_are_never_underestimated() {
        let family = "👨\u{200d}👩\u{200d}👧";
        assert!(display_width(family) >= 2);
        assert_eq!(display_width(family), 6);
    }

    #[test]
    fn width_drops_control_characters() {
        assert_eq!(display_width("a\nb\r\tc\u{7f}"), 3);
    }

    #[test]
    fn fit_keeps_short_lines_untouched() {
        let line = format!("{RED}abc{RESET}");
        assert_eq!(fit(&line, 3), line);
        assert_eq!(fit(&line, 10), line);
        assert_eq!(fit("", 5), "");
    }

    #[test]
    fn fit_cuts_with_ellipsis_and_closes_styles() {
        let line = format!("{RED}abcdefgh{RESET}");
        let cut = fit(&line, 5);
        assert_eq!(cut, format!("{RED}abcd{RESET}…"));
        assert_eq!(display_width(&cut), 5);
    }

    #[test]
    fn fit_closes_hyperlinks_that_were_cut() {
        let line = "\u{1b}]8;;/path\u{1b}\\abcdefgh\u{1b}]8;;\u{1b}\\";
        let cut = fit(line, 4);
        assert_eq!(cut, "\u{1b}]8;;/path\u{1b}\\abc\u{1b}]8;;\u{1b}\\…");
        assert_eq!(display_width(&cut), 4);
    }

    #[test]
    fn fit_does_not_close_what_is_already_closed() {
        let line = format!("\u{1b}]8;;/p\u{1b}\\{RED}ab{RESET}\u{1b}]8;;\u{1b}\\cdefgh");
        let cut = fit(&line, 4);
        assert_eq!(
            cut,
            format!("\u{1b}]8;;/p\u{1b}\\{RED}ab{RESET}\u{1b}]8;;\u{1b}\\c…")
        );
    }

    #[test]
    fn fit_closes_a_style_that_was_never_reset() {
        assert_eq!(fit(&format!("{RED}abc"), 10), format!("{RED}abc{RESET}"));
    }

    #[test]
    fn fit_never_splits_wide_characters() {
        let cut = fit("日本語日本語", 6);
        assert_eq!(cut, "日本…");
        assert!(display_width(&cut) <= 6);

        // a wide character that does not fit the remaining column is dropped
        assert_eq!(fit("ab日本", 4), "ab…");
    }

    #[test]
    fn fit_never_splits_grapheme_clusters() {
        assert_eq!(fit("ae\u{301}e\u{301}e\u{301}", 3), "ae\u{301}…");
    }

    #[test]
    fn fit_removes_control_characters() {
        assert_eq!(fit("a\nb\rc\td", 10), "abcd");
        assert_eq!(fit("a\u{85}b", 10), "ab");
    }

    #[test]
    fn fit_handles_degenerate_widths() {
        assert_eq!(fit("abc", 0), "");
        assert_eq!(fit("abc", 1), "…");
    }

    #[test]
    fn fit_keeps_unknown_escapes_intact() {
        assert_eq!(fit("\u{1b}7ab\u{1b}8", 10), "\u{1b}7ab\u{1b}8");
        assert_eq!(fit("ab\u{1b}", 10), "ab\u{1b}");
        assert_eq!(fit("ab\u{1b}[31", 10), "ab\u{1b}[31");
    }

    #[test]
    fn fit_always_respects_the_budget() {
        let samples = [
            "plain text",
            "日本語のファイル名.txt",
            "e\u{301}e\u{301}e\u{301}e\u{301}",
            "👨\u{200d}👩\u{200d}👧 family",
            "\u{1b}[1m\u{1b}[36mstyled\u{1b}[0m text that is long",
        ];
        for sample in samples {
            for max in 0..30 {
                let cut = fit(sample, max);
                assert!(
                    display_width(&cut) <= max,
                    "{sample:?} cut to {max} is {cut:?}"
                );
            }
        }
    }

    #[test]
    fn printable_replaces_controls_one_for_one() {
        assert_eq!(printable("plain 日本"), "plain 日本");
        assert_eq!(printable("a\u{1b}[31mb\nc"), "a\u{fffd}[31mb\u{fffd}c");
        assert_eq!(printable("a\u{85}b").chars().count(), 3);
        assert_eq!(display_width(&printable("\u{1b}]8;;x\u{7}")), 7);
    }

    #[test]
    fn hyperlinks_wrap_the_label() {
        let link = hyperlink("/a b", "label");
        assert_eq!(link, "\u{1b}]8;;/a b\u{1b}\\label\u{1b}]8;;\u{1b}\\");
        assert_eq!(display_width(&link), 5);
    }

    #[test]
    fn tail_fit_keeps_the_end() {
        assert_eq!(tail_fit("/home/user/project", 40), "/home/user/project");
        assert_eq!(tail_fit("/home/user/project", 10), "…r/project");
        assert_eq!(tail_fit("/home/user/project", 9), "…/project");
        assert_eq!(tail_fit("/home/user/project", 1), "…");
        assert_eq!(tail_fit("/home/user/project", 0), "");
    }

    #[test]
    fn tail_fit_respects_wide_characters() {
        let cut = tail_fit("/日本語/日本語", 7);
        assert!(UnicodeWidthStr::width(cut.as_str()) <= 7);
        assert!(cut.starts_with('…'));
        assert!(cut.ends_with("日本語"));
    }
}
