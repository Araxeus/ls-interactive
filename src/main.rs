mod structs;
mod utils;

use std::env;
use std::fs;
use std::path::Path;

use structs::{Entry, Filetype, Icons};
use utils::{display_choices, err, resolve_lnk};

fn main() {
    human_panic::setup_panic!();

    let args: Vec<String> = env::args().collect();

    let path = if args.len() >= 2 { args[1].trim() } else { "." };

    match fs::canonicalize(path) {
        Ok(path) => main_loop(path.to_string_lossy().to_string()),
        Err(_) => err("Invalid path"),
    }
}

fn main_loop(initial_path: String) {
    let mut path = initial_path;
    loop {
        let choices = get_choices(&path);
        // make user select a choice and get the selected Entry
        let entry = &choices[display_choices(&choices, &path)];

        // exec file
        if entry.filetype.should_exec() {
            match open::that(&entry.path) {
                // quit if file was opened
                Ok(_) => break,
                // else display error and open as directory
                Err(_) => err(format!("Failed to open file \"{}\"", &entry.path[4..])),
            }
        }
        // browse directory by continuing loop with new path
        path = if entry.filetype == Filetype::Lnk {
            resolve_lnk(&entry.path)
        } else {
            entry.path.to_string()
        };
    }
}

fn get_choices(path: &str) -> Vec<Entry> {
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

    // Get files in directory
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
