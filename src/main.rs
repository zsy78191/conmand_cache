mod cli;
mod cmd;
mod error;
mod interactive;
mod model;
mod store;

use cli::parser::{parse_args, CliMode};
use cmd::runner::run_command;
use model::{format_args_for_shell, CommandRecord};
use std::io::Write;
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

        CliMode::Flag {
            show_help,
            clear_history,
        } => {
            if show_help {
                print_help();
            }
            if clear_history {
                eprint!("{CCI}确认清除当前目录历史记录？(y/N): ");
                std::io::stderr().flush().ok();
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
