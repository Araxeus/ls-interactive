use crate::structs::Entry;

use console::{style, Term};

pub use dialoguer::KeyModifiers;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use lnk::ShellLink;

use std::{env, fmt::Display, fs, mem, panic, process};

pub fn err<S: Display>(msg: S) {
    eprintln!("{} {msg}", style("Error:").red().for_stderr());
}

pub fn resolve_lnk(path: &String) -> String {
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path).unwrap());

    mem::drop(panic::take_hook());

    if link.is_err() {
        err(format!("Failed to read shortcut \"{}\"", pretty_path(path)));
        return path.to_string();
    }

    let path_to_open = link
        .unwrap()
        .relative_path()
        .as_ref()
        .and_then(|link_target| fs::canonicalize(link_target).ok());

    path_to_open.map_or_else(
        || path.to_string(),
        |path_to_open| path_to_open.to_string_lossy().to_string(),
    )
}

// dialoguer select from a list of choices
// returns the index of the selected choice
pub fn display_choices(items: &[Entry], path: &str) -> (usize, KeyModifiers) {
    FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(pretty_path(path))
        .report(false)
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .map_or_else(|| process::exit(0), |res| res)
}

pub fn get_first_arg() -> Option<String> {
    env::args().nth(1).and_then(|arg| {
        let arg = arg.trim();
        if arg.is_empty() {
            None
        } else {
            Some(arg.to_owned())
        }
    })
}

pub fn pretty_path(path: &str) -> &str {
    if cfg!(windows) {
        &path[4..]
    } else {
        path
    }
}
