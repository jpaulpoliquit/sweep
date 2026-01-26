//! Lightweight debug logging to a local text file.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn log_dir() -> Option<PathBuf> {
    let base_dir = if cfg!(windows) {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                std::env::var("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Local"))
                    .ok()
            })
    } else {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".local").join("share"))
            .ok()
    }?;

    Some(base_dir.join("wole").join("logs"))
}

fn log_path(file_name: &str) -> Option<PathBuf> {
    let dir = log_dir()?;
    let _ = std::fs::create_dir_all(&dir);
    Some(dir.join(file_name))
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn append_line(file_name: &str, message: &str) {
    let Some(path) = log_path(file_name) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "[{}] {}", timestamp(), message);
}

pub fn cleaning_log(message: &str) {
    append_line("cleaning.log", message);
}
