//! Helpers that depend on the operating system.

use std::env;

/// `path` without the `\\?\` prefix Windows puts in front of canonical paths.
pub fn pretty_path(path: &str) -> &str {
    if cfg!(windows) {
        path.trim_start_matches("\\\\?\\")
    } else {
        path
    }
}

/// The name of this computer, which stands for the list of drives on Windows.
#[cached::proc_macro::once]
pub fn computer_name() -> String {
    env::var("COMPUTERNAME").unwrap_or_else(|_| String::from("My Computer"))
}

/// The letters of the drives Windows has mounted.
#[cfg(windows)]
pub fn logical_drives() -> std::io::Result<Vec<char>> {
    use windows::Win32::Storage::FileSystem::GetLogicalDrives;

    let bitmask = unsafe { GetLogicalDrives() };
    if bitmask == 0 {
        return Err(std::io::Error::last_os_error());
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
