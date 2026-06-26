/// Describes the entry mode for the `cc` tool.
#[derive(Debug, PartialEq, Eq)]
pub enum CliMode {
    /// No arguments: enter interactive history selection.
    Interactive,
    /// A pure-number argument: quick-select history entry by 1-based index.
    QuickSelect(u32),
    /// A flag-based invocation.
    Flag {
        show_help: bool,
        clear_history: bool,
        search: Option<String>,
        stats: bool,
        global: bool,
        limit: u32,
    },
    /// Anything else: treat as a command to record and execute.
    Command(Vec<String>),
}

// ── Internal helper for building flag state ─────────────────

#[derive(Debug, Default, PartialEq)]
struct ParsedFlags {
    show_help: bool,
    clear_history: bool,
    search: Option<String>,
    stats: bool,
    global: bool,
    limit: Option<u32>,
}

impl ParsedFlags {
    fn into_mode(self) -> CliMode {
        CliMode::Flag {
            show_help: self.show_help,
            clear_history: self.clear_history,
            search: self.search,
            stats: self.stats,
            global: self.global,
            limit: self.limit.unwrap_or(10),
        }
    }
}

/// Parse short flags from a combined token like "-sag" or "-ag".
/// Characters after `-` are parsed left-to-right.
/// `s` (search) and `l` (limit) consume the next CLI argument.
/// Other single chars (a, g, h, d) are boolean flags.
/// Returns None if any flag is unknown (caller should fall back to Command).
fn parse_combined_short_flags(token: &str, args: &[String], pos: &mut usize) -> Option<ParsedFlags> {
    let mut flags = ParsedFlags::default();
    for c in token.chars() {
        match c {
            'h' => flags.show_help = true,
            'd' => flags.clear_history = true,
            's' => {
                *pos += 1;
                if *pos < args.len() {
                    flags.search = Some(args[*pos].clone());
                }
                // missing arg → search stays None
            }
            'a' => flags.stats = true,
            'g' => flags.global = true,
            'l' => {
                *pos += 1;
                if *pos < args.len() {
                    if let Ok(n) = args[*pos].parse::<u32>() {
                        if n > 0 {
                            flags.limit = Some(n);
                        }
                    }
                }
                // missing or invalid → limit stays None (defaults to 10)
            }
            _ => return None, // unknown flag → caller falls back to Command
        }
    }
    Some(flags)
}

/// Classify CLI arguments into a [CliMode].
///
/// Order of precedence:
/// 1. Empty args → Interactive
/// 2. Any arg starts with `-` → Flag parsing loop (known flags only, else Command)
/// 3. First arg is a pure u32 → QuickSelect
/// 4. Everything else → Command
pub fn parse_args(args: &[String]) -> CliMode {
    if args.is_empty() {
        return CliMode::Interactive;
    }

    // ── Flag parsing loop ─────────────────────────────────
    // Only enter if the first argument looks like a flag.
    let first = &args[0];
    if first.starts_with('-') {
        let mut flags = ParsedFlags::default();
        let mut i = 0;
        let mut has_flag = false;

        while i < args.len() {
            let arg = &args[i];

            // Long flags
            if arg == "--help" {
                flags.show_help = true;
                has_flag = true;
            } else if arg == "--clear" {
                flags.clear_history = true;
                has_flag = true;
            } else if arg == "--search" {
                has_flag = true;
                i += 1;
                if i < args.len() {
                    flags.search = Some(args[i].clone());
                }
            } else if arg == "--stats" {
                flags.stats = true;
                has_flag = true;
            } else if arg == "--global" {
                flags.global = true;
                has_flag = true;
            } else if arg == "--limit" {
                has_flag = true;
                i += 1;
                if i < args.len() {
                    if let Ok(n) = args[i].parse::<u32>() {
                        if n > 0 {
                            flags.limit = Some(n);
                        }
                    }
                }
            // Known single-char short flags (-h, -d, -s, -a, -g, -l)
            // Delegates to parse_combined_short_flags to avoid duplicating match logic.
            } else if arg.len() == 2 && arg.starts_with('-') {
                let token = &arg[1..];
                match parse_combined_short_flags(token, args, &mut i) {
                    Some(parsed) => {
                        flags.show_help |= parsed.show_help;
                        flags.clear_history |= parsed.clear_history;
                        if parsed.search.is_some() {
                            flags.search = parsed.search;
                        }
                        flags.stats |= parsed.stats;
                        flags.global |= parsed.global;
                        if parsed.limit.is_some() {
                            flags.limit = parsed.limit;
                        }
                        has_flag = true;
                    }
                    None => return CliMode::Command(args.to_vec()),
                }
            } else if arg.starts_with('-') {
                // Multi-char after single `-`: combined short flags
                let token = &arg[1..];
                match parse_combined_short_flags(token, args, &mut i) {
                    Some(parsed) => {
                        flags.show_help |= parsed.show_help;
                        flags.clear_history |= parsed.clear_history;
                        if parsed.search.is_some() {
                            flags.search = parsed.search;
                        }
                        flags.stats |= parsed.stats;
                        flags.global |= parsed.global;
                        if parsed.limit.is_some() {
                            flags.limit = parsed.limit;
                        }
                        has_flag = true;
                    }
                    None => return CliMode::Command(args.to_vec()),
                }
            } else {
                // Non-flag argument → stop flag processing
                break;
            }

            i += 1;
        }

        if has_flag {
            return flags.into_mode();
        }
    }

    // Quick select — pure number
    if let Ok(num) = first.parse::<u32>() {
        return CliMode::QuickSelect(num);
    }

    // Command mode — pass through all args
    CliMode::Command(args.to_vec())
}

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

    // ── Flag parsing ───────────────────────────────────────────

    #[test]
    fn test_search_flag() {
        assert_eq!(
            parse_args(&s(&["-s", "git"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: false, global: false, limit: 10 }
        );
    }

    #[test]
    fn test_stats_flag() {
        assert_eq!(
            parse_args(&s(&["-a"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: true, global: false, limit: 10 }
        );
    }

    #[test]
    fn test_global_flag() {
        assert_eq!(
            parse_args(&s(&["-g"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: false, global: true, limit: 10 }
        );
    }

    #[test]
    fn test_limit_flag() {
        assert_eq!(
            parse_args(&s(&["-l", "20"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: false, global: false, limit: 20 }
        );
    }

    #[test]
    fn test_combined_sa() {
        assert_eq!(
            parse_args(&s(&["-sa", "git"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: true, global: false, limit: 10 }
        );
    }

    #[test]
    fn test_combined_sag() {
        assert_eq!(
            parse_args(&s(&["-sag", "git"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: true, global: true, limit: 10 }
        );
    }

    #[test]
    fn test_combined_ag() {
        assert_eq!(
            parse_args(&s(&["-ag"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: true, global: true, limit: 10 }
        );
    }

    #[test]
    fn test_combined_sg() {
        assert_eq!(
            parse_args(&s(&["-sg", "git"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: false, global: true, limit: 10 }
        );
    }

    #[test]
    fn test_combined_with_limit() {
        assert_eq!(
            parse_args(&s(&["-sa", "git", "-l", "20"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: true, global: false, limit: 20 }
        );
    }

    #[test]
    fn test_combined_sal_order() {
        assert_eq!(
            parse_args(&s(&["-sal", "git", "20"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: true, global: false, limit: 20 }
        );
    }

    #[test]
    fn test_combined_l_and_search() {
        // -l <N> -s <word> separated
        assert_eq!(
            parse_args(&s(&["-l", "5", "-s", "cargo"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("cargo".into()), stats: false, global: false, limit: 5 }
        );
    }

    #[test]
    fn test_long_flags() {
        assert_eq!(
            parse_args(&s(&["--search", "cargo"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("cargo".into()), stats: false, global: false, limit: 10 }
        );
        assert_eq!(
            parse_args(&s(&["--stats"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: true, global: false, limit: 10 }
        );
        assert_eq!(
            parse_args(&s(&["--global"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: false, global: true, limit: 10 }
        );
        assert_eq!(
            parse_args(&s(&["--limit", "50"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: false, global: false, limit: 50 }
        );
        assert_eq!(
            parse_args(&s(&["--search", "git", "--stats", "--global", "--limit", "5"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: true, global: true, limit: 5 }
        );
    }

    #[test]
    fn test_search_without_argument() {
        // -s without next arg → search = None, limit uses default
        let result = parse_args(&s(&["-s"]));
        match result {
            CliMode::Flag { search: None, limit: 10, .. } => {},
            _ => panic!("Expected Flag with search=None on missing arg"),
        }
    }

    #[test]
    fn test_unknown_short_flag() {
        // 未知短 flag 应回退到 Command
        let result = parse_args(&s(&["-x"]));
        match result {
            CliMode::Command(_) => {},
            _ => panic!("Expected Command mode for unknown flag"),
        }
    }

    #[test]
    fn test_mixed_unknown_and_known_in_combined() {
        // 组合中有未知 flag → 回退到 Command
        let result = parse_args(&s(&["-ax"]));
        match result {
            CliMode::Command(_) => {},
            _ => panic!("Expected Command for unknown flag in combined token"),
        }
    }

    #[test]
    fn test_help_preserved() {
        assert_eq!(
            parse_args(&s(&["-h"])),
            CliMode::Flag { show_help: true, clear_history: false, search: None, stats: false, global: false, limit: 10 }
        );
        assert_eq!(
            parse_args(&s(&["--help"])),
            CliMode::Flag { show_help: true, clear_history: false, search: None, stats: false, global: false, limit: 10 }
        );
    }

    #[test]
    fn test_clear_preserved() {
        assert_eq!(
            parse_args(&s(&["-d"])),
            CliMode::Flag { show_help: false, clear_history: true, search: None, stats: false, global: false, limit: 10 }
        );
        assert_eq!(
            parse_args(&s(&["--clear"])),
            CliMode::Flag { show_help: false, clear_history: true, search: None, stats: false, global: false, limit: 10 }
        );
    }

    #[test]
    fn test_existing_behavior_preserved() {
        assert_eq!(parse_args(&s(&[])), CliMode::Interactive);
        assert_eq!(parse_args(&s(&["5"])), CliMode::QuickSelect(5));
        assert_eq!(
            parse_args(&s(&["echo", "hello"])),
            CliMode::Command(s(&["echo", "hello"]))
        );
        assert_eq!(
            parse_args(&s(&["ls", "-la", "/tmp"])),
            CliMode::Command(s(&["ls", "-la", "/tmp"]))
        );
        // 数字开头但不是纯数字 → Command
        assert_eq!(
            parse_args(&s(&["7z", "x", "file.7z"])),
            CliMode::Command(s(&["7z", "x", "file.7z"]))
        );
        // -la is now a valid combined flag: l=limit→default, a=stats
        assert_eq!(
            parse_args(&s(&["-la"])),
            CliMode::Flag { show_help: false, clear_history: false, search: None, stats: true, global: false, limit: 10 }
        );
    }

    #[test]
    fn test_combined_g_and_s_separated() {
        assert_eq!(
            parse_args(&s(&["-g", "-s", "git"])),
            CliMode::Flag { show_help: false, clear_history: false, search: Some("git".into()), stats: false, global: true, limit: 10 }
        );
    }

    #[test]
    fn test_flag_only_s() {
        let result = parse_args(&s(&["-s"]));
        match result {
            CliMode::Flag { search: None, .. } => {},
            _ => panic!("Expected Flag mode without search term"),
        }
    }

    #[test]
    fn test_limit_non_numeric_uses_default() {
        // limit 接受非数字时降级到默认
        let result = parse_args(&s(&["-l", "abc"]));
        match result {
            CliMode::Flag { limit: 10, .. } => {},
            _ => panic!("Expected Flag with default limit for non-numeric -l"),
        }
    }

    #[test]
    fn test_limit_zero_uses_default() {
        let result = parse_args(&s(&["-l", "0"]));
        match result {
            CliMode::Flag { limit: 10, .. } => {},
            _ => panic!("Expected Flag with default limit for zero"),
        }
    }
}
