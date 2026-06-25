use crate::error::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

const HISTORY_FILE: &str = ".cc_history";
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn save_command(command: &str, current_dir: &PathBuf) -> Result<()> {
    let home_dir = std::env::var("HOME")?;
    let history_path = PathBuf::from(home_dir).join(HISTORY_FILE);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path)?;

    let timestamp = chrono::Local::now().format(TIMESTAMP_FORMAT);
    let log_line = format!("[{}]({}) {}\n", timestamp, current_dir.display(), command);
    file.write_all(log_line.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_constants() {
        assert_eq!(HISTORY_FILE, ".cc_history");
        assert_eq!(TIMESTAMP_FORMAT, "%Y-%m-%d %H:%M:%S");
    }
}
