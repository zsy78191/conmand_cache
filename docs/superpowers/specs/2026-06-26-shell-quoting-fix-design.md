# 修复路径含空格/特殊字符时的参数拆分问题

## 问题描述

当用户通过 `cc` 执行包含空格或特殊字符（如 `$`、`!`）的路径参数时，路径会被错误拆分为多个参数：

```bash
# 输入
cc ls -lah "/Users/zhangchao/2026/rust/conmand_cache/context/test dir"

# 期望行为：等价于直接执行
ls -lah "/Users/zhangchao/2026/rust/conmand_cache/context/test dir"

# 实际行为
ls -lah /Users/zhangchao/2026/rust/conmand_cache/context/test dir
# "test dir" 被拆成两个参数，命令失败
```

## 根本原因

数据流如下：

```
std::env::args()          # ["ls", "-lah", "/path/test dir"] — 引号已被外层 shell 消费
       │
       ▼
SimpleFormatter::format() # "ls -lah /path/test dir"        — join(" ") 不恢复引号
       │
       ▼
Command::new("sh").arg("-c").arg(command)  # 字符串传给 sh
       │
       ▼
sh 重新解析字符串          # 按空格分词，"test" 和 "dir" 被拆成两个参数
```

`std::env::args()` 返回的是**已经经过外层 shell 解析后的纯参数**——引号信息、反斜杠转义等信息在到达 `cc` 进程之前已经被消费。`join(" ")` 简单拼接后，传入 `sh -c` 时 shell 重新按空格分词，带空格的原生参数就被拆分了。

## 影响范围

所有包含以下内容的参数都会触发此问题：

- 路径中包含空格（如 `my documents/file.txt`）
- 路径中包含 shell 元字符（如 `file$(date).txt` 被当作命令替换）
- 路径中包含引号（如 `file's name.txt`）

## 方案选择

### 不采用方案 D（直接执行，去掉 sh -c）

虽然去掉 `sh -c` 用 `Command::new().args()` 能从根本上消除 quoting 问题，但会失去所有 shell 特性。

`sh -c` 提供的 shell 特性中，管道 `|`、重定向 `>`、命令连接 `&&` 等**对用户是有实际价值的**。用户已经在使用 `cc 'echo "123" && echo "234"'` 这样的用法。去掉这些特性等于让 `cc` 退化为一个简单的参数传递工具，偏离了"封装 shell 命令"的核心定位。

### 采用方案 A（shell-escape 拼接，保留 sh -c）

对每个参数做 shell quoting 后再 `join(" ")`，恢复引号信息，使 `sh -c` 能正确解析。

引入 `shell-escape` crate（0.2.x）处理 quoting 规则，包括：

- 含空格/特殊字符的参数 → 加单引号包裹：`hello world` → `'hello world'`
- 含单引号的参数 → 转义：`it's me` → `'it'\''s me'`
- `$`、`!`、反引号等元字符 → 加引号阻止展开

## 架构变更

### 变更前

```
std::env::args()
     │
     ▼
  Vec<String>
     │
     ├── SimpleFormatter::format()  → args.join(" ")
     │       │
     │       ▼
     │   CommandRecord { command: String }
     │       │
     │       ├── run_command(&command)  → sh -c "string"
     │       └── save_command(&command) → 写入历史文件
     │
     └── println!("捕获的命令: {}", command)
```

### 变更后

```
std::env::args()
     │
     ▼
  Vec<String>
     │
     ├── ShellEscapeFormatter::format()  → 每个 arg 做 shell quoting 后 join(" ")
     │       │
     │       ▼
     │   CommandRecord { display: String }
     │       │
     │       ├── run_command(&display)   → sh -c "escaped string"
     │       └── save_command(&display)  → 写入历史文件
     │
     └── println!("捕获的命令: {}", display)
```

`Vec<String>` 是事实来源，`display` 字符串是经过 quoting 后的展示视图。

## 文件变更

### `Cargo.toml`

新增依赖：
```toml
shell-escape = "0.2"
```

### `src/model/format.rs`

- `SimpleFormatter` 重命名为 `ShellEscapeFormatter`（或新增，保留 SimpleFormatter 作为 fallback）
- `format()` 对每个 arg 调用 `shell_escape::escape(arg)` 后 `join(" ")`

### `src/model/mod.rs`

- `CommandRecord` 的 `command` 字段语义不变（仍为展示/执行用的字符串）
- 无结构性变更

### `src/main.rs`

- 数据流不变：`args` → `format()` → `record.command` → `run_command` + `save_command`
- `ShellEscapeFormatter` 替换 `SimpleFormatter`
- 展示和日志输出自动使用 quoting 后的字符串

### `src/cmd/runner.rs`

- 无变更：仍通过 `sh -c` 执行，接口 `&str` 不变

## 测试

新增测试用例：

```rust
// 带空格路径
cc ls -lah "/path/test dir"
// 期望：记录为 ls -lah '/path/test dir'
// 执行：sh 正确解析为单个参数 "test dir"

# 含特殊字符
cc echo "it's working"
// 期望：记录为 echo 'it'\''s working'

# 含元字符
cc ls "$HOME/file.txt"
// 期望：记录为 ls '$HOME/file.txt'（阻止变量展开）
```

## 边界情况

- 空参数 `""`：`shell_escape::escape("")` 返回 `"''"`，sh 解析为正确空字符串
- 纯空格参数 `"   "`：同上，加引号包裹
- 仅含安全字符的参数（如 `ls`、`-la`）：`shell-escape` 不添加额外引号，保持原样

## 向后兼容

- 命令行接口不变：用户输入方式不变
- 历史记录文件 `.cc_history` 格式中，带特殊字符的命令会多出引号/转义，但**用户 copy-paste 回终端执行时能正确工作**（这正是 quoting 的目的）
- 已有的纯文本历史记录（不含特殊字符）不受影响
