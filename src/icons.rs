const EXE: &str = "💿";
const TXT: &str = "📄";
const PIC: &str = "🖼️"; // 📷 🖼️
const VID: &str = "🎬";
const MUSIC: &str = "🎵"; //♫ ♬

pub const DIR: &str = "📁";
pub const LINK: &str = "🔗";
pub const UNKNOWN: &str = "❔";
pub const EXPLORER: &str = "💻"; //📂

pub fn from_ext(ext: &str) -> &str {
    match ext {
        "exe" | "bat" | "ps1" | "msi" | "cmd" | "com" | "pif" | "scr" | "vbs" | "vbe" | "jar"
        | "app" | "bin" => EXE,

        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "icn" | "webp" | "tiff" | "tif"
        | "svg" | "pdf" | "heif" | "psd" | "eps" | "jxl" | "icon" | "flif" | "avif" | "jp2"
        | "jpx" => PIC,

        "mp4" | "avi" | "mkv" | "webm" | "amc" | "vp6" | "mpeg" | "mpeg-2" | "wmv" | "m4u"
        | "264" | "ivr" | "media" | "mp5" | "flv" | "f4v" | "swf" | "video" | "ogv" | "av1"
        | "mp4v" | "mpeg4" | "m4v" | "mpg" | "mov" | "dvr" | "movie" | "dv" | "avchd" => VID,

        "mp3" | "wav" | "wave" | "ogg" | "flac" | "aac" | "m4a" | "wma" | "mka" | "m3u" | "pls"
        | "m3u8" | "aif" | "aiff" | "mid" | "ac3" | "opus" | "pcm" | "alac" | "weba" => MUSIC,

        "url" | "lnk" => LINK,

        _ => TXT,
    }
}
