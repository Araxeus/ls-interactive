mod structs;
mod utils;

use std::{fs, path::Path};

use structs::{Entry, Filetype, Icons};
use utils::{display_choices, err, get_first_arg, pretty_path, resolve_lnk, KeyModifiers};

use tiny_update_notifier::check_github;

fn main() {
    human_panic::setup_panic!();

    check_github(
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_REPOSITORY"),
    );

    // path = first arg or current dir
    let path = get_first_arg().map_or_else(|| String::from("."), |path| path);

    fs::canonicalize(path).map_or_else(
        |_| err("Invalid Path"),
        |path| main_loop(path.to_string_lossy().to_string()),
    );
}

fn main_loop(initial_path: String) {
    let mut path = initial_path;
    loop {
        let choices = get_choices(&path);

        let (index, modifier) = display_choices(&choices, &path);

        let entry = &choices[index];

        // exec file
        if entry.filetype.should_exec() || modifier == KeyModifiers::CONTROL {
            match open::that(&entry.path) {
                // quit if file was opened
                Ok(_) => break,
                // else display error and open as directory
                Err(_) => err(format!(
                    "Failed to open file \"{}\"",
                    pretty_path(&entry.path)
                )),
            }
        }
        // browse directory by continuing loop with new path
        path = if entry.filetype == Filetype::Lnk {
            resolve_lnk(&entry.path)
        } else {
            entry.path.to_string()
        };

        if modifier == KeyModifiers::SHIFT || modifier == KeyModifiers::ALT {
            print!("{}", pretty_path(&path));
            break;
        }
    }
}

fn get_choices(path: &str) -> Vec<Entry> {
    let mut result_vector: Vec<Entry> = Vec::new();

    // .. Open parent directory
    if let Some(parent) = Path::new(path).parent() {
        result_vector.push(Entry {
            name: String::from(".."),
            path: parent.to_string_lossy().to_string(),
            icon: &Icons::DIR,
            filetype: Filetype::Directory,
        });
    }

    // Get files in directory
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            result_vector.push(Entry::from_dir_entry(&entry));
        }
    }

    // Open current directory in explorer if it's empty
    if result_vector.len() < 2 {
        result_vector.push(Entry {
            name: String::from("<Open>"),
            path: path.to_string(),
            icon: &Icons::EXPLORER,
            filetype: Filetype::Executable,
        });
    }

    result_vector
}
