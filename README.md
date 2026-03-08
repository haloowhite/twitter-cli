# x-cli

X (Twitter) 命令行工具，使用 Rust 编写，编译为单二进制文件 `x`。支持完整的推特读写操作：浏览推文、发推、回复、引用、点赞、转推、关注等。

## 安装

### 方式一：下载预编译二进制（推荐）

从 [GitHub Releases](https://github.com/haloowhite/twitter-cli/releases) 下载适合你系统的二进制文件：

```bash
# macOS (Apple Silicon)
curl -L https://github.com/haloowhite/twitter-cli/releases/latest/download/x-macos-arm64 -o x
chmod +x x
sudo mv x /usr/local/bin/
```

### 方式二：从源码编译

需要 Rust 工具链（[安装 Rust](https://rustup.rs/)）：

```bash
git clone https://github.com/haloowhite/twitter-cli.git
cd twitter-cli
cargo build --release
# 二进制文件位于 target/release/x
sudo cp target/release/x /usr/local/bin/
```

验证安装：

```bash
x --version
x --help
```

## 认证

使用前需要先设置 Twitter 认证信息。凭证保存在 `~/.x-cli/credentials.json`（权限 600）。

### 从浏览器提取 Cookie（推荐）

确保你已在浏览器中登录 X/Twitter，然后运行：

```bash
x auth --browser chrome
# 支持的浏览器: chrome, firefox, edge, safari
```

### 手动提供 auth_token

如果浏览器提取不可用，可以手动提供 auth_token（从浏览器 DevTools > Application > Cookies 中获取）：

```bash
x auth --token "你的auth_token"
```

这会自动生成 csrf token。

### 高级：使用完整 Cookie

如果遇到某些写操作的问题，可以手动编辑 `~/.x-cli/credentials.json` 添加完整的 cookie 字符串：

```json
{
  "auth_token": "xxx",
  "ct0": "xxx",
  "extra_cookies": "guest_id=xxx; kdt=xxx; twid=xxx"
}
```

## 命令参考

所有命令的输出为 JSON 格式（stdout），状态信息输出到 stderr。支持通过 screen name（如 `elonmusk`）或 user ID 指定用户。

### 读取操作

#### 查看当前认证用户

```bash
x me
```

#### 查看用户信息

```bash
x user elonmusk
```

#### 获取用户推文

```bash
x tweets elonmusk
x tweets elonmusk --limit 50
x tweets 44196397 --limit 10    # 也可以用 user ID
```

#### 获取用户回复

```bash
x replies elonmusk --limit 20
```

#### 获取关注列表

```bash
x following elonmusk --limit 100
```

#### 获取粉丝列表

```bash
x followers elonmusk --limit 100
```

#### 搜索推文

```bash
x search "rust programming" --limit 30
x search "from:elonmusk AI"
```

#### 查看推文详情

```bash
x detail 1234567890              # 单条推文
x detail 1234567890 --context    # 包含对话上下文
```

### 写操作

#### 发推

```bash
x post "Hello from x-cli!"
```

#### 回复推文

```bash
x reply 1234567890 "Great tweet!"
```

#### 引用推文

```bash
x quote 1234567890 "Interesting take"
```

#### 点赞 / 取消点赞

```bash
x like 1234567890
x unlike 1234567890
```

#### 转推 / 取消转推

```bash
x retweet 1234567890
x unretweet 1234567890
```

#### 关注 / 取消关注

```bash
x follow elonmusk
x unfollow elonmusk
```

## 与其他工具配合

所有输出为 JSON，可方便地与 `jq` 等工具配合使用：

```bash
# 获取用户最近推文的文本内容
x tweets elonmusk --limit 5 | jq '.[].text'

# 获取用户 ID
x user elonmusk | jq '.id_str'

# 获取推文的点赞数
x detail 1234567890 | jq '.favorite_count'
```

## 文件说明

| 路径 | 说明 |
|------|------|
| `~/.x-cli/credentials.json` | 认证凭证（auth_token, ct0, extra_cookies） |
| `~/.x-cli/transaction_cache.json` | x-client-transaction-id 缓存（自动管理） |

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| `No credentials found` | 运行 `x auth --browser chrome` 或 `x auth --token <token>` |
| `auth_token cookie not found` | 确保已在浏览器中登录 X/Twitter |
| 读取操作 404 | 某些端点需要 x-client-transaction-id，工具会自动获取和缓存 |
| 写操作返回 226 | 尝试使用 `extra_cookies` 提供完整 cookie |
| 写操作返回空结果 | 可能是 cookie 过期，重新运行 `x auth` |

## 技术说明

- 使用 rquest（Chrome TLS 指纹模拟）+ reqwest（备用）双 HTTP 客户端
- 自动从 x.com 提取和缓存 x-client-transaction-id
- POST 请求自动添加 `origin`/`referer` 头以通过自动化检测
- 支持 screen name 到 user ID 的自动解析
- API 逻辑参考 [heimdall](https://github.com/cyberconnecthq/heimdall) 项目

## License

Private repository.
