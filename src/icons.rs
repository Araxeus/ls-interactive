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
        // EXE
        "exe" => EXE,
        "bat" => EXE,
        "ps1" => EXE,
        "msi" => EXE,

        // PIC
        "png" => PIC,
        "jpg" => PIC,
        "jpeg" => PIC,
        "gif" => PIC,
        "bmp" => PIC,
        "ico" => PIC,
        "webp" => PIC,

        // VID
        "mp4" => VID,
        "avi" => VID,
        "mkv" => VID,

        // MUSIC
        "mp3" => MUSIC,
        "wav" => MUSIC,
        "ogg" => MUSIC,
        "flac" => MUSIC,
        "aac" => MUSIC,
        "m4a" => MUSIC,
        "wma" => MUSIC,
        "mka" => MUSIC,
        "m3u" => MUSIC,
        "pls" => MUSIC,
        "m3u8" => MUSIC,

        // LINKS
        "url" => LINK,
        "lnk" => LINK,

        // OTHER
        _ => TXT,
    }
}
