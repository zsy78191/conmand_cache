# conmand_cache

一个轻量级的命令历史记录工具，捕获你在终端执行的命令并持久化到 `~/.cc_history`。

## 功能

- 拦截 `cc <命令>` 的执行，将命令（含时间戳和目录）写入 `~/.cc_history`
- 只有命令**成功执行**（退出码为 0）才写入历史
- 跨平台支持：Linux、macOS、Windows（自动选择 `sh` 或 `cmd`）
- 模块化设计，可扩展的命令格式化器（`CommandFormatter` trait）

## 格式

历史文件每行格式：

```
[2026-06-25 10:22:09](/Users/zhangchao/conmand_cache) ls -la
```

## 安装

```bash
git clone https://github.com/zsy78191/conmand_cache.git
cd conmand_cache
cargo install --path .
```

安装后会产生两个可执行文件：

| 命令名 | 说明 |
|--------|------|
| `cc` | 主命令，简短易用 |
| `conmand_cache` | 与包名相同的完整命令 |

> **注意：** `cc` 在 macOS 上是 C 编译器的标准名称（即 clang）。安装后 `~/.cargo/bin` 会排在 `/usr/bin` 前面，导致 `cc` 指向本工具而非系统编译器。日常影响极小（极少有人直接敲 `cc` 编译 C 文件），如果确实需要调用 C 编译器，请使用 `clang` 或 `/usr/bin/cc`。

## 使用

```bash
cc ls -la          # 执行 ls -la 并记录到历史
cc cargo test      # 执行 cargo test 并记录
```

## 卸载

### 方法一：cargo uninstall（整包删除）

```bash
cargo uninstall conmand_cache
```

这会同时删除 `cc` 和 `conmand_cache` 两个命令。

### 方法二：单独删除某个命令

`cargo install` 不支持按二进制名单独卸载，需要手动删除：

```bash
# 只删除 conmand_cache，保留 cc
rm ~/.cargo/bin/conmand_cache

# 只删除 cc，保留 conmand_cache
rm ~/.cargo/bin/cc
```

### 方法三：还原 cc 为系统 C 编译器

如果你不再需要 `cc` 命令，可以从 PATH 中移除 cargo bin 的优先级：

1. 编辑 `~/.zshrc`，删除或注释掉这一行：
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```
2. 重新加载配置：
   ```bash
   source ~/.zshrc
   ```
3. 删除本工具安装的 `cc`：
   ```bash
   rm ~/.cargo/bin/cc
   ```

此后 `cc` 会恢复为系统的 C 编译器。

## 开发

```bash
cargo test         # 运行测试
cargo check        # 检查编译
```

## License

MIT
