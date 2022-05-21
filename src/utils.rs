use crate::structs::Entry;

use console::{style, Term};

pub use dialoguer::KeyModifiers;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use lnk::ShellLink;

use std::{fmt::Display, fs, mem, panic, process};

pub fn err<S: Display>(msg: S) {
    eprintln!("{} {}", style("Error").red(), msg);
}

pub fn resolve_lnk(path: &String) -> String {
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path).unwrap());

    mem::drop(panic::take_hook());

    if link.is_err() {
        err(format!("Failed to read shortcut \"{}\"", pretty_path(path)));
        return path.to_string();
    }

    let path_to_open = match link.unwrap().relative_path() {
        Some(link_target) => fs::canonicalize(link_target).ok(),
        None => None,
    };

    match path_to_open {
        Some(path_to_open) => path_to_open.to_string_lossy().to_string(),
        None => path.to_string(),
    }
}

// dialoguer select from a list of choices
// returns the index of the selected choice
pub fn display_choices(items: &[Entry], path: &str) -> (usize, KeyModifiers) {
    match FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(pretty_path(path))
        .report(false)
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap()
    {
        Some(res) => res,
        // exit process if none
        None => process::exit(0),
    }
}

pub fn pretty_path(path: &str) -> &str {
    if cfg!(windows) {
        &path[4..]
    } else {
        path
    }
}
