use super::{Filetype, Icon, Icons};

use std::fmt;
use std::fs;

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
    // pub fn to_string(self) -> String {
    //     format!("{} {}", self.icon, self.name)
    // }

    #[allow(clippy::needless_pass_by_value)]
    pub fn from_dir_entry(entry: fs::DirEntry) -> Self {
        let path = entry.path();

        let native_file_type = entry.file_type().unwrap();

        let filetype = Filetype::from_native(native_file_type, &path);

        Self {
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_str().unwrap().to_string(),
            icon: Icons::from_filetype(&filetype),
            filetype,
        }
    }
}
