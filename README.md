# conmand_cache

> [English](README.en.md)

轻量级命令历史记录工具，让你在终端执行过的命令**可浏览、可搜索、可复用**。

## 功能

- **命令记录**：拦截 `c <命令>` 的执行，成功后写入 `~/.cc_history`
- **交互模式**：直接执行 `c` 进入选择菜单，查看最近命令
- **快速选择**：`c 2` 直接执行编号 2 的历史命令，无需进入菜单
- **模糊搜索**：`c -s <词>` 模糊搜索历史命令（支持 POSIX 组合参数如 `-sag`）
- **统计模式**：`c -a` 双区域展示 — 使用最频繁和最近常用
- **全局范围**：`c -g` 搜索所有目录的历史记录，不限于当前目录
- **条数控制**：`c -l N` 控制加载条数（默认 10，统计 top = N/2）
- **管道支持**：单引号包裹整条命令原样传给 `sh -c` — `c 'curl ... | jq'`
- **Shell 安全**：多参数模式自动转义空格、`$`、反引号等特殊字符
- **历史管理**：`c -d` 清除当前目录记录（需确认）
- **多语言支持**：根据 `LANG` 环境变量自动切换中英文（`zh_CN` → 中文）
- 跨平台：Linux、macOS、Windows（自动选择 `sh` 或 `cmd`）

## 历史文件格式

```text
[2026-06-25 10:22:09](/Users/zhangchao/project) ls -la
```

保存在 `~/.cc_history`，按目录分组。交互模式只显示当前目录的记录（使用 `-g` 查看全局）。

## 安装

```bash
git clone https://github.com/zsy78191/conmand_cache.git
cd conmand_cache
cargo install --path .
```

安装后产生一个可执行文件：`c`。

> **注意：** 二进制名 `c` 通常不会与系统命令冲突。如果你的环境里 `c` 已被占用，可以用 `conmand_cache` 别名代替。

## 使用

### 记录并执行命令

```bash
c ls -la              # 执行 ls -la 并记录
c cargo test          # 执行 cargo test 并记录
c echo hello          # 简单命令
```

### 带管道的命令

用单引号包裹整条命令，原样传给 `sh -c` 执行：

```bash
c 'curl "https://api.example.com/data" | jq .items'
c 'echo hello | tr a-z A-Z'
c 'cat file.txt | sort | uniq'
```

### 交互模式

直接执行 `c` 进入交互菜单：

```text
recent commands:
【1】echo hello | tr a-z A-Z
【2】curl "https://api.example.com/data" | jq
────────────────────
enter number to run (q quit):
```

输入编号执行对应命令，输入 `q` 退出。

### 快速选择

`c 1` 直接执行编号 1 的历史命令。纯数字参数被识别为快速选择。

### 模糊搜索

```bash
c -s git              # 模糊搜索包含 "git" 的命令
c -s gpo              # 子序列匹配 — 找到 "git push origin"
c -sg cargo           # 搜索 + 全局范围
```

结果按匹配度排序。输入编号执行。

### 多语言支持

根据 `LANG` 环境变量自动切换界面语言：

```bash
LANG=zh_CN.UTF-8 c -h    # 中文界面
LANG=en_US.UTF-8 c -h    # 英文界面（默认）
```

如果你系统语言是英文但想 `c` 用中文，设置 `C_LOCALE` 环境变量：

```bash
export C_LOCALE=zh_CN     # 只影响 c 工具，不影响系统
```

### 统计模式

展示两个区域，编号连续递增：

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

配合搜索和全局参数使用：

```bash
c -a                  # 当前目录统计
c -ag                 # 全局统计
c -sa git             # 搜索后统计
c -sag cargo          # 搜索 + 统计 + 全局
```

### 条数控制

```bash
c -l 20               # 加载 20 条（默认 10）
c -al 20              # 统计模式，top = 10
c -sal git 20         # 搜索 + 统计 + 限制
```

### 管理

```bash
c -h                  # 显示帮助
c -d                  # 清除当前目录的历史记录（需确认）
```

## Shell 引用策略

| 参数数量 | 行为 | 示例 |
|---------|------|------|
| 无参数 | 进入交互选择模式 | `c` |
| 单个参数 | 原样传给 `sh -c`，保留管道、变量、重定向 | `c 'echo $HOME \| grep home'` |
| 多个参数 | 逐个转义特殊字符后拼接 | `c echo '$HOME'` → 输出字面量 `$HOME` |

**经验法则：** 需要 shell 特性（管道、变量展开、重定向）时用单引号包裹整条命令；否则让 `c` 自动处理转义。

## 卸载

```bash
cargo uninstall conmand_cache
```

或手动删除：

```bash
rm ~/.cargo/bin/c
```

注意：`cargo uninstall conmand_cache` 会删除 `c` 二进制文件。

## 开发

```bash
cargo test         # 运行测试（106+ 测试用例）
cargo check        # 检查编译
cargo build        # 编译
```

## License

MIT
