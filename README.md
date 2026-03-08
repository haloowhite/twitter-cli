# x-cli

X (Twitter) 命令行工具，使用 Rust 编写，编译为单二进制文件 `x`。支持完整的推特读写操作。

所有输出为**精简 JSON**（自动提取关键字段，无 GraphQL 原始包装）。

## 安装

### 方式一：一键安装（推荐）

自动检测系统和架构，下载对应二进制：

```bash
curl -fsSL https://raw.githubusercontent.com/haloowhite/twitter-cli/main/install.sh | bash
```

自定义安装目录：

```bash
INSTALL_DIR=~/bin curl -fsSL https://raw.githubusercontent.com/haloowhite/twitter-cli/main/install.sh | bash
```

### 方式二：手动下载

从 [GitHub Releases](https://github.com/haloowhite/twitter-cli/releases) 下载对应平台的包：

| 平台 | 文件 |
|------|------|
| Linux x86_64 | `x-linux-amd64.tar.gz` |
| Linux ARM64 | `x-linux-arm64.tar.gz` |
| macOS Intel | `x-darwin-amd64.tar.gz` |
| macOS Apple Silicon | `x-darwin-arm64.tar.gz` |

```bash
# 示例：Linux x86_64
curl -fsSL https://github.com/haloowhite/twitter-cli/releases/latest/download/x-linux-amd64.tar.gz | tar xz
sudo mv x /usr/local/bin/
```

### 方式三：从源码编译

需要 [Rust 工具链](https://rustup.rs/)：

```bash
git clone https://github.com/haloowhite/twitter-cli.git
cd twitter-cli
cargo build --release
sudo cp target/release/x /usr/local/bin/
```

## 认证

```bash
# 从浏览器提取 Cookie（推荐）
x auth --browser chrome   # 支持: chrome, firefox, edge, safari

# 或手动提供 auth_token
x auth --token "你的auth_token"
```

凭证保存在 `~/.x-cli/credentials.json`。

## 输出格式

所有命令输出精简 JSON。推文示例：

```json
{
  "id": "2030159267689632121",
  "url": "https://x.com/elonmusk/status/2030159267689632121",
  "text": "Only Grok speaks the truth...",
  "created_at": "Sat Mar 07 05:51:02 +0000 2026",
  "lang": "en",
  "author": { "id": "44196397", "handle": "elonmusk", "name": "Elon Musk" },
  "stats": { "views": 25806169, "likes": 58482, "retweets": 10789, "replies": 10750, "quotes": 967, "bookmarks": 4533 },
  "referenced_tweet": { "id": "2030151922968318104", "type": "quote" }
}
```

用户示例：

```json
{
  "id": "44196397",
  "screen_name": "elonmusk",
  "name": "Elon Musk",
  "description": "",
  "followers_count": 236140595,
  "following_count": 1292,
  "tweet_count": 98635,
  "is_verified": true,
  "created_at": "Tue Jun 02 20:12:29 +0000 2009",
  "profile_image_url": "https://pbs.twimg.com/..."
}
```

## 命令参考

支持 screen name（如 `elonmusk`）或 user ID。

### 读取

```bash
x me                                    # 当前用户
x user elonmusk                         # 查看用户
x timeline --limit 20                   # 首页时间线
x tweets elonmusk --limit 50            # 用户推文
x replies elonmusk --limit 20           # 用户回复
x followers elonmusk --limit 100        # 粉丝列表
x following elonmusk --limit 100        # 关注列表
x search "rust lang" --limit 30         # 搜索推文
x detail 1234567890                     # 推文详情
x detail 1234567890 --context           # 含对话上下文
```

### 紧凑模式

加 `-c` 减少输出，只保留关键字段（适合 LLM / 管道处理）：

```bash
x -c timeline                           # 紧凑时间线
x -c tweets elonmusk --limit 10         # 紧凑推文
x -c search "AI" --limit 20             # 紧凑搜索
```

### 写操作

```bash
x post "Hello from x-cli!"             # 发推
x reply 1234567890 "Great tweet!"       # 回复
x quote 1234567890 "Interesting"        # 引用
x like 1234567890                       # 点赞
x unlike 1234567890                     # 取消点赞
x retweet 1234567890                    # 转推
x unretweet 1234567890                  # 取消转推
x follow elonmusk                       # 关注
x unfollow elonmusk                     # 取消关注
```

## 与 jq 配合

```bash
x tweets elonmusk --limit 5 | jq '.[].text'                    # 推文文本
x tweets elonmusk | jq 'sort_by(.stats.likes) | last.url'      # 最热推文
x search "AI" --limit 10 | jq '[.[] | select(.stats.likes > 100)]' # 过滤
x user elonmusk | jq '.followers_count'                         # 粉丝数
```

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| `No credentials found` | 运行 `x auth --browser chrome` |
| 写操作 226 错误 | 编辑 credentials.json 添加 `extra_cookies` |
| 404 错误 | 删除 `~/.x-cli/transaction_cache.json` 重试 |

## 完整用法

详见 [skill.md](skill.md)。

## 技术说明

- rquest (Chrome TLS 指纹) + reqwest (备用) 双 HTTP 客户端
- 自动提取 x-client-transaction-id
- 输出自动提取关键字段，去除 GraphQL 包装（减少 98% 数据量）
- API 逻辑参考 [heimdall](https://github.com/cyberconnecthq/heimdall)
