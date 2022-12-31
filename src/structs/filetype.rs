use std::{fs, path::Path};

#[derive(PartialEq, Eq)]
pub enum Filetype {
    Text, // (default)
    Executable,
    Picture,
    Video,
    Audio,
    Settings,
    Web,
    Rust,
    Javascript,
    Css,
    // other:
    Directory,
    Symlink,
    Lnk,
    Unknown,
}

impl Filetype {
    // TODO
    // if metadata.permissions().mode() & 0o111 != 0 {
    //     Filetype::Executable
    // }

    pub const fn should_exec(&self) -> bool {
        !matches!(
            self,
            Self::Directory | Self::Symlink | Self::Lnk | Self::Unknown
        )
    }

    pub fn from_native(native: fs::FileType, path: &Path) -> Self {
        if native.is_file() {
            Self::from_ext(
                path.extension()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
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

            "lnk" => Self::Lnk,

            // Experimental // TODO
            "toml" | "ini" | "conf" | "json" | "yaml" | "yml" | "xml" | "csv" => Self::Settings,

            "url" | "html" => Self::Web,

            "rs" => Self::Rust,

            "js" => Self::Javascript,

            "css" | "scss" => Self::Css,

            // Default
            _ => Self::Text,
        }
    }
}
