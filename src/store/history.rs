use crate::error::Result;
use crate::model::CommandRecord;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const HISTORY_FILE: &str = ".cc_history";
const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn save_command(command: &str, current_dir: &PathBuf) -> Result<()> {
    let history_path = get_history_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path)?;

    let timestamp = chrono::Local::now().format(TIMESTAMP_FORMAT);
    let log_line = format!("[{}]({}) {}\n", timestamp, current_dir.display(), command);
    file.write_all(log_line.as_bytes())?;
    Ok(())
}

// ── parse_line ─────────────────────────────────────────────────────

/// Parse a single history line into a CommandRecord.
/// Format: [2026-06-26 12:34:56](/path/to/dir) command text
fn parse_line(line: &str) -> Option<CommandRecord> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let close_bracket = line.find(']')?;
    let timestamp = line[1..close_bracket].to_string();

    let rest = &line[close_bracket + 1..];
    if !rest.starts_with('(') {
        return None;
    }
    let close_paren = rest.find(')')?;
    let dir = rest[1..close_paren].to_string();

    let command = rest.get(close_paren + 2..).unwrap_or("").to_string(); // skip ") "

    Some(CommandRecord {
        command,
        dir: PathBuf::from(dir),
        timestamp,
    })
}

// ── get_history_path ───────────────────────────────────────────────

fn get_history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(HISTORY_FILE)
}

// ── load_commands / load_from_history ──────────────────────────────

/// Load history records matching current_dir, newest first, max 10.
pub fn load_commands(current_dir: &PathBuf) -> Result<Vec<CommandRecord>> {
    load_from_history(&get_history_path(), current_dir)
}

fn load_from_history(path: &Path, current_dir: &Path) -> Result<Vec<CommandRecord>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(path)?;
    let mut records: Vec<CommandRecord> = content
        .lines()
        .filter_map(parse_line)
        .filter(|r| r.dir == current_dir)
        .collect();

    records.reverse(); // newest first (file is chronological)
    records.truncate(10);
    Ok(records)
}

// ── clear_commands / clear_from_history ────────────────────────────

/// Remove all history entries for current_dir, keep others intact.
pub fn clear_commands(current_dir: &PathBuf) -> Result<()> {
    clear_from_history(&get_history_path(), current_dir)
}

fn clear_from_history(path: &Path, current_dir: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            parse_line(line)
                .map(|r| r.dir != current_dir)
                .unwrap_or(true) // keep malformed lines
        })
        .collect();

    let output = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };
    std::fs::write(path, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_history_constants() {
        assert_eq!(HISTORY_FILE, ".cc_history");
        assert_eq!(TIMESTAMP_FORMAT, "%Y-%m-%d %H:%M:%S");
    }

    // ── parse_line ─────────────────────────────────────────────

    #[test]
    fn test_parse_line_valid() {
        let line = "[2026-06-26 12:34:56](/tmp) echo hello";
        let record = parse_line(line).unwrap();
        assert_eq!(record.timestamp, "2026-06-26 12:34:56");
        assert_eq!(record.dir, PathBuf::from("/tmp"));
        assert_eq!(record.command, "echo hello");
    }

    #[test]
    fn test_parse_line_valid_with_long_path() {
        let line = "[2026-01-01 00:00:00](/home/user/projects/my project) ls -la";
        let record = parse_line(line).unwrap();
        assert_eq!(record.command, "ls -la");
        assert_eq!(record.dir, PathBuf::from("/home/user/projects/my project"));
    }

    #[test]
    fn test_parse_line_valid_empty_command() {
        let line = "[2026-06-26 12:00:00](/tmp) ";
        let record = parse_line(line).unwrap();
        assert_eq!(record.command, "");
    }

    #[test]
    fn test_parse_line_missing_bracket() {
        assert!(parse_line("2026-06-26 12:00:00](/tmp) cmd").is_none());
    }

    #[test]
    fn test_parse_line_missing_paren() {
        assert!(parse_line("[2026-06-26 12:00:00]/tmp) cmd").is_none());
    }

    #[test]
    fn test_parse_line_empty_string() {
        assert!(parse_line("").is_none());
    }

    #[test]
    fn test_parse_line_malformed_prefix() {
        assert!(parse_line("just a normal line").is_none());
    }

    // ── load_from_history ──────────────────────────────────────

    #[test]
    fn test_load_from_history_empty_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_empty_history");
        std::fs::write(&path, "").unwrap();
        let records = load_from_history(&path, &dir).unwrap();
        assert!(records.is_empty());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_from_history_nonexistent_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_nonexistent_history");
        std::fs::remove_file(&path).ok(); // ensure it doesn't exist
        let records = load_from_history(&path, &dir).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_load_from_history_filters_by_dir() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_filter_history");
        let content = "\
[2026-01-01 12:00:00](/other) echo other
[2026-01-02 12:00:00](__TEST_DIR__) echo match
[2026-01-03 12:00:00](/other) ls -la
[2026-01-04 12:00:00](__TEST_DIR__) pwd";
        let content = content.replace("__TEST_DIR__", dir.to_str().unwrap());
        std::fs::write(&path, &content).unwrap();
        let records = load_from_history(&path, &dir).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].command, "pwd");   // newest first
        assert_eq!(records[1].command, "echo match");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_from_history_limits_to_10() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_limit_history");
        let mut content = String::new();
        for i in 0..15 {
            content.push_str(&format!(
                "[2026-01-{:02} 12:00:00]({}) cmd {}\n",
                i + 1,
                dir.display(),
                i
            ));
        }
        std::fs::write(&path, &content).unwrap();
        let records = load_from_history(&path, &dir).unwrap();
        assert_eq!(records.len(), 10);
        assert_eq!(records[0].command, "cmd 14"); // newest first
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_from_history_preserves_escape_chars() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_escape_history");
        let content = format!(
            "[2026-01-01 12:00:00]({}) echo '$HOME' `whoami`\n",
            dir.display()
        );
        std::fs::write(&path, &content).unwrap();
        let records = load_from_history(&path, &dir).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].command, "echo '$HOME' `whoami`");
        std::fs::remove_file(&path).ok();
    }

    // ── clear_from_history ─────────────────────────────────────

    #[test]
    fn test_clear_from_history_removes_matching_dir() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_clear_history");
        let content = "\
[2026-01-01 12:00:00](/other) keep
[2026-01-02 12:00:00](__TEST_DIR__) remove
[2026-01-03 12:00:00](/other) keep too";
        let content = content.replace("__TEST_DIR__", dir.to_str().unwrap());
        std::fs::write(&path, &content).unwrap();

        clear_from_history(&path, &dir).unwrap();

        let remaining = std::fs::read_to_string(&path).unwrap();
        assert!(!remaining.contains("remove"));   // removed
        assert!(remaining.contains("keep"));       // preserved
        assert!(remaining.contains("keep too"));   // preserved
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_clear_from_history_all_removed_file_empty() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_clear_all_history");
        let content = format!(
            "[2026-01-01 12:00:00]({}) only entry\n",
            dir.display()
        );
        std::fs::write(&path, &content).unwrap();

        clear_from_history(&path, &dir).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "");                  // file truncated
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_clear_from_history_nonexistent_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("_test_clear_nonexistent");
        std::fs::remove_file(&path).ok();
        // Should not panic or error
        assert!(clear_from_history(&path, &dir).is_ok());
    }
}
