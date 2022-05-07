use std::fmt;

pub struct Entry {
    pub name: String,
    pub path: String,
    pub icon: Icon,
    pub is_dir: bool,
    pub is_link: bool,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.icon, self.name)
    }
}

#[derive(PartialEq, Eq)]
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

    pub const DIR: Icon = Icon("📁");
    pub const LINK: Icon = Icon("🔗"); // 📎

    pub const UNKNOWN: Icon = Icon("📁");
    pub const EXPLORER: Icon = Icon("💻"); //📂

    pub fn from_ext(ext: &str) -> Icon {
        match ext {
            "exe" | "bat" | "ps1" | "msi" | "cmd" | "com" | "pif" | "scr" | "vbs" | "vbe"
            | "jar" | "app" | "bin" => Self::EXE,

            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "icn" | "webp" | "tiff" | "tif"
            | "svg" | "pdf" | "heif" | "psd" | "eps" | "jxl" | "icon" | "flif" | "avif" | "jp2"
            | "jpx" => Self::PIC,

            "mp4" | "avi" | "mkv" | "webm" | "amc" | "vp6" | "mpeg" | "mpeg-2" | "wmv" | "m4u"
            | "264" | "ivr" | "media" | "mp5" | "flv" | "f4v" | "swf" | "video" | "ogv" | "av1"
            | "mp4v" | "mpeg4" | "m4v" | "mpg" | "mov" | "dvr" | "movie" | "dv" | "avchd"
            | "vob" => Self::VID,

            "mp3" | "wav" | "wave" | "ogg" | "flac" | "aac" | "m4a" | "wma" | "mka" | "m3u"
            | "pls" | "m3u8" | "aif" | "aiff" | "mid" | "ac3" | "opus" | "pcm" | "alac"
            | "weba" => Self::MUSIC,

            "url" | "lnk" => Self::LINK,

            _ => Self::TXT,
        }
    }
}

