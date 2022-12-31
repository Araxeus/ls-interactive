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
        write!(f, "{} {}", self.icon, self.name)
    }
}

impl Entry {
    pub fn from_dir_entry(entry: &fs::DirEntry) -> Self {
        let path = entry.path();

        let filetype = entry
            .file_type()
            .map_or(Filetype::Unknown, |native_file_type| {
                Filetype::from_native(native_file_type, &path)
            });

        Self {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            icon: Icons::from_filetype(&filetype),
            filetype,
        }
    }
}
