use std::env;
use std::fs;

fn main() {
    let res = selector();
    println!("{:?}", res);
    let args: Vec<String> = env::args().collect();
    let path = 
        if args.len() >= 2 { &args[1].trim() }
        else {"."};
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    let icon = String::from(
                        if file_type.is_file() {"📄"}
                        else if file_type.is_dir() {"📁"}
                        else if file_type.is_symlink() {"🔗"}
                        else {"❔"}
                    );
                    println!("{} {}", icon, entry.file_name().to_str().unwrap());
                }
            }
        }
    }
}

use dialoguer::{
    Select,
    theme::ColorfulTheme
};
use console::Term;

fn select(items: Vec<&str>) -> String {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr()).ok().unwrap();

    match selection {
        Some(index) => format!("User selected item : {}", items[index]),
        None => format!("User did not select anything")
    }
}

fn selector() -> String {
    let items = vec!["Item 1", "item 2"];

    select(items)
}

