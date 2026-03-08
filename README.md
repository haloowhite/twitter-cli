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

凭证保存在 `~/.x-cli/credentials.json`，格式：

```json
{
  "auth_token": "你的auth_token",
  "ct0": "自动生成的csrf_token",
  "extra_cookies": "完整cookie字符串（写操作必须，否则发推会报226错误）"
}
```

### 场景一：本地电脑有浏览器登录（最简单）

```bash
x auth --browser chrome   # 支持: chrome, firefox, edge, safari
```

### 场景二：Agent 在云端，推特登录在本地电脑

**步骤 1：在本地电脑获取完整 Cookie**

打开 Chrome/Edge/Firefox，登录 x.com，然后：

1. 按 `F12` 打开 DevTools
2. 点击 **Network**（网络）标签
3. 在页面上随意刷新或浏览，让请求出现
4. 点击任意一个发往 `x.com` 的请求
5. 在右侧找到 **Request Headers** → **Cookie:** 这一行
6. **右键 → 复制值**，得到完整的 Cookie 字符串

**步骤 2：从 Cookie 中提取字段并配置**

```bash
# 从完整 Cookie 字符串中提取 auth_token 和 ct0
FULL_COOKIE="<粘贴完整Cookie字符串>"
AUTH_TOKEN=$(echo "$FULL_COOKIE" | grep -oE 'auth_token=[a-f0-9]+' | cut -d= -f2)
CT0=$(echo "$FULL_COOKIE" | grep -oE 'ct0=[a-f0-9]+' | cut -d= -f2)

mkdir -p ~/.x-cli
cat > ~/.x-cli/credentials.json << CREDENTIALS
{
  "auth_token": "$AUTH_TOKEN",
  "ct0": "$CT0",
  "extra_cookies": "$FULL_COOKIE"
}
CREDENTIALS
chmod 600 ~/.x-cli/credentials.json
```

> **⚠️ 重要**：写操作（发推、回复、引用转推）**必须**提供完整 cookie（`extra_cookies` 字段）。只提供 `auth_token` + `ct0` 会导致 226 错误（"looks like automated behavior"）。读操作（浏览推文、搜索等）不受影响。

### 场景三：只有手机，推特登录在移动端，Agent 在云端

> **⚠️ 注意**：以下方法获取的 cookie 可能不完整（手机浏览器限制），可能只能支持读操作。如需发推等写操作，建议用电脑浏览器获取完整 Cookie（场景二）。

**方法 A：手机浏览器 + JavaScript**

1. 用手机浏览器（Chrome/Safari）打开 https://x.com 并登录
2. 在地址栏输入以下内容并访问（需要手动输入 `javascript:` 前缀，不能粘贴）：

```
javascript:void(document.title=document.cookie)
```

3. 页面标题会变成 cookie 字符串，**复制完整的 cookie 字符串**
4. 将完整字符串发给 Agent，Agent 按场景二的步骤 2 配置（包含 `extra_cookies`）

**方法 B：手机浏览器 DevTools（Android Chrome）**

1. 手机 Chrome 打开 x.com 并登录
2. 电脑 Chrome 打开 `chrome://inspect/#devices`，连接手机
3. 在远程调试界面的 Console 中执行：

```javascript
document.cookie
```

4. 复制输出的**完整 cookie 字符串**

**方法 C：通过请求抓包（iOS/Android 通用）**

1. 安装抓包工具（如 Stream/HTTP Catcher/Charles）
2. 打开 X/Twitter App，随意浏览
3. 在抓包记录中找到发往 `api.x.com` 或 `x.com` 的请求
4. 复制请求头中的**完整 `Cookie` 值**

**方法 D：直接传配置文件**

在任意一台已认证的设备上导出配置，传到云端：

```bash
# 已认证设备上
cat ~/.x-cli/credentials.json
# 复制输出内容

# 云端服务器上
mkdir -p ~/.x-cli
cat > ~/.x-cli/credentials.json << 'EOF'
（粘贴刚才复制的 JSON 内容）
EOF
chmod 600 ~/.x-cli/credentials.json
```

### 注意事项

- **写操作必须提供完整 cookie**（`extra_cookies` 字段），否则会报 226 错误
- `auth_token` 是你的登录凭证，**请勿泄露给他人**
- `auth_token` 有效期较长（通常数月），过期后需重新获取
- 修改密码会使所有 `auth_token` 失效
- `ct0` 可以不提供，工具会自动生成随机值
- 建议对 `credentials.json` 设置 `chmod 600` 权限

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
| 写操作 226 错误 | **必须提供完整 cookie**：从浏览器 Network 标签复制完整 Cookie 字符串，填入 credentials.json 的 `extra_cookies` 字段 |
| 404 错误 | 删除 `~/.x-cli/transaction_cache.json` 重试 |

## 完整用法

详见 [skill.md](skill.md)。

## 技术说明

- rquest (Chrome TLS 指纹) + reqwest (备用) 双 HTTP 客户端
- 自动提取 x-client-transaction-id
- 输出自动提取关键字段，去除 GraphQL 包装（减少 98% 数据量）
- API 逻辑参考 [heimdall](https://github.com/cyberconnecthq/heimdall)
