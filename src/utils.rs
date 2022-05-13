use crate::structs::Entry;

use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use lnk::ShellLink;

use std::{fmt::Display, fs, mem, panic, process};

pub fn err<S: Display>(msg: S) {
    println!("{} {}", style("Error").red(), msg);
}

pub fn resolve_lnk(path: &String) -> String {
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path).unwrap());

    mem::drop(panic::take_hook());

    if link.is_err() {
        err(format!("Failed to read shortcut \"{}\"", &path[4..]));
        return path.to_string();
    }

    let link = link.unwrap();

    let link_target = link.relative_path().as_ref().unwrap();
    let path_to_open = fs::canonicalize(link_target);

    match path_to_open {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => path.to_string(),
    }
}

// dialoguer select from a list of choices
// returns the index of the selected choice
pub fn display_choices(items: &[Entry], path: &str) -> usize {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&path[4..])
        .report(false)
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .ok()
        .unwrap()
    {
        Some(index) => index,
        // exit process if none
        None => process::exit(0),
    }
}
