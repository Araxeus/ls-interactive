#![allow(clippy::multiple_crate_versions)]
mod entry;
mod prompt;
mod render;
mod system;

use std::{env, fmt::Display, fs, path::Path, process};

use console::style;
use crossterm::event::KeyModifiers;

use entry::{shortcut, Entry, Filetype, Icons};
use prompt::{restore_terminal, Ui};
use system::pretty_path;

use tiny_update_notifier::check_github;

fn main() {
    human_panic::setup_panic!();

    // A panic must not leave the terminal in raw mode with a hidden cursor
    let report_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        report_panic(info);
    }));

    check_github(
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_REPOSITORY"),
    );

    // Flags may appear anywhere. The first other argument is the path.
    let args: Vec<String> = env::args()
        .skip(1)
        .map(|arg| arg.trim().to_owned())
        .filter(|arg| !arg.is_empty())
        .collect();
    let has_flag = |flags: &[&str]| args.iter().any(|arg| flags.contains(&arg.as_str()));

    if has_flag(&["--help", "-h"]) {
        println!(
            "Usage: {} [path] [flags]

Flags:
    -v, --version   Print version   
    -h, --help      Print help
    -x, --no-exec   Don't execute files
",
            env!("CARGO_PKG_NAME")
        );
        return;
    }

    if has_flag(&["--version", "-v"]) {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let no_exec_flags = ["--no-exec", "--noexec", "-x"];
    let no_exec = has_flag(&no_exec_flags);

    let path = args
        .iter()
        .find(|arg| !no_exec_flags.contains(&arg.as_str()))
        .map_or(".", String::as_str);

    fs::canonicalize(path).map_or_else(
        |_| err("Invalid Path"),
        |path| main_loop(path.to_string_lossy().to_string(), no_exec),
    );
}

fn main_loop(initial_path: String, no_exec: bool) {
    let mut selected_entry = Entry {
        path: initial_path,
        ..Default::default()
    };

    // One Ui for the whole session, so each folder is drawn over the previous one
    let mut ui = Ui::new();

    loop {
        let choices = get_choices(&selected_entry);

        let (index, modifier) = choose(&mut ui, &choices, &selected_entry.path);

        let entry = choices[index].clone();

        if modifier == KeyModifiers::SHIFT || modifier == KeyModifiers::ALT {
            finish(&mut ui);
            if index == 0 {
                print!("{}", pretty_path(&selected_entry.path));
            } else {
                print!("{}", pretty_path(&entry.path));
            }
            break;
        }

        // exec file
        if entry.filetype.should_exec() || modifier == KeyModifiers::CONTROL {
            if no_exec {
                finish(&mut ui);
                print!("{}", pretty_path(&entry.path));
                break;
            }
            let opened = open::that(&entry.path);
            finish(&mut ui);

            // quit if file was opened, else display error and open as directory
            if opened.is_ok() {
                break;
            }
            err(format!(
                "Failed to open file \"{}\"",
                pretty_path(&entry.path)
            ));
        }

        // browse directory by continuing loop with new path
        selected_entry = entry;

        if selected_entry.filetype == Filetype::Lnk {
            // An error must not land inside the prompt
            finish(&mut ui);
            match shortcut::resolve(&selected_entry.path) {
                Some(target) => selected_entry.path = target,
                None => err(format!(
                    "Failed to read shortcut \"{}\"",
                    pretty_path(&selected_entry.path)
                )),
            }
        }
    }
}

/// Lets the user choose one of `items`. Returns its index and the modifiers held. Quitting
/// exits the program.
fn choose(ui: &mut Ui, items: &[Entry], path: &str) -> (usize, KeyModifiers) {
    match ui.choose(items, pretty_path(path)) {
        Ok(Some(choice)) => choice,
        Ok(None) => process::exit(0),
        Err(error) => {
            err(format!("Failed to read input: {error}"));
            process::exit(1)
        }
    }
}

fn err<S: Display>(msg: S) {
    eprintln!("{} {msg}", style("Error:").red().for_stderr());
}

/// Takes the prompt off the screen so that whatever is printed next starts on a clean line.
fn finish(ui: &mut Ui) {
    if let Err(error) = ui.finish() {
        err(format!("Failed to restore the terminal: {error}"));
    }
}

fn get_choices(entry: &Entry) -> Vec<Entry> {
    let mut result_vector: Vec<Entry> = Vec::new();

    #[cfg(windows)]
    // Open Drives View on Windows
    if entry.filetype == Filetype::DriveView {
        match system::logical_drives() {
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
            path: system::computer_name(),
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
