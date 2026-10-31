use crate::structs::{Entry, Ui};

use console::style;

pub use crossterm::event::KeyModifiers;

use lnk::ShellLink;

use std::{env, fmt::Display, fs, panic, process};

pub fn err<S: Display>(msg: S) {
    eprintln!("{} {msg}", style("Error:").red().for_stderr());
}

pub fn resolve_lnk(path: &String) -> String {
    // Silence the expected panic, then put back the previous hook
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path, ansi_encoding()).unwrap());

    panic::set_hook(previous_hook);

    if link.is_err() {
        err(format!("Failed to read shortcut \"{}\"", pretty_path(path)));
        return path.clone();
    }

    let path_to_open = link
        .unwrap()
        .string_data()
        .relative_path()
        .as_ref()
        .and_then(|link_target| fs::canonicalize(link_target).ok());

    path_to_open.map_or_else(
        || path.clone(),
        |path_to_open| path_to_open.to_string_lossy().to_string(),
    )
}

// Encoding for non-Unicode .lnk strings: the system's active ANSI code page
#[cached::proc_macro::once]
fn ansi_encoding() -> lnk::Encoding {
    #[cfg(windows)]
    {
        let cp = unsafe { windows::Win32::Globalization::GetACP() };
        match cp {
            932 => encoding_rs::SHIFT_JIS,
            936 => encoding_rs::GBK,
            949 => encoding_rs::EUC_KR,
            950 => encoding_rs::BIG5,
            65001 => encoding_rs::UTF_8,
            _ => encoding_rs::Encoding::for_label(format!("windows-{cp}").as_bytes())
                .unwrap_or(encoding_rs::WINDOWS_1252),
        }
    }
    #[cfg(not(windows))]
    {
        encoding_rs::WINDOWS_1252
    }
}

// returns the index of the selected choice
pub fn display_choices(ui: &mut Ui, items: &[Entry], path: &str) -> (usize, KeyModifiers) {
    match ui.choose(items, pretty_path(path)) {
        Ok(Some(choice)) => choice,
        Ok(None) => process::exit(0),
        Err(error) => {
            err(format!("Failed to read input: {error}"));
            process::exit(1)
        }
    }
}

pub fn get_first_arg() -> Option<String> {
    env::args().nth(1).and_then(|arg| {
        let arg = arg.trim();
        if arg.is_empty() {
            None
        } else {
            Some(arg.to_owned())
        }
    })
}

pub fn pretty_path(path: &str) -> &str {
    if cfg!(windows) {
        path.trim_start_matches("\\\\?\\")
    } else {
        path
    }
}

pub fn link_with_label(path: &str, label: &str) -> String {
    format!("\u{1b}]8;;{path}\u{1b}\\{label}\u{1b}]8;;\u{1b}\\")
}

/**** WINDOWS ONLY ****/

#[cfg(windows)]
use std::io::Error;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

#[cfg(windows)]
pub fn get_logical_drives() -> Result<Vec<char>, Error> {
    let bitmask = unsafe { GetLogicalDrives() };
    if bitmask == 0 {
        return Err(Error::last_os_error());
    }

    Ok(bitmask_to_vec(bitmask))
}

#[cfg(windows)]
fn bitmask_to_vec(bitmask: u32) -> Vec<char> {
    let mut vec = Vec::new();
    for i in 0..32 {
        if bitmask & (1 << i) != 0 {
            vec.push((b'A' + i) as char);
        }
    }
    vec
}

#[cached::proc_macro::once]
pub fn get_computer_name() -> String {
    env::var("COMPUTERNAME").unwrap_or_else(|_| String::from("My Computer"))
}
