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
