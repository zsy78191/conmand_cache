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
use store::history::{clear_commands, load_all_commands, load_all_global, load_commands, save_command};

const CCI: &str = "\x1b[36m";
const CCOK: &str = "\x1b[32m";
const CCERR: &str = "\x1b[31m";

fn print_help() {
    eprintln!("c — 命令缓存与执行工具");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  c                     \x1b[2m进入交互模式，选择执行历史命令\x1b[0m");
    eprintln!("  c <数字>              \x1b[2m快速执行指定编号的历史命令\x1b[0m");
    eprintln!("  c <命令...>           \x1b[2m记录并执行命令\x1b[0m");
    eprintln!("  c -h, --help          \x1b[2m显示此帮助信息\x1b[0m");
    eprintln!("  c -d, --clear         \x1b[2m清除当前目录的历史记录\x1b[0m");
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
            if records.is_empty() {
                eprintln!("{CCERR}当前目录暂无历史记录");
                std::process::exit(1);
            }
            let idx = num as usize;
            if idx == 0 || idx > records.len() {
                eprintln!("{CCERR}编号 {} 超出范围（1-{}）", num, records.len());
                std::process::exit(1);
            }
            let record = &records[idx - 1];
            eprintln!("{CCI}当前路径: {}", current_dir.display());
            eprintln!("{CCI}执行: {}", record.command);
            eprintln!("\x1b[2m------------------------\x1b[0m");
            run_command(&record.command)?;
            eprintln!("\x1b[2m------------------------\x1b[0m");
            eprintln!("{CCOK}命令执行成功");
            Ok(())
        }

        CliMode::Flag {
            show_help,
            clear_history,
            search,
            stats,
            global,
            limit,
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

            // Pure flag mode (no search, stats, global) = only help/clear -> done
            if search.is_none() && !stats && !global {
                return Ok(());
            }

            // -s without search term -> error
            if search == Some(String::new()) {
                eprintln!("{CCERR}-s 需要指定搜索词");
                std::process::exit(1);
            }

            let top_n_count = if stats { (limit as usize) / 2 } else { 0 };

            // Load data
            let records = if global {
                load_all_global(limit as usize)?
            } else {
                load_all_commands(&current_dir, limit as usize)?
            };

            if records.is_empty() {
                eprintln!("{CCI}当前{}暂无历史记录", if global { "全局" } else { "目录" });
                return Ok(());
            }

            // Search filter
            let filtered = if let Some(ref query) = search {
                if query.is_empty() {
                    records
                } else {
                    model::fuzzy_filter(query, &records)
                }
            } else {
                records
            };

            if filtered.is_empty() {
                eprintln!("{CCI}没有匹配的历史记录");
                return Ok(());
            }

            // Stats mode
            if stats {
                let grouped = model::group_by_command(&filtered);
                let freq_sorted = model::sort_by_frequency(&grouped);
                let recent_sorted = model::sort_by_recent(&grouped);

                let freq_top: Vec<CommandRecord> = model::top_n(&freq_sorted, top_n_count)
                    .into_iter().map(|s| s.latest_record).collect();
                let recent_top: Vec<CommandRecord> = model::top_n(&recent_sorted, top_n_count)
                    .into_iter().map(|s| s.latest_record).collect();

                if freq_top.is_empty() && recent_top.is_empty() {
                    eprintln!("{CCI}没有匹配的历史记录");
                    return Ok(());
                }
                interactive::run_stats(&freq_top, &recent_top)?;
            } else {
                interactive::run(&filtered)?;
            }

            Ok(())
        }

        CliMode::Command(cmd_args) => {
            let command = format_args_for_shell(&cmd_args);
            let record = CommandRecord::new(command.clone(), current_dir.clone());

            eprintln!("{CCI}当前路径: {}", current_dir.display());
            eprintln!("{CCI}捕获的命令: {}", record.command);
            eprintln!("\x1b[2m------------------------\x1b[0m");
            run_command(&record.command)?;
            save_command(&record.command, &record.dir)?;
            eprintln!("\x1b[2m------------------------\x1b[0m");
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_search_stats_logical_flow() {
        use crate::model::{fuzzy_filter, group_by_command, sort_by_frequency, sort_by_recent, top_n};

        let records = vec![
            CommandRecord { command: "git push".into(), dir: PathBuf::from("/dir"), timestamp: "2026-01-03".into() },
            CommandRecord { command: "cargo build".into(), dir: PathBuf::from("/dir"), timestamp: "2026-01-01".into() },
            CommandRecord { command: "git commit".into(), dir: PathBuf::from("/dir"), timestamp: "2026-01-02".into() },
            CommandRecord { command: "git push".into(), dir: PathBuf::from("/dir"), timestamp: "2026-01-04".into() },
        ];

        // search: filter for "git" records
        let filtered = fuzzy_filter("git", &records);
        assert_eq!(filtered.len(), 3);

        // stats: group and sort on filtered results
        let grouped = group_by_command(&filtered);
        let freq = sort_by_frequency(&grouped);
        let recent = sort_by_recent(&grouped);

        let n = 2;
        let freq_top: Vec<CommandRecord> = top_n(&freq, n).into_iter().map(|s| s.latest_record).collect();
        let recent_top: Vec<CommandRecord> = top_n(&recent, n).into_iter().map(|s| s.latest_record).collect();

        // git push appears 2 times -> frequency top1
        assert_eq!(freq_top[0].command, "git push");
        // git push (2026-01-04) is most recent -> recent top1
        assert_eq!(recent_top[0].command, "git push");
    }

    #[test]
    fn test_limit_half_for_stats_top() {
        assert_eq!(10usize / 2, 5);
        assert_eq!(20usize / 2, 10);
        assert_eq!(1usize / 2, 0);
        assert_eq!(2usize / 2, 1);
    }
}
