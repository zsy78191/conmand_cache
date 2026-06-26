# conmand_cache

> [English](README.en.md)

一个轻量级的命令历史记录工具，让你在终端执行过的命令**可浏览、可检索、可复用**。

## 功能

- **命令记录**：拦截 `cc <命令>` 的执行，命令成功执行后写入 `~/.cc_history`
- **交互模式**：直接执行 `cc` 进入交互选择菜单，查看当前目录最近 10 条历史
- **快速选择**：`cc 2` 直接执行编号为 2 的历史命令，无需进入菜单
- **管道支持**：用单引号包裹整条管道命令，`cc 'curl ... | jq'` 原样传给子 shell
- **Shell 安全**：多参数模式自动转义空格、`$`、反引号等特殊字符
- **历史管理**：`cc -d` 清除当前目录记录（需二次确认），`cc -h` 查看帮助
- 跨平台支持：Linux、macOS、Windows（自动选择 `sh` 或 `cmd`）

## 历史文件格式

```text
[2026-06-25 10:22:09](/Users/zhangchao/project) ls -la
```

文件保存在 `~/.cc_history`，按目录分组。交互模式只显示当前目录的记录。

## 安装

```bash
git clone https://github.com/zsy78191/conmand_cache.git
cd conmand_cache
cargo install --path .
```

安装后产生两个可执行文件：

| 命令名 | 说明 |
|--------|------|
| `cc` | 主命令 |
| `conmand_cache` | 与包名相同的完整命令 |

> **注意：** `cc` 在 macOS 上也是 C 编译器（clang）的标准名称。安装后 `~/.cargo/bin` 排在前面的情况下，`cc` 指向本工具。日常影响极小——极少有人直接敲 `cc` 编译 C 文件。如需调用 C 编译器，请使用 `clang` 或 `/usr/bin/cc`。

## 使用

### 记录并执行命令

```bash
cc ls -la              # 执行 ls -la 并记录
cc cargo test          # 执行 cargo test 并记录
cc echo hello          # 简单命令
```

### 带管道的命令

用单引号包裹整条命令，原样传给 `sh -c` 执行：

```bash
cc 'curl "https://api.example.com/data" | jq .items'
cc 'echo hello | tr a-z A-Z'
cc 'cat file.txt | sort | uniq'
```

### 交互模式

直接执行 `cc` 进入交互菜单，查看当前目录最近 10 条命令：

```text
最近命令记录:
【1】echo hello | tr a-z A-Z
【2】curl "https://api.example.com/data" | jq
────────────────────
输入编号执行(q 退出):
```

输入编号执行对应命令，输入 `q` 退出。

### 快速选择

`cc 1` 直接执行编号 1 的历史命令，无需进入菜单。纯数字参数被识别为快速选择。

### 管理

```bash
cc -h                  # 显示帮助
cc --help              # 同上
cc -d                  # 清除当前目录的历史记录（需二次确认）
cc --clear             # 同上
```

## Shell 引用策略

| 参数数量 | 行为 | 示例 |
|---------|------|------|
| 无参数 | 进入交互选择模式 | `cc` |
| 单个参数 | 原样传给 `sh -c`，保留管道、变量、重定向 | `cc 'echo $HOME \| grep home'` |
| 多个参数 | 逐个转义特殊字符后拼接 | `cc echo '$HOME'` → 输出字面量 `$HOME` |

**经验法则**：需要 shell 特性（管道、变量展开、重定向）时用单引号包裹整条命令；否则让 cc 自动处理转义。

## 卸载

### 方法一：cargo uninstall

```bash
cargo uninstall conmand_cache
```

同时删除 `cc` 和 `conmand_cache`。

### 方法二：手动删除单个命令

```bash
rm ~/.cargo/bin/cc           # 只删 cc
rm ~/.cargo/bin/conmand_cache # 只删 conmand_cache
```

## 开发

```bash
cargo test         # 运行测试（55+ 测试用例）
cargo check        # 检查编译
cargo build        # 编译
```

## License

MIT
