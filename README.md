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

## 使用

```bash
cc ls -la          # 执行 ls -la 并记录到历史
cc cargo test      # 执行 cargo test 并记录
```

## 开发

```bash
cargo test         # 运行测试
cargo check        # 检查编译
```

## License

MIT
