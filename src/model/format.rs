use shell_escape::escape;

pub trait CommandFormatter {
    fn format(&self, args: &[&str]) -> String;
}

pub struct SimpleFormatter;

impl CommandFormatter for SimpleFormatter {
    fn format(&self, args: &[&str]) -> String {
        args.join(" ")
    }
}

pub struct ShellEscapeFormatter;

impl CommandFormatter for ShellEscapeFormatter {
    fn format(&self, args: &[&str]) -> String {
        args.iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, arg)| {
                // No space before standalone semicolons: "cmd; next" not "cmd ; next"
                if i > 0 && *arg != ";" {
                    acc.push(' ');
                }
                if arg.is_empty() {
                    acc.push_str("''");
                } else {
                    let escaped = escape(std::borrow::Cow::Borrowed(*arg)).into_owned();
                    // If the argument consists solely of shell metacharacters,
                    // pass it through unescaped so operators like &&, |, >, ;
                    // remain functional in the constructed command.
                    if arg.chars().all(|c| "&|;<>".contains(c)) && escaped != *arg {
                        acc.push_str(arg);
                    } else {
                        acc.push_str(&escaped);
                    }
                }
                acc
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ShellEscapeFormatter 测试 ───────────────────────────────────

    #[test]
    fn test_space_from_backslash_escape() {
        let f = ShellEscapeFormatter;
        let args = vec!["ls", "-lah", "/Users/zhangchao/2026/rust/conmand_cache/context/test dir"];
        assert_eq!(f.format(&args), "ls -lah '/Users/zhangchao/2026/rust/conmand_cache/context/test dir'");
    }

    #[test]
    fn test_space_from_quotes() {
        let f = ShellEscapeFormatter;
        let args = vec!["cat", "/path/my documents/file.txt"];
        assert_eq!(f.format(&args), "cat '/path/my documents/file.txt'");
    }

    #[test]
    fn test_dollar_sign() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "$HOME/file.txt"];
        assert_eq!(f.format(&args), "echo '$HOME/file.txt'");
    }

    #[test]
    fn test_backtick() {
        let f = ShellEscapeFormatter;
        let args = vec!["cat", "file`date`.txt"];
        assert_eq!(f.format(&args), "cat 'file`date`.txt'");
    }

    #[test]
    fn test_single_quote() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "it's working"];
        assert_eq!(f.format(&args), "echo 'it'\\''s working'");
    }

    // ── Shell 特性保留（spec 第 6-9 条） ────────────────────────────

    #[test]
    fn test_pipe_preserved() {
        let f = ShellEscapeFormatter;
        let args = vec!["cat", "file.txt", "|", "grep", "pattern"];
        assert_eq!(f.format(&args), "cat file.txt | grep pattern");
    }

    #[test]
    fn test_chain_and() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "hello", "&&", "echo", "world"];
        assert_eq!(f.format(&args), "echo hello && echo world");
    }

    #[test]
    fn test_redirect() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "data", ">", "/tmp/output.txt"];
        assert_eq!(f.format(&args), "echo data > /tmp/output.txt");
    }

    #[test]
    fn test_semicolon_chain() {
        let f = ShellEscapeFormatter;
        let args = vec!["cd", "/tmp", "&&", "pwd", ";", "echo", "done"];
        assert_eq!(f.format(&args), "cd /tmp && pwd; echo done");
    }

    // ── 边界情况（spec 第 10-16 条） ────────────────────────────────

    #[test]
    fn test_empty_arg() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", ""];
        assert_eq!(f.format(&args), "echo ''");
    }

    #[test]
    fn test_only_spaces() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "   "];
        assert_eq!(f.format(&args), "echo '   '");
    }

    #[test]
    fn test_only_safe_chars() {
        let f = ShellEscapeFormatter;
        let args = vec!["ls", "-la", "/usr/local/bin"];
        assert_eq!(f.format(&args), "ls -la /usr/local/bin");
    }

    #[test]
    fn test_multiple_special_chars() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "hello $USER `whoami` ; rm -rf /"];
        assert_eq!(f.format(&args), "echo 'hello $USER `whoami` ; rm -rf /'");
    }

    #[test]
    fn test_only_dollar_sign() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "$"];
        assert_eq!(f.format(&args), "echo '$'");
    }

    #[test]
    fn test_unicode_emoji() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "路径/日本語/🎉"];
        assert_eq!(f.format(&args), "echo '路径/日本語/🎉'");
    }

    #[test]
    fn test_long_safe_string() {
        let f = ShellEscapeFormatter;
        let args = vec!["echo", "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"];
        assert_eq!(f.format(&args), "echo abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
    }

    // ── SimpleFormatter 原有测试 ────────────────────────────────────

    #[test]
    fn test_simple_formatter_basic() {
        let formatter = SimpleFormatter;
        let args = vec!["echo", "hello"];
        assert_eq!(formatter.format(&args), "echo hello");
    }

    #[test]
    fn test_simple_formatter_empty() {
        let formatter = SimpleFormatter;
        assert_eq!(formatter.format(&[]), "");
    }
}
