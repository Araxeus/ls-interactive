use console::style;
use update_informer::{registry, Check};

pub fn run() {
    let pkg_name = env!("CARGO_PKG_NAME");
    let current_version = env!("CARGO_PKG_VERSION");

    let informer =
        update_informer::new(registry::GitHub, "Araxeus/ls-interactive", current_version);

    if let Ok(Some(version)) = informer.check_version() {
        let msg = format!(
            "A new release of {pkg_name} is available: v{current_version} -> {new_version}",
            pkg_name = style(pkg_name).italic().cyan(),
            current_version = current_version,
            new_version = style(version.to_string()).green()
        );

        let release_url = format!(
            "https://github.com/{pkg_name}/{pkg_name}/releases/tag/{version}",
            pkg_name = pkg_name,
            version = version
        );

        println!(
            "\n{msg}\n{url}",
            msg = msg,
            url = style(release_url).yellow()
        );
    }
}
