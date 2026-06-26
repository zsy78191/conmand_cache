# conmand_cache

> [中文](README.md)

A lightweight command history tool that lets you **browse, search, and reuse** commands you've run in your terminal.

## Features

- **Command recording**: intercepts `c <command>`, logs it to `~/.cc_history` on success
- **Interactive mode**: run `c` with no arguments for a pickable menu of recent commands
- **Quick select**: `c 2` runs history entry #2 directly, no menu needed
- **Fuzzy search**: `c -s <query>` searches history with fuzzy matching (supports POSIX combined flags like `-sag`)
- **Stats mode**: `c -a` shows two ranked views — most frequent commands and most recent commands
- **Global scope**: `c -g` queries history across all directories instead of just the current one
- **Limit control**: `c -l <N>` controls how many entries to load (default 10; stats top N = N/2)
- **Pipeline support**: wrap in single quotes for raw `sh -c` passthrough — `c 'curl ... | jq'`
- **Shell-safe**: multi-argument mode auto-escapes spaces, `$`, backticks, and other special characters
- **History management**: `c -d` clears current directory's history (with confirmation)
- **Multi-language**: auto-switches between English and Chinese based on `LANG` env var
- Cross-platform: Linux, macOS, Windows (auto-detects `sh` or `cmd`)

## History File Format

```text
[2026-06-25 10:22:09](/Users/zhangchao/project) ls -la
```

Stored at `~/.cc_history`. Entries are grouped by directory; interactive mode shows only entries for the current directory (use `-g` for global).

## Installation

```bash
git clone https://github.com/zsy78191/conmand_cache.git
cd conmand_cache
cargo install --path .
```

Installs a single binary: `c`.

> **Note:** The binary name `c` is short and unlikely to conflict with system commands. If `c` is already taken in your environment, you can alias `conmand_cache` (see below).

## Usage

### Record and run a command

```bash
c ls -la              # runs ls -la and records it
c cargo test          # runs cargo test and records it
c echo hello          # simple command
```

### Pipelines and shell features

Wrap the full command in single quotes for raw `sh -c` passthrough:

```bash
c 'curl "https://api.example.com/data" | jq .items'
c 'echo hello | tr a-z A-Z'
c 'cat file.txt | sort | uniq'
```

### Interactive mode

Run `c` with no arguments to enter the interactive menu:

```text
recent commands:
【1】echo hello | tr a-z A-Z
【2】curl "https://api.example.com/data" | jq
────────────────────
enter number to run (q quit):
```

Type a number to run that command, or `q` to quit.

### Quick select

`c 1` runs history entry #1 directly. Any pure-number argument is treated as a quick-select.

### Fuzzy search

```bash
c -s git              # fuzzy search commands matching "git"
c -s gpo              # subsequence match — finds "git push origin"
c -sg cargo           # search + global scope
```

Results are ordered by match relevance. Type a number to run.

### Multi-language support

The UI language is auto-detected from the `LANG` environment variable:

```bash
LANG=en_US.UTF-8 c -h    # English (default)
LANG=zh_CN.UTF-8 c -h    # Chinese
```

To force Chinese without changing your system language, set `C_LOCALE`:

```bash
export C_LOCALE=zh_CN     # only affects c, not your system
```

### Stats mode

Shows two ranked views of your command history, each with consecutive numbering:

```text
━━━ Most Frequent ━━━
【1】git push origin main
【2】cargo build
...
【5】echo hello

━━━ Most Recent ━━━
【6】cargo test
【7】npm run dev
...
【10】cargo build
```

Combine with search and global:

```bash
c -a                  # stats for current directory
c -ag                 # stats across all directories
c -sa git             # search + stats on results
c -sag cargo          # search + stats + global
```

### Limit control

```bash
c -l 20               # load 20 entries (default 10)
c -al 20              # stats mode with top = 10
c -sal git 20         # search + stats + limit
```

### Management

```bash
c -h                  # show help
c -d                  # clear current directory's history (with confirmation)
```

## Shell Quoting Strategy

| Arguments | Behavior | Example |
|-----------|----------|---------|
| None | Interactive selection mode | `c` |
| Single | Raw passthrough to `sh -c` — preserves pipes, variables, redirects | `c 'echo $HOME \| grep home'` |
| Multiple | Each argument escaped individually, then joined | `c echo '$HOME'` → literal `$HOME` |

**Rule of thumb:** wrap the full command in single quotes when you need shell features (pipes, variable expansion, redirects); otherwise let `c` handle escaping automatically.

## Uninstall

```bash
cargo uninstall conmand_cache
```

Or remove manually:

```bash
rm ~/.cargo/bin/c
```

Note: `cargo uninstall conmand_cache` removes the `c` binary.

## Development

```bash
cargo test         # run tests (106+ test cases)
cargo check        # type-check
cargo build        # compile
```

## License

MIT
