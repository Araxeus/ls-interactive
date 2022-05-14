use notify_rust::Notification;

pub fn run() {
    std::thread::spawn(|| {
        check_version();
    });
}

pub fn check_version() {
    let pkg_name = env!("CARGO_PKG_NAME");
    let current_version = env!("CARGO_PKG_VERSION");

    if let Some(new_version) = get_latest_version() {
        if new_version != current_version {
            Notification::new()
                .summary(pkg_name)
                .body(&format!(
                    "A new release of {pkg_name} is available: \n\
    v{current_version} -> v{new_version}\n\
    https://github.com/Araxeus/{pkg_name}/releases/tag/{new_version}",
                    pkg_name = pkg_name,
                    current_version = current_version,
                    new_version = new_version
                ))
                .icon("/usr/share/icons/hicolor/256x256/apps/gnome-software.png")
                .timeout(5000)
                .show()
                .ok();
        }
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
            Some(
                stdout
                    .split("\"tag_name\": \"")
                    .nth(1)
                    .unwrap()
                    .split('\"')
                    .next()
                    .unwrap()
                    .trim_start_matches('v')
                    .to_string(),
            )
        }
        Err(_) => None,
    }
}
