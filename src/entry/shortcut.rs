//! Windows shortcuts (`.lnk` files).

use std::{
    fs, panic,
    path::{Path, PathBuf},
};

use lnk::ShellLink;

/// Where the shortcut at `path` points. Gives `path` back when the target is missing, and
/// `None` when the shortcut cannot be read.
pub fn resolve(path: &str) -> Option<String> {
    // Silence the expected panic, then put back the previous hook
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_info| {}));

    let link = panic::catch_unwind(|| ShellLink::open(path, ansi_encoding()).unwrap());

    panic::set_hook(previous_hook);

    let folder = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
    let target = targets(&link.ok()?, folder)
        .into_iter()
        .find_map(|target| fs::canonicalize(target).ok());

    Some(target.map_or_else(
        || path.to_owned(),
        |target| target.to_string_lossy().to_string(),
    ))
}

/// The paths a shortcut in `folder` may point to, most reliable first.
fn targets(link: &ShellLink, folder: &Path) -> Vec<PathBuf> {
    let mut targets = Vec::new();

    if let Some(info) = link.link_info() {
        let base = info
            .local_base_path_unicode()
            .as_deref()
            .or_else(|| info.local_base_path());
        let suffix = info
            .common_path_suffix_unicode()
            .as_deref()
            .unwrap_or_else(|| info.common_path_suffix());
        if let Some(base) = base {
            targets.push(PathBuf::from(format!("{base}{suffix}")));
        }
    }

    // The relative path starts at the shortcut's own folder
    if let Some(relative) = link.string_data().relative_path() {
        targets.push(folder.join(relative));
    }

    targets
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal Unicode shortcut that stores only a relative path.
    fn relative_shortcut(relative: &str) -> Vec<u8> {
        const HAS_RELATIVE_PATH: u32 = 0x08;
        const IS_UNICODE: u32 = 0x80;
        const SHOW_NORMAL: u8 = 1;
        const LINK_CLSID: [u8; 16] = [
            0x01, 0x14, 0x02, 0, 0, 0, 0, 0, 0xC0, 0, 0, 0, 0, 0, 0, 0x46,
        ];

        let mut bytes = vec![0; 76];
        bytes[0..4].copy_from_slice(&76u32.to_le_bytes());
        bytes[4..20].copy_from_slice(&LINK_CLSID);
        bytes[20..24].copy_from_slice(&(HAS_RELATIVE_PATH | IS_UNICODE).to_le_bytes());
        bytes[60] = SHOW_NORMAL;

        let chars: Vec<u16> = relative.encode_utf16().collect();
        bytes.extend(u16::try_from(chars.len()).unwrap().to_le_bytes());
        bytes.extend(chars.iter().flat_map(|c| c.to_le_bytes()));
        bytes.extend(0u32.to_le_bytes());
        bytes
    }

    #[test]
    fn resolves_a_relative_target_from_the_shortcut_folder() {
        let folder = std::env::temp_dir().join(format!("lsi-shortcut-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("target.txt"), "").unwrap();
        let link = folder.join("link.lnk");
        fs::write(&link, relative_shortcut(".\\target.txt")).unwrap();

        let resolved = resolve(link.to_str().unwrap());
        let expected = fs::canonicalize(folder.join("target.txt")).unwrap();
        fs::remove_dir_all(&folder).unwrap();

        assert_eq!(resolved, Some(expected.to_string_lossy().to_string()));
    }
}
