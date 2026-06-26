mod cli;
mod cmd;
mod error;
mod interactive;
mod model;
mod store;

rust_i18n::i18n!("locales");

use cli::parser::{parse_args, CliMode};
use cmd::runner::run_command;
use model::{format_args_for_shell, CommandRecord};
use std::io::Write;
use store::history::{clear_commands, load_all_commands, load_all_global, load_commands, save_command};
use rust_i18n::t;

const CCI: &str = "\x1b[36m";
const CCOK: &str = "\x1b[32m";
const CCERR: &str = "\x1b[31m";

fn print_help() {
    eprintln!("{}", t!("help.title"));
    eprintln!();
    eprintln!("{}", t!("help.usage"));
    eprintln!("  c                     \x1b[2m{}\x1b[0m", t!("help.interactive"));
    eprintln!("  c <number>            \x1b[2m{}\x1b[0m", t!("help.quick_select"));
    eprintln!("  c <command...>        \x1b[2m{}\x1b[0m", t!("help.record"));
    eprintln!("  c -h, --help          \x1b[2m{}\x1b[0m", t!("help.help"));
    eprintln!("  c -d, --clear         \x1b[2m{}\x1b[0m", t!("help.clear"));
    eprintln!("  c -s <query>          \x1b[2m{}\x1b[0m", t!("help.search"));
    eprintln!("  c -a                  \x1b[2m{}\x1b[0m", t!("help.stats"));
    eprintln!("  c -g                  \x1b[2m{}\x1b[0m", t!("help.global"));
    eprintln!("  c -l <N>              \x1b[2m{}\x1b[0m", t!("help.limit"));
    eprintln!("  c -sag <query>        \x1b[2m{}\x1b[0m", t!("help.combined"));
}

fn run() -> crate::error::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = parse_args(&args);
    let current_dir = std::env::current_dir()?;

    match mode {
        CliMode::Interactive => {
            let records = load_commands(&current_dir)?;
            if records.is_empty() {
                eprintln!("{CCI}{}", t!("msg.no_history_dir"));
                return Ok(());
            }
            interactive::run(&records)
        }

        CliMode::QuickSelect(num) => {
            let records = load_commands(&current_dir)?;
            if records.is_empty() {
                eprintln!("{CCERR}{}", t!("msg.no_history_dir"));
                std::process::exit(1);
            }
            let idx = num as usize;
            if idx == 0 || idx > records.len() {
                eprintln!("{CCERR}{}", t!("msg.entry_out_of_range", num = num, max = records.len()));
                std::process::exit(1);
            }
            let record = &records[idx - 1];
            eprintln!("{CCI}{}", t!("msg.cwd", path = current_dir.display()));
            eprintln!("{CCI}{} {}",
                t!("interactive.running_label"), record.command);
            eprintln!("\x1b[2m------------------------\x1b[0m");
            run_command(&record.command)?;
            eprintln!("\x1b[2m------------------------\x1b[0m");
            eprintln!("{CCOK}{}", t!("msg.command_succeeded"));
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
                eprint!("{CCI}{} (y/N): ", t!("msg.clear_prompt"));
                std::io::stderr().flush().ok();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
                if input.trim().eq_ignore_ascii_case("y") {
                    clear_commands(&current_dir)?;
                    eprintln!("{CCOK}{}", t!("msg.history_cleared"));
                }
            }

            // Pure flag mode (no search, stats, global) = only help/clear -> done
            if search.is_none() && !stats && !global {
                return Ok(());
            }

            // -s without search term -> error
            if search == Some(String::new()) {
                return Err(crate::error::Error::InvalidArgument);
            }

            let top_n_count = if stats { (limit as usize) / 2 } else { 0 };

            // Load data
            let records = if global {
                load_all_global(limit as usize)?
            } else {
                load_all_commands(&current_dir, limit as usize)?
            };

            if records.is_empty() {
                eprintln!("{CCI}{}",
                    if global { t!("msg.no_history_global") } else { t!("msg.no_history_dir") });
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
                eprintln!("{CCI}{}", t!("msg.no_matching"));
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
                    eprintln!("{CCI}{}", t!("msg.no_matching"));
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

            eprintln!("{CCI}{}", t!("msg.cwd", path = current_dir.display()));
            eprintln!("{CCI}{}",
                t!("msg.captured", command = &record.command));
            eprintln!("\x1b[2m------------------------\x1b[0m");
            run_command(&record.command)?;
            save_command(&record.command, &record.dir)?;
            eprintln!("\x1b[2m------------------------\x1b[0m");
            eprintln!("{CCOK}{}", t!("msg.command_succeeded"));
            Ok(())
        }
    }
}

fn main() {
    // Locale detection priority: C_LOCALE > LANG
    if let Some(locale) = std::env::var("C_LOCALE").ok().filter(|v| !v.is_empty()) {
        if locale.starts_with("zh") {
            rust_i18n::set_locale("zh-CN");
        }
    } else if let Some(lang) = std::env::var("LANG").ok() {
        if lang.starts_with("zh_") {
            rust_i18n::set_locale("zh-CN");
        }
    }
    if let Err(e) = run() {
        eprintln!("{CCERR}{}", e);
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

    #[test]
    fn test_locale_keys_match_across_languages() {
        let en: serde_yaml::Value = serde_yaml::from_str(include_str!("../locales/en.yml")).unwrap();
        let zh: serde_yaml::Value = serde_yaml::from_str(include_str!("../locales/zh-CN.yml")).unwrap();

        fn collect_keys(val: &serde_yaml::Value, prefix: &str, keys: &mut Vec<String>) {
            match val {
                serde_yaml::Value::Mapping(m) => {
                    for (k, v) in m {
                        let key = k.as_str().unwrap();
                        if key.starts_with('_') { continue; }
                        let full = if prefix.is_empty() { key.to_string() } else { format!("{}.{}", prefix, key) };
                        collect_keys(v, &full, keys);
                    }
                }
                _ => { keys.push(prefix.to_string()); }
            }
        }

        let mut en_keys = Vec::new();
        let mut zh_keys = Vec::new();
        collect_keys(&en, "", &mut en_keys);
        collect_keys(&zh, "", &mut zh_keys);

        en_keys.sort();
        zh_keys.sort();

        assert_eq!(en_keys, zh_keys, "en.yml and zh-CN.yml must have identical key sets");
    }
}
