# conmand_cache

A lightweight command history tool that lets you **browse, search, and reuse** commands you've run in your terminal.

## Features

- **Command recording**: intercepts `cc <command>`, logs it to `~/.cc_history` on success
- **Interactive mode**: run `cc` with no arguments for a pickable menu of recent commands (10 per directory)
- **Quick select**: `cc 2` runs history entry #2 directly, no menu needed
- **Pipeline support**: wrap a pipeline in single quotes for raw `sh -c` passthrough — `cc 'curl ... | jq'`
- **Shell-safe**: multi-argument mode auto-escapes spaces, `$`, backticks, and other special characters
- **History management**: `cc -d` clears the current directory's history (with confirmation), `cc -h` shows help
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
| `cc` | Primary command, short and convenient |
| `conmand_cache` | Full package name alias |

> **Note:** On macOS, `cc` is also the standard name for the C compiler (clang). After installation, `~/.cargo/bin` typically takes precedence over `/usr/bin`, so `cc` will point to this tool. This rarely matters in practice — very few people type `cc` directly to compile C files. Use `clang` or `/usr/bin/cc` when you need the system C compiler.

## Usage

### Record and run a command

```bash
cc ls -la              # runs ls -la and records it
cc cargo test          # runs cargo test and records it
cc echo hello          # simple command
```

### Pipelines and shell features

Wrap the full command in single quotes for raw `sh -c` passthrough:

```bash
cc 'curl "https://api.example.com/data" | jq .items'
cc 'echo hello | tr a-z A-Z'
cc 'cat file.txt | sort | uniq'
```

### Interactive mode

Run `cc` with no arguments to enter the interactive menu:

```text
最近命令记录:
【1】echo hello | tr a-z A-Z
【2】curl "https://api.example.com/data" | jq
────────────────────
输入编号执行(q 退出):
```

Type a number to run that command, or `q` to quit.

### Quick select

`cc 1` runs history entry #1 directly. Any pure-number argument is treated as a quick-select.

### Management

```bash
cc -h                  # show help
cc --help              # same
cc -d                  # clear current directory's history (with confirmation)
cc --clear             # same
```

## Shell Quoting Strategy

| Arguments | Behavior | Example |
|-----------|----------|---------|
| None | Enter interactive selection mode | `cc` |
| Single | Raw passthrough to `sh -c` — preserves pipes, variables, redirects | `cc 'echo $HOME \| grep home'` |
| Multiple | Each argument escaped individually, then joined | `cc echo '$HOME'` → literal `$HOME` |

**Rule of thumb:** wrap the full command in single quotes when you need shell features (pipes, variable expansion, redirects); otherwise let cc handle escaping automatically.

## Uninstall

### Method 1: cargo uninstall

```bash
cargo uninstall conmand_cache
```

Removes both `cc` and `conmand_cache`.

### Method 2: Manual removal

```bash
rm ~/.cargo/bin/cc              # remove cc only
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
