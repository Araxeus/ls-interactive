mod structs;
mod utils;

use std::{fs, path::Path};

use structs::{Entry, Filetype, Icons};
use utils::{display_choices, err, get_first_arg, pretty_path, resolve_lnk, KeyModifiers};

#[cfg(windows)]
use utils::{get_logical_drives, get_pc_name};

use tiny_update_notifier::run_notifier;

fn main() {
    human_panic::setup_panic!();

    run_notifier(
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
    let mut selected_entry = Entry {
        path: initial_path,
        ..Default::default()
    };

    loop {
        let choices = get_choices(&selected_entry);

        let (index, modifier) = display_choices(&choices, &selected_entry.path);

        let entry = choices[index].clone();

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
        if entry.filetype == Filetype::Lnk {
            selected_entry.path = resolve_lnk(&entry.path);
        }

        selected_entry = entry;

        if modifier == KeyModifiers::SHIFT || modifier == KeyModifiers::ALT {
            print!("{}", pretty_path(&selected_entry.path));
            break;
        }
    }
}

fn get_choices(entry: &Entry) -> Vec<Entry> {
    let mut result_vector: Vec<Entry> = Vec::new();

    #[cfg(windows)]
    // Open Drives View on Windows
    if entry.filetype == Filetype::DriveView {
        match get_logical_drives() {
            Ok(drives) => {
                for drive in drives {
                    result_vector.push(Entry {
                        name: format!("{drive}:\\"),
                        path: format!("{drive}:\\"),
                        icon: &Icons::DRIVE,
                        filetype: Filetype::Directory,
                    });
                }
                return result_vector;
            }
            Err(_) => err("Failed to get drives"),
        }
    }

    // .. Open parent directory
    if let Some(parent) = Path::new(&entry.path).parent() {
        result_vector.push(Entry {
            name: String::from(".."),
            path: parent.to_string_lossy().to_string(),
            icon: &Icons::DIR,
            filetype: Filetype::Directory,
        });
    }

    #[cfg(windows)]
    if result_vector.is_empty() {
        // .. Open Drives View on Windows
        result_vector.push(Entry {
            name: String::from(".."),
            path: get_pc_name(),
            icon: &Icons::PC,
            filetype: Filetype::DriveView,
        });
    }

    // Get files in directory
    if let Ok(entries) = fs::read_dir(&entry.path) {
        for entry in entries.flatten() {
            result_vector.push(Entry::from_dir_entry(&entry));
        }
    }

    // Open current directory in explorer if it's empty
    if result_vector.len() < 2 {
        result_vector.push(Entry {
            name: String::from("<Open>"),
            path: entry.path.clone(),
            icon: &Icons::EXPLORER,
            filetype: Filetype::Executable,
        });
    }

    result_vector
}
