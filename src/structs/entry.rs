use super::{Filetype, Icon, Icons};

use std::{fmt, fs};

pub struct Entry {
    pub name: String,
    pub path: String,
    pub icon: &'static Icon,
    pub filetype: Filetype,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            console::Emoji(
                self.icon.to_str(),
                if self.filetype == Filetype::Directory {
                    "\\"
                } else {
                    " "
                }
            ),
            self.name
        )
    }
}

impl Entry {
    pub fn from_dir_entry(entry: &fs::DirEntry) -> Self {
        let path = entry.path();

        let filetype = match entry.file_type() {
            Ok(native_file_type) => Filetype::from_native(native_file_type, &path),
            Err(_) => Filetype::Unknown,
        };

        Self {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            icon: Icons::from_filetype(&filetype),
            filetype,
        }
    }
}
