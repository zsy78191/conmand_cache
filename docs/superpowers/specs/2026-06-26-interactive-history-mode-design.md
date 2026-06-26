# 交互式历史记录选择模式设计

## 概述

为 `cc` CLI 工具添加交互式历史记录选择功能，支持四种入口模式，让用户可以浏览、快速选择、管理和检查当前目录的历史命令。

## 入口模式

`cc` 参数解析产生四种模式（`CliMode` 枚举），判定逻辑按以下顺序：

| 条件 | 模式 | 行为 |
|------|------|------|
| `args.is_empty()` | `Interactive` | 进入交互选择菜单 |
| `args[0]` 是纯数字（如 `1`, `23`） | `QuickSelect(u32)` | 直接按编号执行历史命令 |
| `args[0]` 以 `-` 开头 | `Flag { show_help, clear_history }` | 显示帮助或清除历史 |
| 其他 | `Command(Vec<String>)` | 正常记录并执行命令（现有流程） |

`QuickSelect` 中的编号对应交互菜单中显示的序号（1-based），不是文件行号。

## 交互菜单流程

```
cc | 最近命令记录（/current/dir）:
cc |  1  echo hello world
cc |  2  curl http://example.com | jq
cc |  3  ls -la /tmp
cc | ------------------------
cc | 输入编号执行(q 退出):
```

- 用户输入 `1-9` → 执行对应命令 → 执行完成后退出 cc
- 输入 `q` → 直接退出，不执行任何命令
- 其他输入 → 显示 `无效输入，请输入编号或 q` → 重新显示菜单

每次显示最多最近 10 条，按时间倒序（最新的在 `1`）。

## 历史记录加载

从 `~/.cc_history` 文件中加载历史记录，格式为：

```
[2026-06-26 12:34:56](/path/to/dir) echo hello
```

存储新增两个函数：
- `load_commands(dir: &PathBuf) -> Result<Vec<CommandRecord>>` — 按目录过滤，返回最近 10 条
- `clear_commands(dir: &PathBuf) -> Result<()>` — 清除指定目录的所有历史记录

## Flags

### `-h` / `--help` 输出内容

```
cc — 命令缓存与执行工具

用法:
  cc                   进入交互模式，选择执行历史命令
  cc <数字>            快速执行指定编号的历史命令
  cc <命令...>         记录并执行命令
  cc -h, --help        显示此帮助信息
  cc -d, --clear       清除当前目录的历史记录
```

### `-d` / `--clear` 清除流程

1. 显示确认提示：`cc | 确认清除当前目录历史记录？(y/N):`
2. 用户输入 `y` 或 `Y` → 清除当前目录的所有历史记录，显示成功
3. 默认 `n` → 取消清除

## 架构设计

### 文件结构

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/cli/parser.rs` | 新增 | `CliMode` 枚举 + `parse_args()` 函数 |
| `src/cli/mod.rs` | 新增 | 模块入口 |
| `src/interactive/mod.rs` | 新增 | 交互式菜单逻辑 |
| `src/main.rs` | 修改 | 接入 `CliMode` 分发 |
| `src/store/history.rs` | 修改 | 追加 `load_commands()`, `clear_commands()` |
| `src/cmd/runner.rs` | 不变 | 复用已有 `run_command()` |

### 数据流

```
main::run()
  ↓
cli::parser::parse_args(&args)  →  CliMode
  ↓ dispatch
Interactive  →  interactive::run(history)  →  run_command(selected)
QuickSelect  →  load_commands() → index → run_command()
Flag(-h)     →  打印帮助信息
Flag(-d)     →  clear_commands() 确认后清除
Command      →  format_args_for_shell() → run_command() → save_command()
```

### CliMode 定义

```rust
pub enum CliMode {
    Interactive,
    QuickSelect(u32),
    Flag { show_help: bool, clear_history: bool },
    Command(Vec<String>),
}
```

## 错误处理

- 历史文件不存在或无法读取 → 空列表，交互模式显示"暂无历史记录"
- 快速选择编号超出范围 → 打印错误并退出（非 0 码）
- 清除确认输入非法 → 视为取消
- 命令执行失败 → 现有 `Error::CommandFailed` 处理流程不变

## 测试要点

- `parse_args()`: 每种模式的边界测试（空数组、纯数字、负号开头、混合）
- `load_commands()`: 文件解析测试，空文件测试
- `clear_commands()`: 按目录过滤清除测试
- 交互模块的手动测试（需要 stdin 模拟）
