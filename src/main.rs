mod cmd;
mod error;
mod model;
mod store;

use cmd::runner::run_command;
use model::{format_args_for_shell, CommandRecord};
use store::history::save_command;

const CCI: &str = "\x1b[36mcc |\x1b[0m ";
const CCOK: &str = "\x1b[32mcc |\x1b[0m ";
const CCERR: &str = "\x1b[31mcc |\x1b[0m ";

fn run() -> crate::error::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{CCERR}用法: c <命令>");
        std::process::exit(1);
    }

    let command = format_args_for_shell(&args);

    let current_dir = std::env::current_dir()?;
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

fn main() {
    if let Err(e) = run() {
        eprintln!("{CCERR}错误: {}", e);
        std::process::exit(1);
    }
}
