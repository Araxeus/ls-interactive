//! Helpers that depend on the operating system.

use std::{env, path::PathBuf};

/// `path` with a leading `~` replaced by the home directory.
///
/// Shells such as PowerShell and cmd pass `~` to programs untouched, so lsi expands it itself.
pub fn expand_tilde(path: &str) -> PathBuf {
    expand_tilde_with(path, env::home_dir())
}

fn expand_tilde_with(path: &str, home: Option<PathBuf>) -> PathBuf {
    let Some(rest) = path.strip_prefix('~') else {
        return PathBuf::from(path);
    };

    let separators: &[char] = if cfg!(windows) { &['/', '\\'] } else { &['/'] };

    match home {
        Some(home) if rest.is_empty() => home,
        Some(home) if rest.starts_with(separators) => {
            home.join(rest.trim_start_matches(separators))
        }
        _ => PathBuf::from(path),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/home/me";

    #[test]
    fn tilde_alone_is_home() {
        assert_eq!(
            expand_tilde_with("~", Some(HOME.into())),
            PathBuf::from(HOME)
        );
    }

    #[test]
    fn tilde_slash_joins_home() {
        assert_eq!(
            expand_tilde_with("~/code/lsi", Some(HOME.into())),
            PathBuf::from(HOME).join("code/lsi")
        );
    }

    #[cfg(windows)]
    #[test]
    fn tilde_backslash_joins_home_on_windows() {
        assert_eq!(
            expand_tilde_with(r"~\code", Some(HOME.into())),
            PathBuf::from(HOME).join("code")
        );
    }

    #[test]
    fn tilde_inside_a_name_is_kept() {
        assert_eq!(
            expand_tilde_with("~user", Some(HOME.into())),
            PathBuf::from("~user")
        );
        assert_eq!(
            expand_tilde_with("a/~", Some(HOME.into())),
            PathBuf::from("a/~")
        );
    }

    #[test]
    fn tilde_is_kept_without_a_home() {
        assert_eq!(expand_tilde_with("~/x", None), PathBuf::from("~/x"));
    }
}
