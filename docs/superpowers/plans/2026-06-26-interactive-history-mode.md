# 交互式历史记录选择模式 — 实现方案

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `cc` CLI 增加交互式历史记录选择、快速选择、flag 路由功能

**Architecture:** 新增 `cli/parser` 模块统一入口模式解析，新增 `interactive` 模块处理交互菜单，`store/history` 追加 `load_commands` 和 `clear_commands`，`main.rs` 收窄为薄分发层

**Tech Stack:** Rust, shell-escape 0.1, chrono

## Global Constraints

- 历史文件路径 `~/.cc_history`（`HOME` 环境变量决定）
- 历史记录格式 `[timestamp](dir) command`
- 交互菜单最多显示最近 10 条
- 仅保留现有依赖，不新增 crate
- 纯数字判定用 `str::parse::<u32>()`，解析成功即为纯数字
- 所有 debug 输出使用 `eprintln!` 写到 stderr
- ANSI 前缀常量：`CCI` = `"\x1b[36mcc |\x1b[0m "`, `CCOK` = `"\x1b[32mcc |\x1b[0m "`, `CCERR` = `"\x1b[31mcc |\x1b[0m "`

---

### Task 1: Store 层 — 历史记录加载与清除

**Files:**
- Modify: `src/store/history.rs`

**Produced interfaces:**
- `pub fn load_commands(current_dir: &PathBuf) -> Result<Vec<CommandRecord>>`
- `pub fn clear_commands(current_dir: &PathBuf) -> Result<()>`
- Internal `fn parse_line(line: &str) -> Option<CommandRecord>`
- Internal `fn get_history_path() -> PathBuf`
- Internal `fn load_from_history(path: &Path, current_dir: &Path) -> Result<Vec<CommandRecord>>`
- Internal `fn clear_from_history(path: &Path, current_dir: &Path) -> Result<()>`

- [ ] **Step 1: 添加 parse_line 和 get_history_path**

```rust
// 在 src/store/history.rs 中，save_command 之后追加

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

    let command = rest[close_paren + 2..].to_string(); // skip ") "

    Some(CommandRecord {
        command,
        dir: PathBuf::from(dir),
        timestamp,
    })
}

fn get_history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(HISTORY_FILE)
}
```

- [ ] **Step 2: 添加 load_commands**

```rust
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
```

- [ ] **Step 3: 添加 clear_commands**

```rust
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
```

- [ ] **Step 4: 写 parse_line 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
```

- [ ] **Step 5: 写 load_from_history 测试**

```rust
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
```

- [ ] **Step 6: 写 clear_from_history 测试**

```rust
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
```

- [ ] **Step 7: 编译确认，运行测试**

```bash
cargo test --lib store::history::tests 2>&1 | tail -30
# 预期: 所有 history 测试通过
```

- [ ] **Step 8: 提交**

```bash
git add src/store/history.rs
git commit -m "feat: add load_commands and clear_commands to history store"
```

---

### Task 2: CLI 参数解析器 — CliMode 枚举 + parse_args

**Files:**
- Create: `src/cli/mod.rs`
- Create: `src/cli/parser.rs`

**Produced interfaces:**
- `pub enum CliMode { Interactive, QuickSelect(u32), Flag{show_help,clear_history}, Command(Vec<String>) }`
- `pub fn parse_args(args: &[String]) -> CliMode`

- [ ] **Step 1: 创建模块入口**

```rust
// src/cli/mod.rs
pub mod parser;
```

- [ ] **Step 2: 写 CliMode 枚举和 parse_args**

```rust
// src/cli/parser.rs
/// Describes the entry mode for the `cc` tool.
#[derive(Debug, PartialEq, Eq)]
pub enum CliMode {
    /// No arguments: enter interactive history selection.
    Interactive,
    /// A pure-number argument: quick-select history entry by 1-based index.
    QuickSelect(u32),
    /// A `-h`/`--help` or `-d`/`--clear` flag.
    Flag {
        show_help: bool,
        clear_history: bool,
    },
    /// Anything else: treat as a command to record and execute.
    Command(Vec<String>),
}

/// Classify CLI arguments into a [CliMode].
///
/// Order of precedence:
/// 1. Empty args → Interactive
/// 2. First arg starts with `-` → Flag (known flags only, else Command)
/// 3. First arg is a pure u32 → QuickSelect
/// 4. Everything else → Command
pub fn parse_args(args: &[String]) -> CliMode {
    if args.is_empty() {
        return CliMode::Interactive;
    }

    let first = &args[0];

    // Flags
    if first == "-h" || first == "--help" {
        return CliMode::Flag {
            show_help: true,
            clear_history: false,
        };
    }
    if first == "-d" || first == "--clear" {
        return CliMode::Flag {
            show_help: false,
            clear_history: true,
        };
    }

    // Quick select — pure number
    if let Ok(num) = first.parse::<u32>() {
        return CliMode::QuickSelect(num);
    }

    // Command mode — pass through all args
    CliMode::Command(args.to_vec())
}
```

- [ ] **Step 3: 写 parse_args 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // ── Interactive ────────────────────────────────────────────

    #[test]
    fn test_empty_args_is_interactive() {
        assert_eq!(parse_args(&s(&[])), CliMode::Interactive);
    }

    // ── QuickSelect ────────────────────────────────────────────

    #[test]
    fn test_single_digit_quick_select() {
        assert_eq!(parse_args(&s(&["5"])), CliMode::QuickSelect(5));
    }

    #[test]
    fn test_multi_digit_quick_select() {
        assert_eq!(parse_args(&s(&["23"])), CliMode::QuickSelect(23));
    }

    #[test]
    fn test_zero_is_quick_select() {
        assert_eq!(parse_args(&s(&["0"])), CliMode::QuickSelect(0));
    }

    // ── Flag ───────────────────────────────────────────────────

    #[test]
    fn test_flag_hyphen_h() {
        assert_eq!(
            parse_args(&s(&["-h"])),
            CliMode::Flag { show_help: true, clear_history: false }
        );
    }

    #[test]
    fn test_flag_double_hyphen_help() {
        assert_eq!(
            parse_args(&s(&["--help"])),
            CliMode::Flag { show_help: true, clear_history: false }
        );
    }

    #[test]
    fn test_flag_hyphen_d() {
        assert_eq!(
            parse_args(&s(&["-d"])),
            CliMode::Flag { show_help: false, clear_history: true }
        );
    }

    #[test]
    fn test_flag_double_hyphen_clear() {
        assert_eq!(
            parse_args(&s(&["--clear"])),
            CliMode::Flag { show_help: false, clear_history: true }
        );
    }

    // ── Command ────────────────────────────────────────────────

    #[test]
    fn test_simple_command() {
        assert_eq!(
            parse_args(&s(&["echo", "hello"])),
            CliMode::Command(s(&["echo", "hello"]))
        );
    }

    #[test]
    fn test_command_with_args() {
        assert_eq!(
            parse_args(&s(&["ls", "-la", "/tmp"])),
            CliMode::Command(s(&["ls", "-la", "/tmp"]))
        );
    }

    #[test]
    fn test_number_prefix_not_pure() {
        // "7z" is not a pure u32, falls to Command
        assert_eq!(
            parse_args(&s(&["7z", "x", "file.7z"])),
            CliMode::Command(s(&["7z", "x", "file.7z"]))
        );
    }

    #[test]
    fn test_dash_prefixed_not_flag() {
        // "-la" starts with `-` but is not a known flag → Command
        assert_eq!(
            parse_args(&s(&["-la"])),
            CliMode::Command(s(&["-la"]))
        );
    }
}
```

- [ ] **Step 4: 运行测试**

```bash
cargo test --lib cli::parser::tests 2>&1 | tail -30
# 预期: 所有 parse_args 测试通过
```

- [ ] **Step 5: 提交**

```bash
git add src/cli/
git commit -m "feat: add CliMode parser with comprehensive tests"
```

---

### Task 3: 交互模块

**Files:**
- Create: `src/interactive/mod.rs`

**Consumes:** `load_commands(dir)`, `run_command(cmd)`, `CommandRecord`
**Produces:** `pub fn run(records: &[CommandRecord]) -> Result<()>`

- [ ] **Step 1: 写交互菜单逻辑**

```rust
// src/interactive/mod.rs
use std::io::Write;
use crate::cmd::runner::run_command;
use crate::error::Result;
use crate::model::CommandRecord;

const CCI: &str = "\x1b[36mcc |\x1b[0m ";
const CCERR: &str = "\x1b[31mcc |\x1b[0m ";

/// Enter interactive mode: display a numbered menu, let user pick by number.
/// Returns Ok(()) if user quits (q) or after executing a selected command.
pub fn run(records: &[CommandRecord]) -> Result<()> {
    loop {
        eprintln!("{CCI}最近命令记录:");
        for (i, record) in records.iter().enumerate() {
            eprintln!("{CCI}  {}  {}", i + 1, record.command);
        }
        eprintln!("\x1b[2mcc | ------------------------\x1b[0m");
        eprint!("{CCI}输入编号执行(q 退出): ");
        std::io::stdout().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input == "q" {
            return Ok(());
        }

        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= records.len() {
                let cmd = &records[num - 1].command;
                eprintln!("{CCI}执行: {}", cmd);
                eprintln!("\x1b[2mcc | ------------------------\x1b[0m");
                run_command(cmd)?;
                return Ok(());
            }
        }

        eprintln!("{CCERR}无效输入，请输入编号或 q");
        // loop re-displays the menu
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add src/interactive/
git commit -m "feat: add interactive history selection module"
```

---

### Task 4: Main.rs 分发 + Flag 处理

**Files:**
- Modify: `src/main.rs`

**Consumes:** `parse_args()`, `CliMode`, `load_commands()`, `clear_commands()`, `interactive::run()`, `run_command()`, `save_command()`, `format_args_for_shell()`, `CommandRecord`

- [ ] **Step 1: 重构 main.rs**

```rust
mod cli;
mod cmd;
mod error;
mod interactive;
mod model;
mod store;

use cli::parser::{parse_args, CliMode};
use cmd::runner::run_command;
use model::{format_args_for_shell, CommandRecord};
use store::history::{clear_commands, load_commands, save_command};

const CCI: &str = "\x1b[36mcc |\x1b[0m ";
const CCOK: &str = "\x1b[32mcc |\x1b[0m ";
const CCERR: &str = "\x1b[31mcc |\x1b[0m ";

fn print_help() {
    eprintln!("{CCI}cc — 命令缓存与执行工具");
    eprintln!("{CCI}");
    eprintln!("{CCI}用法:");
    eprintln!("{CCI}  cc                   进入交互模式，选择执行历史命令");
    eprintln!("{CCI}  cc <数字>            快速执行指定编号的历史命令");
    eprintln!("{CCI}  cc <命令...>         记录并执行命令");
    eprintln!("{CCI}  cc -h, --help        显示此帮助信息");
    eprintln!("{CCI}  cc -d, --clear       清除当前目录的历史记录");
}

fn run() -> crate::error::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = parse_args(&args);
    let current_dir = std::env::current_dir()?;

    match mode {
        CliMode::Interactive => {
            let records = load_commands(&current_dir)?;
            if records.is_empty() {
                eprintln!("{CCI}当前目录暂无历史记录");
                return Ok(());
            }
            interactive::run(&records)
        }

        CliMode::QuickSelect(num) => {
            let records = load_commands(&current_dir)?;
            let idx = num as usize;
            if idx == 0 || idx > records.len() {
                eprintln!("{CCERR}编号 {} 超出范围（1-{}）", num, records.len());
                std::process::exit(1);
            }
            let record = &records[idx - 1];
            eprintln!("{CCI}当前路径: {}", current_dir.display());
            eprintln!("{CCI}执行: {}", record.command);
            eprintln!("\x1b[2mcc | ------------------------\x1b[0m");
            run_command(&record.command)?;
            eprintln!("\x1b[2mcc | ------------------------\x1b[0m");
            eprintln!("{CCOK}命令执行成功");
            Ok(())
        }

        CliMode::Flag { show_help, clear_history } => {
            if show_help {
                print_help();
            }
            if clear_history {
                eprint!("{CCI}确认清除当前目录历史记录？(y/N): ");
                std::io::stdout().flush().ok();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                if input.trim().eq_ignore_ascii_case("y") {
                    clear_commands(&current_dir)?;
                    eprintln!("{CCOK}已清除当前目录的历史记录");
                }
            }
            Ok(())
        }

        CliMode::Command(cmd_args) => {
            let command = format_args_for_shell(&cmd_args);
            let record = CommandRecord::new(command.clone(), current_dir.clone());

            eprintln!("{CCI}当前路径: {}", current_dir.display());
            eprintln!("{CCI}捕获的命令: {}", record.command);
            eprintln!("\x1b[2mcc | ------------------------\x1b[0m");
            run_command(&record.command)?;
            save_command(&record.command, &record.dir)?;
            eprintln!("\x1b[2mcc | ------------------------\x1b[0m");
            eprintln!("{CCOK}命令执行成功");
            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{CCERR}错误: {}", e);
        std::process::exit(1);
    }
}
```

- [ ] **Step 2: 编译确认所有测试通过**

```bash
cargo test 2>&1 | tail -30
# 预期: 已有的 27 测试 + 新增的 history/parser 测试全部通过
```

- [ ] **Step 3: 手动测试 — 交互模式**

```bash
# 先记录一些命令
cargo run -- echo hello
cargo run -- ls -la /tmp
cargo run -- "curl 'http://example.com' | jq .data"

# 测试交互模式（输入 1 选中第一条，验证执行）
echo "1" | cargo run
# 预期: 显示菜单 → 执行第一条命令 → 退出

# 测试交互模式 q 退出
echo "q" | cargo run
# 预期: 显示菜单 → 直接退出（不执行命令）
```

- [ ] **Step 4: 手动测试 — 快速选择**

```bash
# 测试有效编号
cargo run -- 1
# 预期: 执行第一条历史命令

# 测试无效编号 (0)
cargo run -- 0
# 预期: 错误 "编号 0 超出范围"

# 测试超界编号
cargo run -- 99
# 预期: 错误 "编号 99 超出范围"

# 测试非纯数字不被误判
cargo run -- 7z
# 预期: 作为命令执行（可能执行失败，但不应进入 QuickSelect）
```

- [ ] **Step 5: 手动测试 — Flags**

```bash
cargo run -- -h
# 预期: 显示帮助信息

cargo run -- --help
# 预期: 显示帮助信息

# 清除（先确认一条记录存在）
echo "y" | cargo run -- -d
# 预期: 确认提示 → 已清除

echo "n" | cargo run -- -d
# 预期: 确认提示 → 不执行清除
```

- [ ] **Step 6: 提交**

```bash
git add src/main.rs
git commit -m "feat: wire up CliMode dispatch in main.rs"
```

---

## 执行交付

方案已保存到 `docs/superpowers/plans/2026-06-26-interactive-history-mode.md`。两种执行方式：

**1. Subagent-Driven（推荐）** — 每个任务派发独立子代理执行，任务间 review
**2. Inline Execution** — 在当前会话中逐个任务执行

哪种方式？
