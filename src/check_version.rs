use console::style;

pub fn run() {
    std::thread::spawn(|| {
        check_version();
    });
}

pub fn check_version() {
    let pkg_name = env!("CARGO_PKG_NAME");
    let current_version = env!("CARGO_PKG_VERSION");

    if let Some(version) = get_latest_version() {
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


pub fn get_latest_version() -> Option<String> {
        let output = std::process::Command::new("curl")
        .arg("--silent")
        .arg("https://api.github.com/repos/Araxeus/ls-interactive/releases/latest")
        .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Some(stdout.split("\"tag_name\": \"").nth(1).unwrap().split('\"').next().unwrap().trim_start_matches('v').to_string())
            },
            Err(_) => None,
        }
}
