use std::io::Write;
use crate::cmd::runner::run_command;
use crate::error::Result;
use crate::model::CommandRecord;

const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";
const RST: &str = "\x1b[0m";

/// Interactive stats mode: display two zones (frequent + recent) with
/// continuous numbering. User picks a number to execute or `q` to quit.
pub fn run_stats(
    freq_records: &[CommandRecord],
    recent_records: &[CommandRecord],
) -> Result<()> {
    let total = freq_records.len() + recent_records.len();
    if total == 0 {
        eprintln!("{RED}没有匹配的历史记录{RST}");
        return Ok(());
    }

    // Build a flat index for number-to-record lookup
    let all: Vec<&CommandRecord> = {
        let mut v = Vec::with_capacity(total);
        v.extend(freq_records);
        v.extend(recent_records);
        v
    };

    loop {
        eprintln!("{DIM}━━━ 使用最频繁 ━━━{RST}");
        for (i, record) in freq_records.iter().enumerate() {
            eprintln!("{CYAN}【{}】{RST}{}", i + 1, record.command);
        }

        eprintln!();
        eprintln!("{DIM}━━━ 最近常用 ━━━{RST}");
        for (i, record) in recent_records.iter().enumerate() {
            eprintln!("{CYAN}【{}】{RST}{}", freq_records.len() + i + 1, record.command);
        }

        eprintln!("{DIM}────────────────────{RST}");
        eprint!("{DIM}输入编号执行(q 退出): {RST}");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input == "q" || input.is_empty() {
            return Ok(());
        }

        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= all.len() {
                let cmd = &all[num - 1].command;
                eprintln!("{DIM}执行: {RST}{}", cmd);
                eprintln!("{DIM}────────────────────{RST}");
                run_command(cmd)?;
                return Ok(());
            }
        }

        eprintln!("{RED}无效输入，请输入编号或 q{RST}");
    }
}

/// Enter interactive mode: display a numbered menu, let user pick by number.
/// Returns Ok(()) if user quits (q) or after executing a selected command.
pub fn run(records: &[CommandRecord]) -> Result<()> {
    if records.is_empty() {
        eprintln!("{RED}没有历史命令记录{RST}");
        return Ok(());
    }

    loop {
        eprintln!("{DIM}最近命令记录:{RST}");
        for (i, record) in records.iter().enumerate() {
            eprintln!("{CYAN}【{}】{RST}{}", i + 1, record.command);
        }
        eprintln!("{DIM}────────────────────{RST}");
        eprint!("{DIM}输入编号执行(q 退出): {RST}");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim();

        if input == "q" || input.is_empty() {
            return Ok(());
        }

        if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= records.len() {
                let cmd = &records[num - 1].command;
                eprintln!("{DIM}执行: {RST}{}", cmd);
                eprintln!("{DIM}────────────────────{RST}");
                run_command(cmd)?;
                return Ok(());
            }
        }

        eprintln!("{RED}无效输入，请输入编号或 q{RST}");
    }
}
