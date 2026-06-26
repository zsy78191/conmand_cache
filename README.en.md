# conmand_cache

A lightweight command history tool that lets you **browse, search, and reuse** commands you've run in your terminal.

## Features

- **Command recording**: intercepts `c <command>`, logs it to `~/.cc_history` on success
- **Interactive mode**: run `c` with no arguments for a pickable menu of recent commands (10 per directory)
- **Quick select**: `c 2` runs history entry #2 directly, no menu needed
- **Pipeline support**: wrap a pipeline in single quotes for raw `sh -c` passthrough — `c 'curl ... | jq'`
- **Shell-safe**: multi-argument mode auto-escapes spaces, `$`, backticks, and other special characters
- **History management**: `c -d` clears the current directory's history (with confirmation), `c -h` shows help
- Cross-platform: Linux, macOS, Windows (auto-detects `sh` or `cmd`)

## History File Format

```text
[2026-06-25 10:22:09](/Users/zhangchao/project) ls -la
```

The file is stored at `~/.cc_history`. Entries are grouped by directory; interactive mode shows only entries for the current directory.

## Installation

```bash
git clone https://github.com/zsy78191/conmand_cache.git
cd conmand_cache
cargo install --path .
```

Two binaries are produced:

| Command | Description |
|---------|-------------|
| `c` | Primary command, short and convenient |
| `conmand_cache` | Full package name alias |

> **Note:** The binary name `c` is short and unlikely to conflict with system commands. If `c` is already taken in your environment, use `conmand_cache` instead.

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
最近命令记录:
【1】echo hello | tr a-z A-Z
【2】curl "https://api.example.com/data" | jq
────────────────────
输入编号执行(q 退出):
```

Type a number to run that command, or `q` to quit.

### Quick select

`c 1` runs history entry #1 directly. Any pure-number argument is treated as a quick-select.

### Management

```bash
c -h                  # show help
c --help              # same
c -d                  # clear current directory's history (with confirmation)
c --clear             # same
```

## Shell Quoting Strategy

| Arguments | Behavior | Example |
|-----------|----------|---------|
| None | Enter interactive selection mode | `c` |
| Single | Raw passthrough to `sh -c` — preserves pipes, variables, redirects | `c 'echo $HOME \| grep home'` |
| Multiple | Each argument escaped individually, then joined | `c echo '$HOME'` → literal `$HOME` |

**Rule of thumb:** wrap the full command in single quotes when you need shell features (pipes, variable expansion, redirects); otherwise let `c` handle escaping automatically.

## Uninstall

### Method 1: cargo uninstall

```bash
cargo uninstall conmand_cache
```

Removes both `c` and `conmand_cache`.

### Method 2: Manual removal

```bash
rm ~/.cargo/bin/c               # remove c only
rm ~/.cargo/bin/conmand_cache    # remove conmand_cache only
```

## Development

```bash
cargo test         # run tests (55+ test cases)
cargo check        # type-check
cargo build        # compile
```

## License

MIT
