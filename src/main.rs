mod cmd;
mod error;
mod model;
mod store;

use cmd::runner::run_command;
use model::{CommandFormatter, CommandRecord, ShellEscapeFormatter};
use store::history::save_command;

fn run() -> crate::error::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("用法: c <命令>");
        std::process::exit(1);
    }

    let formatter = ShellEscapeFormatter;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let command = formatter.format(&arg_refs);

    let current_dir = std::env::current_dir()?;
    let record = CommandRecord::new(command.clone(), current_dir.clone());

    println!(
        "当前路径: {}, 捕获的命令: {}",
        current_dir.display(),
        record.command
    );

    run_command(&record.command)?;
    save_command(&record.command, &record.dir)?;
    println!("命令执行成功");

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
