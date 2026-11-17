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
use system::{expand_tilde, pretty_path};

fn main() {
    human_panic::setup_panic!();

    // A panic must not leave the terminal in raw mode with a hidden cursor
    let report_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        report_panic(info);
    }));

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

    fs::canonicalize(expand_tilde(path)).map_or_else(
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
        let target = modified_target(&selected_entry, &entry);

        if modifier == KeyModifiers::SHIFT || modifier == KeyModifiers::ALT {
            finish(&mut ui);
            print!("{}", pretty_path(target));
            break;
        }

        // A shortcut to a folder browses it. Any other shortcut opens like a file.
        let shortcut_folder = (entry.filetype == Filetype::Lnk)
            .then(|| shortcut::folder(&entry.path))
            .flatten();
        let opens = entry.filetype.should_exec()
            || (entry.filetype == Filetype::Lnk && shortcut_folder.is_none());

        // exec file
        if opens || modifier == KeyModifiers::CONTROL {
            if no_exec {
                finish(&mut ui);
                print!("{}", pretty_path(target));
                break;
            }
            let opened = open::that(target);
            finish(&mut ui);

            // quit if file was opened, else display error and open as directory
            if opened.is_ok() {
                break;
            }
            err(format!("Failed to open file \"{}\"", pretty_path(target)));

            // The folder being browsed failed to open, so its listing stays up
            if entry.name == ".." {
                continue;
            }
        }

        // browse directory by continuing loop with new path
        selected_entry = entry;
        if let Some(folder) = shortcut_folder {
            selected_entry.path = folder;
        }
    }
}

/// The path that Enter with a modifier acts on. For the `..` entry that is the folder being
/// browsed, because its parent is already one plain Enter away.
fn modified_target<'a>(current: &'a Entry, chosen: &'a Entry) -> &'a str {
    if chosen.name == ".." {
        &current.path
    } else {
        &chosen.path
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
            icon: &Icons::PARENT,
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

    // Get files in directory. A link that denies listing, like `C:\Documents and Settings`,
    // lists its target, and its entries keep paths under the link.
    let folder = Path::new(&entry.path);
    let listing = fs::read_dir(folder).or_else(|_| fs::read_dir(fs::canonicalize(folder)?));
    if let Ok(entries) = listing {
        for dir_entry in entries.flatten() {
            let mut item = Entry::from_dir_entry(&dir_entry);
            item.path = folder
                .join(dir_entry.file_name())
                .to_string_lossy()
                .to_string();
            result_vector.push(item);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str, path: &str) -> Entry {
        Entry {
            name: name.to_owned(),
            path: path.to_owned(),
            icon: &Icons::PARENT,
            filetype: Filetype::Directory,
        }
    }

    #[test]
    fn a_modified_enter_on_the_parent_entry_targets_the_folder_being_browsed() {
        let current = folder("project", "/home/user/project");
        let parent = folder("..", "/home/user");
        assert_eq!(modified_target(&current, &parent), "/home/user/project");
    }

    #[test]
    fn a_modified_enter_on_the_listed_parent_entry_targets_the_folder_being_browsed() {
        // The folder does not have to exist to be listed with its parent entry
        let current = folder("missing", "/lsi/missing");
        let choices = get_choices(&current);
        assert_eq!(choices[0].name, "..");
        assert_eq!(modified_target(&current, &choices[0]), "/lsi/missing");
    }

    #[cfg(windows)]
    #[test]
    fn lists_a_junction_that_denies_listing_through_its_target() {
        let junction = r"C:\Documents and Settings";
        let choices = get_choices(&folder("Documents and Settings", junction));

        let listed: Vec<_> = choices
            .iter()
            .filter(|choice| choice.name != "..")
            .collect();
        assert!(listed.iter().all(|choice| choice.name != "<Open>"));
        assert!(listed
            .iter()
            .all(|choice| choice.path.starts_with(&format!(r"{junction}\"))));
    }

    #[test]
    fn a_modified_enter_on_any_other_entry_targets_that_entry() {
        // The first entry is not `..` in the root folder and in the drive view
        let current = folder("", "/");
        let first = folder("bin", "/bin");
        assert_eq!(modified_target(&current, &first), "/bin");
    }
}
