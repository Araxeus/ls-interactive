mod icons;

use std::env;
use std::fmt;
use std::fs;
use std::path::Path;

struct Entry {
    name: String,
    path: String,
    icon: String,
    is_dir: bool,
    is_link: bool,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.icon, self.name)
    }
}

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
            icon: String::from(icons::DIR),
            is_dir: true,
            is_link: false,
        });
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                let ext = entry.path();
                let icon = String::from(if file_type.is_file() {
                    icons::from_ext(
                        ext.extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                } else if file_type.is_dir() {
                    icons::DIR
                } else if file_type.is_symlink() {
                    icons::LINK
                } else {
                    icons::UNKNOWN
                });
                let entry = Entry {
                    name: entry.file_name().to_str().unwrap().to_string(),
                    path: entry.path().to_str().unwrap().to_string(),
                    is_link: icon == icons::LINK,
                    is_dir: file_type.is_dir(),
                    icon,
                };
                result_vector.push(entry);
            }
        }
    }

    // Open current directory in explorer
    result_vector.push(Entry {
        name: String::new(),
        path: path.to_string(),
        icon: String::from(icons::EXPLORER),
        is_dir: false,
        is_link: false,
    });

    result_vector
}

use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

fn select(items: &[Entry], path: &str) -> usize {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&path[4..])
        .report(false)
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .ok()
        .unwrap();

    match selection {
        Some(index) => index,
        // exit process if none
        None => std::process::exit(0),
    }
}
