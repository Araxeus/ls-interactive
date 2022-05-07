use std::{fmt::Display, fs, mem, panic};

use console::style;
use lnk::ShellLink;

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

pub fn err<S: Display>(msg: S) {
    println!("{} {}", style("Error").red(), msg);
}
