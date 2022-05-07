mod structs;
mod utils;

use std::env;
use std::fs;
use std::path::Path;

use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

use structs::{Entry, Filetype, Icons};
use utils::{err, resolve_lnk};

fn main() {
    human_panic::setup_panic!();
    let args: Vec<String> = env::args().collect();
    let path = if args.len() >= 2 { args[1].trim() } else { "." };

    let path = fs::canonicalize(path);

    match path {
        Ok(path) => main_loop(path.to_string_lossy().to_string()),
        Err(_) => err("Invalid path"),
    }
}

fn main_loop(initial_path: String) {
    let mut path = initial_path;
    loop {
        let dir_menu = get_dir_menu(&path);
        let entry = &dir_menu[display_menu(&dir_menu, &path)];
        if entry.filetype.is_openable() {
            match open::that(&entry.path) {
                Ok(_) => break,
                Err(_) => err(format!("Failed to open file \"{}\"", &entry.path[4..])),
            }
        }
        path = if entry.filetype == Filetype::Lnk {
            resolve_lnk(&entry.path)
        } else {
            entry.path.to_string()
        };
    }
}

fn get_dir_menu(path: &str) -> Vec<Entry> {
    let mut result_vector: Vec<Entry> = Vec::new();

    // .. Open parent directory
    if let Ok(parent) = Path::new(path).parent().ok_or("No parent") {
        result_vector.push(Entry {
            name: String::from(".."),
            path: parent.to_str().unwrap().to_string(),
            icon: &Icons::DIR,
            filetype: Filetype::Directory,
        });
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            result_vector.push(Entry::from_dir_entry(entry));
        }
    }

    // Open current directory in explorer
    result_vector.push(Entry {
        name: String::new(),
        path: path.to_string(),
        icon: &Icons::EXPLORER,
        filetype: Filetype::Executable,
    });

    result_vector
}

fn display_menu(items: &[Entry], path: &str) -> usize {
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
        None => std::process::exit(0),
    }
}
