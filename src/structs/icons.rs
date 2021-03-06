use super::Filetype;

use std::fmt;

pub struct Icon(&'static str);

impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[non_exhaustive]
pub struct Icons;

impl Icons {
    pub const EXE: Icon = Icon("๐ฟ"); // ๐ฝ ๐ ๐พ
    pub const TXT: Icon = Icon("๐"); // ๐ฐ ๐ ๐ ๐ ๐ ๐ ๐ ๐งพ ๐ ๐
    pub const PIC: Icon = Icon("๐ผ๏ธ"); // ๐ท ๐ธ ๐จ
    pub const VID: Icon = Icon("๐ฌ"); // ๐๏ธ ๐บ ๐น ๐ฝ๏ธ ๐ฅ ๐ผ
    pub const MUSIC: Icon = Icon("๐ต"); // ๐ง ๐ ๐ผ โซ โฌ ๐ป
    pub const DIR: Icon = Icon("๐"); // ๐
    pub const LINK: Icon = Icon("๐"); // ๐ ๐๏ธ
    pub const UNKNOWN: Icon = Icon("๐ซ"); // โ โ โฌ
    pub const EXPLORER: Icon = Icon("๐ป"); // ๐๏ธ ๐๏ธ ๐๏ธ

    // Experimental: // TODO
    pub const SETTINGS: Icon = Icon("โ๏ธ"); // ๐๏ธ ๐ง ๐จ ๐ ๏ธ
    pub const WEB: Icon = Icon("๐"); // ๐ ๐ ๐
    pub const RUST: Icon = Icon("๐ฆ"); // ๐ฆ
    pub const JS: Icon = Icon("๐"); // ๐ฏ ๐ "\x1b[30;43m๐ฏ\x1b[0m" = black on yellow
    pub const CSS: Icon = Icon("๐"); // ๐

    // TODO
    // Packages: ๐ฆ (cargo.toml, package.json, etc)
    // Typescript: ๐
    // Python:๐
    // C#: #๏ธโฃ
    // Ruby: ๐ ๐งง ๐ฉธ ๐ฎ ๐ถ
    // CoffeeScript: ๐ต
    // Recycle bin: ๐๏ธ
    // Others: ใฝ๏ธ ๐ฐ ๐ก๏ธ ๐งผ โฐ ๐งฌ ๐
    // ๐ท๏ธ ๐ธ๏ธ ๐ฆ ๐ ๐ง โ๏ธ ๐งฟ ๐  ๐ฝ ๐ฆ ๐ ๐ ๐ฆ ๐ฆ๐ ๐ถ โต ๐ท๏ธ ๐ธ ๐ ๐จ
    // โ๏ธ ๐ ๐ ๐ ๐ ๐ ๐ฒ๏ธ ๐ฑ๏ธ โจ๏ธ ๐ ๐ โ๏ธ ๐ฑ ๐๏ธ ๐๏ธ ๐ฏ ๐ ๐ฟ ๐๏ธ ๐งถ ๐งต ๐งธ
    // ๐ฎ ๐น๏ธ ๐ช ๐ ๐ช ๐ ๐งง ๐ก๏ธ โ ๐งฑ ๐ ๐๏ธ ๐ฃ ๐บ๏ธ ๐บ๏ธ ๐ฟ ๐ฝ ๐งญ ๐งฑ ๐งฐ ๐งฒ ๐งฎ
    // ๐ ๐ ๐ ๐ท๏ธ ๐ฐ ๐ฏ๏ธ ๐ก ๐ช ๐ณ โ๏ธ ๐ง ๐ฎ ๐ณ๏ธ โ๏ธ ๐๏ธ ๐๏ธ ๐ ๐ ๐ ๐ ๐
    // ๐ฉธ ๐ฎ ๐งฏ ๐ ๐ โ๏ธ ๐ โฑ๏ธ ๐ ๐ ๐ ๐ ๐จ ๐ช ๐งด โ๏ธ ๐งช ๐งซ ๐ก ๐ฌ ๐ฌ ๐
    // ๐ฉ โ๏ธ โ๏ธ ๐๏ธ ๐ฉบ ๐ฟ ๐๏ธโ๐จ๏ธ ๐ ๐ ๐ ๐ ๐ถ โ๏ธ ๐ ๐  โ๏ธ โน๏ธ *๏ธโฃ ๐ด ๐ฆ ๐ ๐ฆง

    pub const fn from_filetype(filetype: &Filetype) -> &'static Icon {
        match filetype {
            Filetype::Text => &Self::TXT,
            Filetype::Executable => &Self::EXE,
            Filetype::Picture => &Self::PIC,
            Filetype::Video => &Self::VID,
            Filetype::Audio => &Self::MUSIC,
            Filetype::Directory => &Self::DIR,
            Filetype::Symlink | Filetype::Lnk => &Self::LINK,
            Filetype::Settings => &Self::SETTINGS,
            Filetype::Web => &Self::WEB,
            Filetype::Rust => &Self::RUST,
            Filetype::Javascript => &Self::JS,
            Filetype::Css => &Self::CSS,
            Filetype::Unknown => &Self::UNKNOWN,
        }
    }
}
