//! Windows shortcuts (`.lnk` files).

use std::{fs, panic};

use lnk::ShellLink;

/// Where the shortcut at `path` points. Gives `path` back when the target is missing, and
/// `None` when the shortcut cannot be read.
pub fn resolve(path: &str) -> Option<String> {
    // Silence the expected panic, then put back the previous hook
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path, ansi_encoding()).unwrap());

    panic::set_hook(previous_hook);

    let target = link
        .ok()?
        .string_data()
        .relative_path()
        .as_ref()
        .and_then(|target| fs::canonicalize(target).ok());

    Some(target.map_or_else(
        || path.to_owned(),
        |target| target.to_string_lossy().to_string(),
    ))
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
