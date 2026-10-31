use super::Filetype;

use std::fmt;

#[derive(PartialEq, Eq)]
pub struct Icon(&'static str);

// impl Icon {
//     pub const fn str(&self) -> &'static str {
//         self.0
//     }
// }

impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[non_exhaustive]
pub struct Icons;

impl Icons {
    pub const EXE: Icon = Icon("💿"); //  📀 💾
    pub const TXT: Icon = Icon("📄"); // 📰 📝 📖 📜 📒 📓 📑 🧾 📋 📇
    pub const PIC: Icon = Icon("🖼️"); // 📷 📸 🎨
    pub const VID: Icon = Icon("🎬"); // 🎞️ 📺 📹 📽️ 🎥 📼
    pub const MUSIC: Icon = Icon("🎵"); // 🎧 🔊 🎼 ♫ ♬ 📻
    pub const DIR: Icon = Icon("📁");
    pub const PARENT: Icon = Icon("📂");
    pub const LINK: Icon = Icon("🔗"); // 📎 🖇️
    pub const UNKNOWN: Icon = Icon("🚫"); // ❔ ❓ ⬜
    pub const EXPLORER: Icon = Icon("💻"); // 🗂️ 🗃️ 🗄️

    // Experimental: // TODO
    pub const SETTINGS: Icon = Icon("⚙️"); // 🎛️ 🔧 🔨 🛠️
    pub const WEB: Icon = Icon("🌐"); // 🌎 🌍 🌏
    pub const RUST: Icon = Icon("🦀"); // 🦞
    pub const JS: Icon = Icon("📒"); // 🇯 🐍 "\x1b[30;43m🇯\x1b[0m" = black on yellow
    pub const CSS: Icon = Icon("💄"); // 💅

    pub const ARCHIVE: Icon = Icon("📦");
    pub const DOCUMENT: Icon = Icon("📕");
    pub const SPREADSHEET: Icon = Icon("📊");
    pub const MARKDOWN: Icon = Icon("📝");
    pub const SHELL: Icon = Icon("📜");
    pub const TS: Icon = Icon("📘");
    pub const PYTHON: Icon = Icon("🐍");
    pub const KEY: Icon = Icon("🔑");
    pub const FONT: Icon = Icon("🔤");
    pub const LOG: Icon = Icon("🧾");
    pub const RUBY: Icon = Icon("💎");
    pub const GO: Icon = Icon("🐹");
    pub const JAVA: Icon = Icon("☕");
    pub const DOCKER: Icon = Icon("🐳");
    pub const GIT: Icon = Icon("🌳");
    pub const LOCK: Icon = Icon("🔒");
    pub const MAKE: Icon = Icon("🔨");

    // Windows Only
    pub const PC: Icon = Icon("🖥️");

    // Drives on Windows, disk images everywhere
    pub const DRIVE: Icon = Icon("💽");

    // TODO
    // C#: #️⃣
    // CoffeeScript: 🍵
    // Recycle bin: 🗑️
    // Others: 〽️ 🔰 🛡️ 🧼 ➰ 🧬 🛒
    // 🕷️ 🕸️ 🦄 🐋 💧 ☄️ 🧿 🐠 🐽 🦃 🐉 🌈 🦜 🐦💊 🛶 ⛵ 🛷️ 🛸 📚 💨
    // ✏️ 📕 📗 📘 📙 📔 🖲️ 🖱️ ⌨️ 🔋 📟 ☎️ 📱 🎚️ 🎙️ 📯 💎 📿 🛍️ 🧶 🧵 🧸
    // 🔮 🕹️ 🪁 🎈 🪀 🎁 🧧 🌡️ ⌛ 🧱 💈 🛎️ 💣 🗺️ 🏺️ 🗿 🗽 🧭 🧱 🧰 🧲 🧮
    // 📏 📐 🔖 🏷️ 💰 🕯️ 💡 🪔 💳 ✉️ 📧 📮 🗳️ ✒️ 🖌️ 🖍️ 📅 📆 📈 📉 📊
    // 🩸 🏮 🧯 📌 📍 ✂️ 💉 ⚱️ 🔐 🔒 🔓 🔑 🔨 🪓 🧴 ⚗️ 🧪 🧫 📡 🔬 💬 🐒
    // 🔩 ⚖️ ⛓️ 🗜️ 🩺 🚿 👁️‍🗨️ 🛑 🌀 🔅 🔆 📶 ⚜️ 📛 💠 Ⓜ️ ℹ️ *️⃣ 🌴 🦂 💍 🦧

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
            Filetype::Archive => &Self::ARCHIVE,
            Filetype::Document => &Self::DOCUMENT,
            Filetype::Spreadsheet => &Self::SPREADSHEET,
            Filetype::Markdown => &Self::MARKDOWN,
            Filetype::Shell => &Self::SHELL,
            Filetype::Typescript => &Self::TS,
            Filetype::Python => &Self::PYTHON,
            Filetype::DiskImage => &Self::DRIVE,
            Filetype::Key => &Self::KEY,
            Filetype::Font => &Self::FONT,
            Filetype::Log => &Self::LOG,
            Filetype::Ruby => &Self::RUBY,
            Filetype::Go => &Self::GO,
            Filetype::Java => &Self::JAVA,
            Filetype::Docker => &Self::DOCKER,
            Filetype::Git => &Self::GIT,
            Filetype::Lock => &Self::LOCK,
            Filetype::Make => &Self::MAKE,
            Filetype::DriveView => &Self::PC,
            Filetype::Unknown => &Self::UNKNOWN,
        }
    }
}
