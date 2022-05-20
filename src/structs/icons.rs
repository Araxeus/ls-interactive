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
    pub const EXE: Icon = Icon("💿"); // 💽 📀 💾
    pub const TXT: Icon = Icon("📄"); // 📰 📝 📖 📜 📒 📓 📑 🧾 📋 📇
    pub const PIC: Icon = Icon("🖼️"); // 📷 📸 🎨
    pub const VID: Icon = Icon("🎬"); // 🎞️ 📺 📹 📽️ 🎥 📼
    pub const MUSIC: Icon = Icon("🎵"); // 🎧 🔊 🎼 ♫ ♬ 📻
    pub const DIR: Icon = Icon("📁"); // 📂
    pub const LINK: Icon = Icon("🔗"); // 📎 🖇️
    pub const UNKNOWN: Icon = Icon("🚫"); // ❔ ❓ ⬜
    pub const EXPLORER: Icon = Icon("💻"); // 🗂️ 🗃️ 🗄️

    // Experimental: // TODO
    pub const SETTINGS: Icon = Icon("⚙️"); // 🎛️ 🔧 🔨 🛠️
    pub const WEB: Icon = Icon("🌐"); // 🌎 🌍 🌏
    pub const RUST: Icon = Icon("🦀"); // 🦞
    pub const JS: Icon = Icon("📒"); // 🇯 🐍 "\x1b[30;43m🇯\x1b[0m" = black on yellow
    pub const CSS: Icon = Icon("💄"); // 💅

    // TODO
    // Packages: 📦 (cargo.toml, package.json, etc)
    // Typescript: 📘
    // Python:🐍
    // C#: #️⃣
    // Ruby: 📕 🧧 🩸 🏮 🔶
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
            Filetype::Unknown => &Self::UNKNOWN,
        }
    }
}
