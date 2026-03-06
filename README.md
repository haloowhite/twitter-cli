# twitter-cli

Twitter/X CLI 工具，使用 Rust 编写，编译为单二进制文件分发。

## 功能

- 从浏览器自动提取 cookie 进行认证（Chrome/Firefox/Edge/Safari）
- 读取推文、回复、关注列表、粉丝列表
- 搜索推文
- 发推、回复、引用推文
- 点赞/取消点赞、转推/取消转推
- 关注/取消关注
- 所有输出为 JSON 格式，方便管道处理

## 安装

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/twitter-cli`。

## 快速开始

```bash
# 1. 从浏览器提取认证信息
twitter-cli auth --browser chrome

# 2. 查看用户推文
twitter-cli tweets 12345678 --limit 10

# 3. 搜索推文
twitter-cli search "rust lang" --limit 20

# 4. 发推
twitter-cli post "Hello from CLI"
```

## 完整用法

详见 [skill.md](skill.md)。

## 技术参考

API 调用逻辑参考 [heimdall](../heimdall) 项目的 Python 实现。
