use crate::structs::{ColorfulTheme, Entry, Prompt};

use console::style;

pub use crossterm::event::KeyModifiers;

use lnk::ShellLink;

use std::{env, fmt::Display, fs, mem, panic, process};

pub fn err<S: Display>(msg: S) {
    eprintln!("{} {msg}", style("Error:").red().for_stderr());
}

pub fn resolve_lnk(path: &String) -> String {
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path).unwrap());

    mem::drop(panic::take_hook());

    if link.is_err() {
        err(format!("Failed to read shortcut \"{}\"", pretty_path(path)));
        return path.to_string();
    }

    let path_to_open = link
        .unwrap()
        .relative_path()
        .as_ref()
        .and_then(|link_target| fs::canonicalize(link_target).ok());

    path_to_open.map_or_else(
        || path.to_string(),
        |path_to_open| path_to_open.to_string_lossy().to_string(),
    )
}

// returns the index of the selected choice
pub fn display_choices(items: &[Entry], path: &str) -> (usize, KeyModifiers) {
    Prompt::with_theme(&ColorfulTheme::default())
        .title(pretty_path(path))
        .items(items)
        .run()
        .unwrap()
        .map_or_else(|| process::exit(0), |res| res)
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

/**** WINDOWS ONLY ****/

#[cfg(windows)]
use std::io::Error;
#[cfg(windows)]
use windows::{
    core::PWSTR,
    Win32::{
        Storage::FileSystem::GetLogicalDrives,
        System::SystemInformation::{ComputerNameNetBIOS, GetComputerNameExW},
    },
};

#[cfg(windows)]
pub fn get_logical_drives() -> Result<Vec<char>, Error> {
    let bitmask = unsafe { GetLogicalDrives() };
    if bitmask == 0 {
        return Err(Error::last_os_error());
    }
    let mut result: Vec<char> = vec![];
    let mut mask = 1;
    for index in 1..26 {
        if mask & bitmask == mask {
            let char = char::from_u32(index + 64);
            result.push(char.unwrap());
        }
        mask <<= 1;
    }
    Ok(result)
}

#[cfg(windows)]
pub fn get_pc_name() -> String {
    let mut buffer = [0u16; 256];
    #[allow(clippy::cast_possible_truncation)]
    let mut size = buffer.len() as u32;
    unsafe {
        GetComputerNameExW(ComputerNameNetBIOS, PWSTR(buffer.as_mut_ptr()), &mut size);
    }
    let name = String::from_utf16_lossy(&buffer);
    format!("{name}:\\\\")
}
