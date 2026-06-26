pub mod format;
pub use format::{format_args_for_shell, CommandFormatter, ShellEscapeFormatter};

use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CommandRecord {
    pub command: String,
    pub dir: PathBuf,
    pub timestamp: String,
}

impl CommandRecord {
    pub fn new(command: String, dir: PathBuf) -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            command,
            dir,
            timestamp,
        }
    }

    #[allow(dead_code)]
    pub fn line(&self) -> String {
        format!(
            "[{}]({}) {}\n",
            self.timestamp,
            self.dir.display(),
            self.command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_record_line_contains_command() {
        let record = CommandRecord::new("echo hello".to_string(), PathBuf::from("/tmp"));
        let line = record.line();
        assert!(line.contains("echo hello"), "行中应包含命令内容");
    }

    #[test]
    fn test_command_record_line_format() {
        let record = CommandRecord::new("ls".to_string(), PathBuf::from("/tmp"));
        let line = record.line();
        assert!(line.starts_with('['), "应以 [ 开头");
        assert!(line.contains("]("), "应包含 ](");
        assert!(line.ends_with('\n'), "应以换行结尾");
    }

    #[test]
    fn test_shell_escape_formatter_basic() {
        let formatter = ShellEscapeFormatter;
        let args = vec!["echo", "hello"];
        assert_eq!(formatter.format(&args), "echo hello");
    }
}
