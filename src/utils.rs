use std::fs;

use lnk::ShellLink;

pub fn lnk_to_link (path: &String) -> String {
    let link =ShellLink::open(path).unwrap();
    let link_target = link.relative_path().as_ref().unwrap();
    println!("{}", link_target); // DEBUG
    let path_to_open = fs::canonicalize(link_target);
    match path_to_open {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => path.to_string(),
    }
}
