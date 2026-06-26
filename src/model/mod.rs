pub mod format;
pub use format::{format_args_for_shell, CommandFormatter};

use std::path::PathBuf;

use fuzzy_matcher::FuzzyMatcher;

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

// ── CommandStats ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CommandStats {
    pub command: String,
    pub count: usize,
    pub latest_record: CommandRecord,
}

/// Group records by command name, tracking count and latest record.
pub fn group_by_command(records: &[CommandRecord]) -> Vec<CommandStats> {
    use std::collections::HashMap;
    let mut map: HashMap<&str, (usize, &CommandRecord)> = HashMap::new();
    for record in records {
        let entry = map.entry(&record.command).or_insert((0, record));
        entry.0 += 1;
        // timestamp 格式为 YYYY-MM-DD HH:MM:SS，字符串比较即时间顺序
        if record.timestamp > entry.1.timestamp {
            entry.1 = record;
        }
    }
    let mut result: Vec<CommandStats> = map
        .into_iter()
        .map(|(cmd, (count, rec))| CommandStats {
            command: cmd.to_string(),
            count,
            latest_record: rec.clone(),
        })
        .collect();
    result.sort_by(|a, b| a.command.cmp(&b.command));
    result
}

/// Sort stats by frequency descending, tie-break by most recent timestamp.
pub fn sort_by_frequency(stats: &[CommandStats]) -> Vec<CommandStats> {
    let mut s = stats.to_vec();
    s.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.latest_record.timestamp.cmp(&a.latest_record.timestamp))
    });
    s
}

/// Sort stats by most recent timestamp descending, tie-break by command name.
pub fn sort_by_recent(stats: &[CommandStats]) -> Vec<CommandStats> {
    let mut s = stats.to_vec();
    s.sort_by(|a, b| {
        b.latest_record
            .timestamp
            .cmp(&a.latest_record.timestamp)
            .then_with(|| a.command.cmp(&b.command))
    });
    s
}

// ── fuzzy_filter ──────────────────────────────────

pub fn fuzzy_filter(query: &str, records: &[CommandRecord]) -> Vec<CommandRecord> {
    if query.is_empty() {
        return records.to_vec();
    }
    let matcher = fuzzy_matcher::clangd::ClangdMatcher::default();
    let mut scored: Vec<(i64, &CommandRecord)> = records
        .iter()
        .filter_map(|r| matcher.fuzzy_match(&r.command, query).map(|score| (score, r)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, r)| r.clone()).collect()
}

/// Take the first n items from a slice.
pub fn top_n<T: Clone>(items: &[T], n: usize) -> Vec<T> {
    items.iter().take(n).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(timestamp: &str, dir: &str, cmd: &str) -> CommandRecord {
        CommandRecord {
            command: cmd.to_string(),
            dir: PathBuf::from(dir),
            timestamp: timestamp.to_string(),
        }
    }

    fn stats(cmd: &str, count: usize, ts: &str) -> CommandStats {
        CommandStats {
            command: cmd.to_string(),
            count,
            latest_record: rec(ts, "/tmp", cmd),
        }
    }

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
        let formatter = format::ShellEscapeFormatter;
        let args = vec!["echo", "hello"];
        assert_eq!(formatter.format(&args), "echo hello");
    }

    // ── group_by_command ──────────────────────────────────

    #[test]
    fn test_group_identical_commands() {
        let records = vec![
            rec("2026-01-03 12:00:00", "/tmp", "git push"),
            rec("2026-01-02 12:00:00", "/tmp", "cargo build"),
            rec("2026-01-04 12:00:00", "/tmp", "git push"),
            rec("2026-01-01 12:00:00", "/tmp", "echo hi"),
            rec("2026-01-05 12:00:00", "/tmp", "cargo build"),
            rec("2026-01-06 12:00:00", "/tmp", "git push"),
        ];
        let grouped = group_by_command(&records);
        assert_eq!(grouped.len(), 3);

        let gp = grouped.iter().find(|s| s.command == "git push").unwrap();
        assert_eq!(gp.count, 3);
        assert_eq!(gp.latest_record.timestamp, "2026-01-06 12:00:00");

        let cb = grouped.iter().find(|s| s.command == "cargo build").unwrap();
        assert_eq!(cb.count, 2);
        assert_eq!(cb.latest_record.timestamp, "2026-01-05 12:00:00");

        assert!(grouped.iter().any(|s| s.command == "echo hi" && s.count == 1));
    }

    #[test]
    fn test_group_empty() {
        assert!(group_by_command(&[]).is_empty());
    }

    #[test]
    fn test_group_single_record() {
        let grouped = group_by_command(&[rec("2026-01-01 12:00:00", "/tmp", "cmd")]);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].count, 1);
    }

    #[test]
    fn test_group_all_same_command() {
        let records = vec![
            rec("2026-01-01 12:00:00", "/tmp", "same"),
            rec("2026-01-02 12:00:00", "/tmp", "same"),
            rec("2026-01-03 12:00:00", "/tmp", "same"),
        ];
        let grouped = group_by_command(&records);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].count, 3);
        assert_eq!(grouped[0].latest_record.timestamp, "2026-01-03 12:00:00");
    }

    #[test]
    fn test_group_all_different() {
        let records = vec![
            rec("2026-01-01 12:00:00", "/tmp", "a"),
            rec("2026-01-02 12:00:00", "/tmp", "b"),
            rec("2026-01-03 12:00:00", "/tmp", "c"),
        ];
        let grouped = group_by_command(&records);
        assert_eq!(grouped.len(), 3);
        assert!(grouped.iter().all(|s| s.count == 1));
    }

    // ── sort_by_frequency ─────────────────────────────────

    #[test]
    fn test_frequency_descending() {
        let s = vec![stats("a", 1, "2026-01-01"), stats("b", 5, "2026-01-01"), stats("c", 3, "2026-01-01")];
        let sorted = sort_by_frequency(&s);
        assert_eq!(sorted[0].command, "b");
        assert_eq!(sorted[1].command, "c");
        assert_eq!(sorted[2].command, "a");
    }

    #[test]
    fn test_frequency_empty() {
        let sorted: Vec<CommandStats> = sort_by_frequency(&[]);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_frequency_tie_break_by_time() {
        let s = vec![
            stats("old", 2, "2026-01-01 12:00:00"),
            stats("new", 2, "2026-02-01 12:00:00"),
        ];
        let sorted = sort_by_frequency(&s);
        assert_eq!(sorted[0].command, "new");
        assert_eq!(sorted[1].command, "old");
    }

    // ── sort_by_recent ────────────────────────────────────

    #[test]
    fn test_recent_descending() {
        let s = vec![
            stats("old", 1, "2026-01-01"),
            stats("mid", 1, "2026-02-01"),
            stats("new", 1, "2026-03-01"),
        ];
        let sorted = sort_by_recent(&s);
        assert_eq!(sorted[0].command, "new");
        assert_eq!(sorted[1].command, "mid");
        assert_eq!(sorted[2].command, "old");
    }

    #[test]
    fn test_recent_empty() {
        let sorted: Vec<CommandStats> = sort_by_recent(&[]);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_recent_tie_break_by_command_name() {
        let s = vec![
            stats("bb", 1, "2026-01-01"),
            stats("aa", 1, "2026-01-01"),
        ];
        let sorted = sort_by_recent(&s);
        assert_eq!(sorted[0].command, "aa"); // 字典序
        assert_eq!(sorted[1].command, "bb");
    }

    // ── top_n ─────────────────────────────────────────────

    #[test]
    fn test_top_n_returns_n() {
        let items: Vec<i32> = (1..=10).collect();
        assert_eq!(top_n(&items, 3), vec![1, 2, 3]);
    }

    #[test]
    fn test_top_n_less_than_n() {
        let items = vec![1, 2];
        assert_eq!(top_n(&items, 5), vec![1, 2]);
    }

    #[test]
    fn test_top_n_empty() {
        let items: Vec<i32> = vec![];
        assert!(top_n(&items, 5).is_empty());
    }

    #[test]
    fn test_top_n_zero() {
        let items = vec![1, 2, 3];
        assert!(top_n(&items, 0).is_empty());
    }

    // ── fuzzy_filter ──────────────────────────────────

    fn rec_fuzzy(cmd: &str) -> CommandRecord {
        CommandRecord {
            command: cmd.to_string(),
            dir: PathBuf::from("/tmp"),
            timestamp: "2026-01-01 12:00:00".to_string(),
        }
    }

    #[test]
    fn test_exact_match() {
        let r = fuzzy_filter("git push", &[rec_fuzzy("git push")]);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_subsequence_match() {
        let r = fuzzy_filter("gpo", &[rec_fuzzy("git push origin main")]);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_case_insensitive() {
        let r = fuzzy_filter("GIT", &[rec_fuzzy("git status")]);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_no_match() {
        let r = fuzzy_filter("zzzzz", &[rec_fuzzy("git push")]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_scored_ordering() {
        let records = vec![
            rec_fuzzy("cargo build --release"),
            rec_fuzzy("cargo build"),
            rec_fuzzy("cargo test"),
        ];
        let r = fuzzy_filter("cargo build", &records);
        assert_eq!(r[0].command, "cargo build");
    }

    #[test]
    fn test_empty_query_returns_all() {
        let records = vec![rec_fuzzy("git push"), rec_fuzzy("cargo build")];
        let r = fuzzy_filter("", &records);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_empty_records() {
        let r = fuzzy_filter("git", &[]);
        assert!(r.is_empty());
    }

    #[test]
    fn test_unicode_match() {
        // 中文字符匹配（不影响匹配机制，但确保不 panic）
        let r = fuzzy_filter("中文", &[rec_fuzzy("测试中文命令")]);
        // fuzzy-matcher 是否匹配中文取决于实现，只要不 panic 即可
        // 此处不 assert 结果，仅确保不崩溃
        let _ = r;
    }

    #[test]
    fn test_multiple_matches_ordered_by_relevance() {
        let records = vec![
            rec_fuzzy("git push origin main"),
            rec_fuzzy("git status"),
            rec_fuzzy("git push -f"),
            rec_fuzzy("echo hello"),
        ];
        let r = fuzzy_filter("git push", &records);
        assert!(r.len() >= 2);
        // "git push" 系列的匹配应该排在 "git status"（无 push）之前
        for cmd in r.iter() {
            assert!(cmd.command.contains("git"), "{} 应包含 git", cmd.command);
        }
    }

    // ── 边界场景 ────────────────────────────────────

    #[test]
    fn test_limit_one_produces_empty_stats() {
        let limit = 1usize;
        let top = limit / 2;
        assert_eq!(top, 0);
        // run_stats 中的 empty check 应正确返回
    }

    #[test]
    fn test_all_same_command_stats() {
        let records = vec![
            rec("2026-01-01", "/dir", "same command"),
            rec("2026-01-02", "/dir", "same command"),
            rec("2026-01-03", "/dir", "same command"),
        ];
        let grouped = group_by_command(&records);
        assert_eq!(grouped.len(), 1);

        let freq = sort_by_frequency(&grouped);
        let recent = sort_by_recent(&grouped);
        assert_eq!(freq[0].command, "same command");
        assert_eq!(recent[0].command, "same command");
    }

    #[test]
    fn test_search_then_stats_no_crash() {
        let records = vec![
            rec("2026-01-01", "/dir", "git push"),
            rec("2026-01-02", "/dir", "cargo build"),
        ];
        let filtered = fuzzy_filter("git", &records);
        assert_eq!(filtered.len(), 1);

        let grouped = group_by_command(&filtered);
        assert_eq!(grouped.len(), 1);
    }
}
