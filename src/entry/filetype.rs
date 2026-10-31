use std::{fs, path::Path};

#[derive(PartialEq, Eq, Clone)]
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
    Archive,
    Document,
    Spreadsheet,
    Markdown,
    Shell,
    Typescript,
    Python,
    DiskImage,
    Key,
    Font,
    Log,
    Ruby,
    Go,
    Java,
    Docker,
    Git,
    Lock,
    Make,
    // other:
    Directory,
    Symlink,
    Lnk,
    Unknown,
    #[allow(dead_code)]
    DriveView,
}

impl Filetype {
    // TODO
    // if metadata.permissions().mode() & 0o111 != 0 {
    //     Filetype::Executable
    // }

    pub const fn should_exec(&self) -> bool {
        !matches!(
            self,
            Self::Directory | Self::Symlink | Self::Lnk | Self::Unknown | Self::DriveView
        )
    }

    pub fn from_native(native: fs::FileType, path: &Path) -> Self {
        if native.is_file() {
            Self::from_file(path)
        } else if native.is_dir() {
            Self::Directory
        } else if native.is_symlink() {
            Self::Symlink
        } else {
            Self::Unknown
        }
    }

    fn from_file(path: &Path) -> Self {
        let name = path.file_name().unwrap_or_default().to_ascii_lowercase();
        let ext = path.extension().unwrap_or_default().to_ascii_lowercase();

        Self::from_name(name.to_str().unwrap_or_default())
            .unwrap_or_else(|| Self::from_ext(ext.to_str().unwrap_or_default()))
    }

    // Matches files known by their full name. Expects a lowercase name.
    fn from_name(name: &str) -> Option<Self> {
        let filetype = match name {
            "dockerfile"
            | "containerfile"
            | ".dockerignore"
            | "docker-compose.yml"
            | "docker-compose.yaml"
            | "compose.yml"
            | "compose.yaml" => Self::Docker,

            ".gitignore" | ".gitattributes" | ".gitmodules" | ".gitkeep" => Self::Git,

            "package-lock.json" | "pnpm-lock.yaml" | "bun.lockb" => Self::Lock,

            "makefile" | "gnumakefile" => Self::Make,

            _ if name.starts_with("dockerfile.") => Self::Docker,

            _ => return None,
        };

        Some(filetype)
    }

    fn from_ext(ext: &str) -> Self {
        match ext {
            "exe" | "msi" | "com" | "pif" | "scr" | "jar" | "app" => Self::Executable,

            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "icn" | "webp" | "tiff" | "tif"
            | "svg" | "heif" | "psd" | "eps" | "jxl" | "icon" | "flif" | "avif" | "jp2" | "jpx" => {
                Self::Picture
            }

            "mp4" | "avi" | "mkv" | "webm" | "amc" | "vp6" | "mpeg" | "mpeg-2" | "wmv" | "m4u"
            | "264" | "ivr" | "media" | "mp5" | "flv" | "f4v" | "swf" | "video" | "ogv" | "av1"
            | "mp4v" | "mpeg4" | "m4v" | "mpg" | "mov" | "dvr" | "movie" | "dv" | "avchd"
            | "vob" => Self::Video,

            "mp3" | "wav" | "wave" | "ogg" | "flac" | "aac" | "m4a" | "wma" | "mka" | "m3u"
            | "pls" | "m3u8" | "aif" | "aiff" | "mid" | "ac3" | "opus" | "pcm" | "alac"
            | "weba" => Self::Audio,

            "lnk" => Self::Lnk,

            "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "zst" | "7z" | "rar" => Self::Archive,

            "pdf" | "doc" | "docx" | "odt" | "rtf" | "epub" => Self::Document,

            "xls" | "xlsx" | "ods" | "csv" | "tsv" => Self::Spreadsheet,

            "md" | "mdx" | "rst" => Self::Markdown,

            "sh" | "bash" | "zsh" | "fish" | "nu" | "bat" | "cmd" | "ps1" | "psm1" | "vbs"
            | "vbe" => Self::Shell,

            "iso" | "img" | "dmg" | "vhd" | "vhdx" => Self::DiskImage,

            "pem" | "key" | "crt" | "cer" | "pub" | "gpg" | "asc" => Self::Key,

            "ttf" | "otf" | "woff" | "woff2" => Self::Font,

            "log" => Self::Log,

            "lock" => Self::Lock,

            "mk" => Self::Make,

            // Experimental // TODO
            "toml" | "ini" | "conf" | "json" | "yaml" | "yml" | "xml" => Self::Settings,

            "url" | "html" | "htm" => Self::Web,

            "rs" => Self::Rust,

            "js" | "mjs" | "cjs" | "jsx" => Self::Javascript,

            "ts" | "tsx" | "mts" | "cts" => Self::Typescript,

            "py" | "pyi" | "pyw" | "ipynb" => Self::Python,

            "rb" | "gemspec" => Self::Ruby,

            "go" => Self::Go,

            "java" | "class" | "kt" => Self::Java,

            "css" | "scss" => Self::Css,

            // Default
            _ => Self::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str) -> Filetype {
        Filetype::from_file(Path::new(name))
    }

    #[test]
    fn names_take_precedence_over_extensions() {
        assert!(file("Cargo.lock") == Filetype::Lock);
        assert!(file("package-lock.json") == Filetype::Lock);
        assert!(file("docker-compose.yml") == Filetype::Docker);
        assert!(file("compose.yaml") == Filetype::Docker);
    }

    #[test]
    fn matches_extensionless_and_dotfile_names() {
        assert!(file("Dockerfile") == Filetype::Docker);
        assert!(file("Dockerfile.dev") == Filetype::Docker);
        assert!(file(".gitignore") == Filetype::Git);
        assert!(file("Makefile") == Filetype::Make);
        assert!(file("LICENSE") == Filetype::Text);
    }

    #[test]
    fn sorts_moved_extensions_into_their_new_types() {
        assert!(file("report.PDF") == Filetype::Document);
        assert!(file("data.csv") == Filetype::Spreadsheet);
        assert!(file("index.htm") == Filetype::Web);
        assert!(file("app.mjs") == Filetype::Javascript);
        assert!(file("firmware.bin") == Filetype::Text);
    }
}
