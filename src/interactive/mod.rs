use std::io::Write;
use crate::cmd::runner::run_command;
use crate::error::Result;
use crate::model::CommandRecord;

const CCI: &str = "\x1b[36mcc |\x1b[0m ";
const CCERR: &str = "\x1b[31mcc |\x1b[0m ";

/// Enter interactive mode: display a numbered menu, let user pick by number.
/// Returns Ok(()) if user quits (q) or after executing a selected command.
pub fn run(records: &[CommandRecord]) -> Result<()> {
    if records.is_empty() {
        eprintln!("{CCERR}没有历史命令记录");
        return Ok(());
    }

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

        if input == "q" || input.is_empty() {
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
