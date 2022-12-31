use notify_rust::Notification;

use directories::ProjectDirs;
use std::{
    fs,
    io::{self, Error, ErrorKind},
    time::Duration,
};

pub fn run() {
    std::thread::spawn(|| {
        // if should_check_update is bool true - check version, if it's an error - show notification
        match should_check_update() {
            Err(e) => notification(&format!("Error: should_check_update() Failed: \n{e}")),
            Ok(true) => check_version(),
            Ok(false) => (),
        }
    });
}

fn check_version() {
    let current_version = env!("CARGO_PKG_VERSION");

    if let Ok(new_version) = get_latest_version() {
        if new_version != current_version {
            notification(&format!(
                "A new release of {pkg_name} is available: \n\
    v{current_version} -> v{new_version}\n\
    {repo_url}/releases/tag/{new_version}",
                pkg_name = env!("CARGO_PKG_NAME"),
                repo_url = env!("CARGO_PKG_REPOSITORY")
            ));
        }
    }
}

fn notification(body: &str) {
    Notification::new()
        .summary(env!("CARGO_PKG_NAME"))
        .body(body)
        .icon("/usr/share/icons/hicolor/256x256/apps/gnome-software.png")
        .timeout(5000)
        .show()
        .ok();
}

fn get_latest_version() -> Result<String, Error> {
    let repo_url = env!("CARGO_PKG_REPOSITORY");
    let data = repo_url.split('/').collect::<Vec<&str>>();
    let owner = data[3];
    let repo = data[4];

    let output = std::process::Command::new("curl")
        .arg("--silent")
        .arg(format!(
            "https://api.github.com/repos/{owner}/{repo}/releases/latest"
        ))
        .output();

    write_last_checked().unwrap_or_else(|e| {
        notification(&format!("Error: write_last_checked() failed: \n{e}"));
    });

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout
                .split("\"tag_name\": \"")
                .nth(1)
                .unwrap()
                .split('\"')
                .next()
                .unwrap()
                .trim_start_matches('v')
                .to_string())
        }
        Err(e) => {
            notification(&format!("Error: get_latest_version() failed: \n{e}"));
            Err(e)
        }
    }
}

fn should_check_update() -> io::Result<bool> {
    let binding = get_cache_dir()?;
    let cache_dir = binding.cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?;
    }
    let path = cache_dir.join(format!("{}-last-update-check", env!("CARGO_PKG_NAME")));
    if path.exists() {
        let metadata = fs::metadata(path)?;
        let last_modified_diff = metadata.modified()?.elapsed().unwrap_or_default();
        Ok(last_modified_diff > Duration::from_secs(60 * 60 * 24)) // 1 day
    } else {
        Ok(true)
    }
}

fn write_last_checked() -> io::Result<()> {
    let path = get_cache_dir()?
        .cache_dir()
        .join(format!("{}-last-update-check", env!("CARGO_PKG_NAME")));
    fs::write(path, "")
}

fn get_cache_dir() -> io::Result<ProjectDirs> {
    let project_dir = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"));
    project_dir.ok_or_else(|| io::Error::new(ErrorKind::Other, "Could not get project directory"))
}
