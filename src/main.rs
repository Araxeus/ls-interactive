mod structs;

use std::env;
use std::fs;
use std::path::Path;

use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use structs::{Icons, Entry};

fn main() {
    human_panic::setup_panic!();
    let args: Vec<String> = env::args().collect();
    let path = if args.len() >= 2 { args[1].trim() } else { "." };

    let path = fs::canonicalize(path);

    match path {
        Ok(path) => open_entry(path.to_str().unwrap()),
        Err(_) => println!("Invalid path"),
    }
}

fn open_entry(path: &str) {
    let dir = get_dir(path);

    let entry = &dir[select(&dir, path)];
    if entry.is_dir || entry.is_link {
        open_entry(&entry.path);
    } else {
        open::that(&entry.path).unwrap();
    }
}

fn get_dir(path: &str) -> Vec<Entry> {
    let mut result_vector: Vec<Entry> = Vec::new();

    // .. Open parent directory
    if let Ok(parent) = Path::new(path).parent().ok_or("No parent") {
        result_vector.push(Entry {
            name: String::from(".."),
            path: parent.to_str().unwrap().to_string(),
            icon: Icons::DIR,
            is_dir: true,
            is_link: false,
        });
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                let icon = if file_type.is_file() {
                    Icons::from_ext(
                        entry
                            .path()
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                } else if file_type.is_dir() {
                    Icons::DIR
                } else if file_type.is_symlink() {
                    Icons::LINK
                } else {
                    Icons::UNKNOWN
                };
                result_vector.push(Entry {
                    name: entry.file_name().to_str().unwrap().to_string(),
                    path: entry.path().to_str().unwrap().to_string(),
                    is_link: icon == Icons::LINK,
                    is_dir: file_type.is_dir(),
                    icon,
                });
            }
        }
    }

    // Open current directory in explorer
    result_vector.push(Entry {
        name: String::new(),
        path: path.to_string(),
        icon: Icons::EXPLORER,
        is_dir: false,
        is_link: false,
    });

    result_vector
}

fn select(items: &[Entry], path: &str) -> usize {
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
