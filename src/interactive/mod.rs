use std::io::Write;
use crate::cmd::runner::run_command;
use crate::error::Result;
use crate::model::CommandRecord;

const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";
const RST: &str = "\x1b[0m";

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
