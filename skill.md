---
name: twitter
description: Use x-cli for ALL Twitter/X operations — reading tweets, posting, replying, quoting, liking, retweeting, following, searching, user lookups. Invoke whenever user requests any Twitter interaction.
---

# x-cli — Twitter/X CLI Tool

**Binary:** `x`
**Repo:** https://github.com/haloowhite/twitter-cli (private)
**Credentials:** `~/.x-cli/credentials.json`

## Setup

```bash
# One-line install (auto-detects OS/arch)
curl -fsSL https://raw.githubusercontent.com/haloowhite/twitter-cli/main/install.sh | bash

# Or build from source
git clone https://github.com/haloowhite/twitter-cli.git && cd twitter-cli
cargo build --release  # binary at target/release/x
```

## Authentication

**IMPORTANT FOR AGENTS**: Before executing ANY x-cli command, you MUST first check if credentials exist. If not, you MUST proactively guide the user through the authentication process step by step. Do NOT assume credentials are configured. Do NOT skip this step.

### Step 0: Check if already authenticated

```bash
x me 2>/dev/null && echo "AUTH_OK" || echo "AUTH_NEEDED"
```

If `AUTH_OK`, skip to [Command Reference](#command-reference).
If `AUTH_NEEDED`, proceed to guide the user:

### Step 1: Agent guides user to obtain cookies

**You (the agent) MUST send the following message to the user, choosing the appropriate version based on their device:**

---

**If user has a desktop/laptop computer:**

> 我需要你的 Twitter 登录凭证来帮你操作。请按以下步骤获取：
>
> 1. 用 Chrome/Edge/Firefox 打开 https://x.com（确保已登录）
> 2. 按 `F12` 打开开发者工具
> 3. 点击顶部的 **Network**（网络）标签
> 4. 在页面上随意点击刷新或浏览，让网络请求出现
> 5. 在请求列表中，点击任意一个发往 `x.com` 的请求
> 6. 在右侧找到 **Request Headers**（请求头）
> 7. 找到 `Cookie:` 这一行，**右键 → 复制值**
> 8. 把复制的完整 Cookie 字符串发给我
>
> ⚠️ 这个 Cookie 包含你的登录信息，请不要分享给其他人。我只会在本机使用它来配置 x-cli。

---

**If user only has a mobile phone:**

> 我需要你的 Twitter 登录凭证来帮你操作。请按以下步骤获取：
>
> **安卓手机：**
> 1. 用 Chrome 打开 https://x.com（确保已登录）
> 2. 在地址栏中手动输入（注意：必须手打，不能粘贴 `javascript:` 前缀）：
>    `javascript:void(prompt('Cookie',document.cookie))`
> 3. 会弹出一个对话框，里面是你的 Cookie 字符串
> 4. 长按全选 → 复制 → 发给我
>
> **iPhone：**
> 1. 用 Safari 打开 https://x.com（确保已登录）
> 2. 如果你有"快捷指令"App，可以创建一个快捷指令：
>    - 动作：在 Safari 中运行 JavaScript
>    - 代码：`completion(document.cookie)`
>    - 运行后复制输出发给我
> 3. 或者：安装一个抓包工具（如 Stream、HTTP Catcher），打开 Twitter App 浏览一下，找到请求头中的 Cookie 值发给我
>
> ⚠️ 这个 Cookie 包含你的登录信息，请不要分享给其他人。

---

### Step 2: Agent parses cookie string and configures x-cli

When the user provides the cookie string, the agent MUST:

1. **Extract `auth_token` and `ct0`** from the cookie string:

```bash
# Example cookie string from user:
# "guest_id=v1%3A...; ct0=abc123def456...; auth_token=a1b2c3d4e5f6...; twid=u%3D123..."

# Parse with shell:
FULL_COOKIE="<paste user's cookie string here>"
AUTH_TOKEN=$(echo "$FULL_COOKIE" | grep -oE 'auth_token=[a-f0-9]+' | cut -d= -f2)
CT0=$(echo "$FULL_COOKIE" | grep -oE 'ct0=[a-f0-9]+' | cut -d= -f2)
echo "auth_token: $AUTH_TOKEN"
echo "ct0: $CT0"
```

2. **Write credentials file with full cookies** (for maximum compatibility):

```bash
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

3. **Verify authentication**:

```bash
x me
```

If successful, tell the user their account is connected and show their handle.
If failed, ask the user to re-check their cookie (may have expired or been copied incorrectly).

### Step 3: Handle common auth issues

| Symptom | Agent action |
|---------|-------------|
| `x me` returns error | Cookie expired — guide user to re-extract |
| Read works, write returns 226 | Need full cookies — ask user for complete Cookie header from Network tab |
| `ct0` not found in cookie | OK — tool auto-generates one |
| User says "I changed my password" | All old cookies invalidated — guide re-extraction |

### Credentials file format

```json
{
  "auth_token": "40-char hex (REQUIRED)",
  "ct0": "csrf token (auto-generated if empty)",
  "extra_cookies": "full cookie string (recommended for write ops)"
}
```

Location: `~/.x-cli/credentials.json` (permissions: 600)

### Security

- `auth_token` is equivalent to a login session — **never log it, print it, or share it**
- Valid for months; invalidated by password change
- Agent should treat cookie values as secrets (do not echo to stdout unnecessarily)

---

## Output Format

All commands output **clean, extracted JSON** to stdout. No raw GraphQL wrappers.

### Tweet Object

```json
{
  "id": "2030159267689632121",
  "url": "https://x.com/elonmusk/status/2030159267689632121",
  "text": "Only Grok speaks the truth...",
  "created_at": "Sat Mar 07 05:51:02 +0000 2026",
  "lang": "en",
  "author": {
    "id": "44196397",
    "handle": "elonmusk",
    "name": "Elon Musk"
  },
  "stats": {
    "views": 25806169,
    "likes": 58482,
    "retweets": 10789,
    "replies": 10750,
    "quotes": 967,
    "bookmarks": 4533
  },
  "referenced_tweet": {        // only present if quote or retweet
    "id": "2030151922968318104",
    "type": "quote"            // "quote" or "retweet"
  },
  "in_reply_to_id": null       // only present if reply
}
```

### User Object

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
  "profile_image_url": "https://pbs.twimg.com/profile_images/.../photo_normal.jpg"
}
```

### Action Result (like, unlike, retweet, unretweet, follow, unfollow)

```json
{
  "success": true,
  "action": "like",
  "id": "2030159267689632121"   // optional, present for retweet/follow
}
```

---

## Command Reference

User arguments accept **screen_name** (e.g. `elonmusk`) or **numeric user_id** (e.g. `44196397`).
Default `--limit` is 20. Status/errors go to stderr.

### Read Operations

#### `x me`

```bash
x me
x me | jq '.screen_name'     # → "fineandthx"
x me | jq '.id'              # → "1998996094701940737"
```

Returns: **User object**

---

#### `x user <screen_name>`

```bash
x user elonmusk
x user elonmusk | jq '.id'               # → "44196397"
x user elonmusk | jq '.followers_count'   # → 236140595
x user elonmusk | jq '.description'       # → bio text
```

Returns: **User object**

---

#### `x tweets <user> [--limit N]`

```bash
x tweets elonmusk
x tweets elonmusk --limit 50
x tweets 44196397 --limit 10
```

Returns: **Array of Tweet objects**

```bash
# Get all tweet texts
x tweets elonmusk --limit 5 | jq '.[].text'

# Get IDs
x tweets elonmusk --limit 5 | jq '.[].id'

# Get first tweet URL
x tweets elonmusk --limit 1 | jq -r '.[0].url'

# Filter only original tweets (no retweets/quotes)
x tweets elonmusk --limit 20 | jq '[.[] | select(.referenced_tweet == null)]'

# Get most liked tweet
x tweets elonmusk --limit 20 | jq 'sort_by(.stats.likes) | last'
```

---

#### `x replies <user> [--limit N]`

```bash
x replies elonmusk --limit 20
```

Returns: **Array of Tweet objects** (includes replies)

---

#### `x followers <user> [--limit N]`

```bash
x followers elonmusk --limit 100
x followers elonmusk --limit 50 | jq '.[].screen_name'
```

Returns: **Array of User objects**

---

#### `x following <user> [--limit N]`

```bash
x following elonmusk --limit 100
x following elonmusk | jq '[.[] | select(.followers_count > 1000000)] | length'
```

Returns: **Array of User objects**

---

#### `x search "<query>" [--limit N]`

```bash
x search "rust programming" --limit 30
x search "from:elonmusk AI"
x search "#bitcoin since:2026-01-01 lang:en"
```

**Search operators:**
- `from:user` / `to:user` / `@user`
- `#hashtag`
- `lang:en`
- `since:YYYY-MM-DD` / `until:YYYY-MM-DD`
- `min_retweets:N` / `min_faves:N` / `min_replies:N`
- `filter:links` / `filter:media` / `-filter:replies`

Returns: **Array of Tweet objects**

```bash
# Search and extract top tweets by likes
x search "AI safety" --limit 20 | jq 'sort_by(.stats.likes) | reverse | .[:5] | .[].url'
```

---

#### `x detail <tweet_id> [--context]`

```bash
x detail 2030159267689632121
x detail 2030159267689632121 --context
```

**Without --context:** Returns **single Tweet object**
**With --context:** Returns **Array of Tweet objects** (conversation thread)

```bash
# Get tweet text
x detail 2030159267689632121 | jq -r '.text'

# Get conversation thread texts
x detail 2030159267689632121 --context | jq '.[].text'
```

---

### Write Operations

#### `x post "<text>"`

```bash
x post "Hello from x-cli!"
```

Returns: **Tweet object** (the created tweet)

```bash
# Post and get tweet ID/URL
x post "Hello world" | jq -r '.id'
x post "Hello world" | jq -r '.url'
```

---

#### `x reply <tweet_id> "<text>"`

```bash
x reply 2030159267689632121 "Great point!"
```

Returns: **Tweet object** (the reply)

---

#### `x quote <tweet_id> "<text>"`

```bash
x quote 2030159267689632121 "Interesting perspective"
```

Returns: **Tweet object** (the quote tweet)

---

#### `x like <tweet_id>` / `x unlike <tweet_id>`

```bash
x like 2030159267689632121
x unlike 2030159267689632121
```

Returns: **Action result** `{"success": true, "action": "like"}`

---

#### `x retweet <tweet_id>` / `x unretweet <tweet_id>`

```bash
x retweet 2030159267689632121
x unretweet 2030159267689632121
```

Returns: **Action result** `{"success": true, "action": "retweet", "id": "..."}`

---

#### `x follow <user>` / `x unfollow <user>`

```bash
x follow elonmusk
x unfollow elonmusk
```

Returns: **Action result** `{"success": true, "action": "follow", "id": "44196397"}`

---

## Agent Workflows

### Post and verify

```bash
TWEET=$(x post "My tweet text")
TWEET_ID=$(echo "$TWEET" | jq -r '.id')
TWEET_URL=$(echo "$TWEET" | jq -r '.url')
echo "Posted: $TWEET_URL"
```

### Reply to someone's latest tweet

```bash
TWEET_ID=$(x tweets targetuser --limit 1 | jq -r '.[0].id')
x reply "$TWEET_ID" "Nice post!"
```

### Create a thread

```bash
T1=$(x post "Thread 1/3: First point" | jq -r '.id')
T2=$(x reply "$T1" "2/3: Second point" | jq -r '.id')
x reply "$T2" "3/3: Final point"
```

### Like all search results

```bash
x search "interesting topic" --limit 5 | jq -r '.[].id' | while read id; do
  x like "$id"
done
```

### Get user info then follow

```bash
x user targethandle | jq '{id, screen_name, followers_count}'
x follow targethandle
```

### Find most popular tweets

```bash
x tweets elonmusk --limit 20 | jq 'sort_by(.stats.likes) | reverse | .[:3] | .[] | {url, likes: .stats.likes, text: .text[:80]}'
```

### Get conversation thread

```bash
x detail 1234567890 --context | jq '.[] | {author: .author.handle, text: .text[:100]}'
```

### Check if user follows back

```bash
MY_ID=$(x me | jq -r '.id')
x followers "$MY_ID" --limit 200 | jq -r '.[].screen_name' | grep -q "targetuser" && echo "Yes" || echo "No"
```

---

## Error Reference

| Error | Cause | Fix |
|-------|-------|-----|
| `No credentials found` | Not authenticated | `x auth --browser chrome` |
| `auth_token cookie not found` | Browser not logged in | Log in to x.com first |
| HTTP 226 | Automated detection | Add `extra_cookies` to credentials.json |
| HTTP 404 | Transaction ID issue | Delete `~/.x-cli/transaction_cache.json`, retry |
| HTTP 429 | Rate limited | Wait and retry |
| Error 64 | Account suspended | Use different account |
| Error 187 | Duplicate tweet | Change text |
| Error 186 | Tweet too long | Keep under 280 chars |
| Empty response | Cookie expired | Re-run `x auth` |

---

## Limitations

- **Text only** — no media/image upload
- **No DMs** — no direct messaging
- **No bookmarks** — can't bookmark/unbookmark
- **No lists** — can't manage Twitter lists
- **No notifications** — can't read notifications
- **No polls** — can't create polls
- **Single account** — one credentials file at a time

---

## Files

| Path | Purpose |
|------|---------|
| `~/.x-cli/credentials.json` | Auth credentials |
| `~/.x-cli/transaction_cache.json` | Transaction ID cache (1h TTL, auto-managed) |
