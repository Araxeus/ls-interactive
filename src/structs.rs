use std::fmt;
use std::fs;
use std::path::Path;

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

#[derive(PartialEq)]
pub enum Filetype {
    // filetypes:
    Text, // (default)
    Executable,
    Picture,
    Video,
    Audio,
    Web,
    Rust,
    // other:
    Directory,
    Symlink,
    Lnk,
    Unknown,
}

impl Filetype {
    // DELETE
    // if metadata.permissions().mode() & 0o111 != 0 {
    //     Filetype::Executable
    // }

    pub const fn is_openable(&self) -> bool {
        !matches!(
            self,
            Self::Directory | Self::Symlink | Self::Lnk | Self::Unknown
        )
    }

    fn from_native(native: fs::FileType, path: &Path) -> Self {
        if native.is_file() {
            Self::from_ext(
                path.extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            )
        } else if native.is_dir() {
            Self::Directory
        } else if native.is_symlink() {
            Self::Symlink
        } else {
            Self::Unknown
        }
    }

    fn from_ext(ext: &str) -> Self {
        match ext {
            "exe" | "bat" | "ps1" | "msi" | "cmd" | "com" | "pif" | "scr" | "vbs" | "vbe"
            | "jar" | "app" | "bin" => Self::Executable,

            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "icn" | "webp" | "tiff" | "tif"
            | "svg" | "pdf" | "heif" | "psd" | "eps" | "jxl" | "icon" | "flif" | "avif" | "jp2"
            | "jpx" => Self::Picture,

            "mp4" | "avi" | "mkv" | "webm" | "amc" | "vp6" | "mpeg" | "mpeg-2" | "wmv" | "m4u"
            | "264" | "ivr" | "media" | "mp5" | "flv" | "f4v" | "swf" | "video" | "ogv" | "av1"
            | "mp4v" | "mpeg4" | "m4v" | "mpg" | "mov" | "dvr" | "movie" | "dv" | "avchd"
            | "vob" => Self::Video,

            "mp3" | "wav" | "wave" | "ogg" | "flac" | "aac" | "m4a" | "wma" | "mka" | "m3u"
            | "pls" | "m3u8" | "aif" | "aiff" | "mid" | "ac3" | "opus" | "pcm" | "alac"
            | "weba" => Self::Audio,

            "url" | "html" => Self::Web,

            "rs" => Self::Rust,

            "lnk" => Self::Lnk,

            _ => Self::Text,
        }
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

#[derive(PartialEq, Eq)] // TODO delete Eq?
pub struct Icon(&'static str);

impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[non_exhaustive]
pub struct Icons;

impl Icons {
    pub const EXE: Icon = Icon("💿");
    pub const TXT: Icon = Icon("📄"); // 📰
    pub const PIC: Icon = Icon("🖼️"); // 📷 🖼️
    pub const VID: Icon = Icon("🎬"); // 📹
    pub const MUSIC: Icon = Icon("🎵"); // 🎧 ♫ ♬

    pub const WEB: Icon = Icon("🌐");
    pub const RUST: Icon = Icon("🦀"); // 📦

    // TODO package files? (cargo.toml, package.json, etc)
    pub const DIR: Icon = Icon("📁");
    pub const LINK: Icon = Icon("🔗"); // 📎

    pub const UNKNOWN: Icon = Icon("📁");
    pub const EXPLORER: Icon = Icon("💻"); //📂

    pub const fn from_filetype(filetype: &Filetype) -> &'static Icon {
        match filetype {
            Filetype::Text => &Self::TXT,
            Filetype::Executable => &Self::EXE,
            Filetype::Picture => &Self::PIC,
            Filetype::Video => &Self::VID,
            Filetype::Audio => &Self::MUSIC,
            Filetype::Directory => &Self::DIR,
            Filetype::Symlink | Filetype::Lnk => &Self::LINK,
            Filetype::Web => &Self::WEB,
            Filetype::Rust => &Self::RUST,
            Filetype::Unknown => &Self::UNKNOWN,
        }
    }
}
