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

Credentials file: `~/.x-cli/credentials.json`

```json
{
  "auth_token": "40-char hex string",
  "ct0": "csrf token (auto-generated if omitted)",
  "extra_cookies": "optional, full cookie string for write ops"
}
```

### If agent has access to local browser

```bash
x auth --browser chrome    # also: firefox, edge, safari
```

### If agent is on cloud, user's Twitter login is on local PC

Guide the user to:
1. Open x.com in Chrome/Edge/Firefox (must be logged in)
2. Press `F12` → **Application** tab → **Cookies** → `https://x.com`
3. Copy the `auth_token` value (40-char hex)
4. (Optional) Copy `ct0` value

Then on the cloud server:
```bash
x auth --token "the_auth_token_value"
```

Or write the file directly:
```bash
mkdir -p ~/.x-cli && cat > ~/.x-cli/credentials.json << 'CREDENTIALS'
{"auth_token":"PASTE_HERE","ct0":"PASTE_HERE_OR_OMIT"}
CREDENTIALS
chmod 600 ~/.x-cli/credentials.json
```

### If user only has a mobile phone (Twitter on mobile, agent on cloud)

**Method A: Mobile browser JavaScript (easiest)**
1. Open https://x.com in mobile Chrome/Safari, log in
2. Type in address bar (must manually type `javascript:` prefix, cannot paste):
   `javascript:void(document.title=document.cookie)`
3. Page title becomes the cookie string — find `auth_token=xxx` value
4. Send the value to the cloud agent

**Method B: Android Chrome remote debug**
1. Open x.com in phone Chrome
2. On desktop Chrome, open `chrome://inspect/#devices`
3. In Console run: `document.cookie.split(';').find(c=>c.trim().startsWith('auth_token=')).trim()`
4. Copy the output value

**Method C: Network capture (iOS/Android)**
1. Install a packet capture app (Stream, HTTP Catcher, Charles)
2. Open X/Twitter app, browse anything
3. Find requests to `api.x.com` or `x.com`
4. Extract `auth_token` and `ct0` from the `Cookie` request header

### After obtaining auth_token

On the cloud server, run:
```bash
x auth --token "the_auth_token"
x me  # verify authentication works
```

### Write operations returning error 226

Add full cookies to `~/.x-cli/credentials.json`:
```json
{
  "auth_token": "xxx",
  "ct0": "xxx",
  "extra_cookies": "guest_id=xxx; kdt=xxx; twid=xxx; __cf_bm=xxx"
}
```
Get the full cookie string from browser DevTools: **Network** tab → any request to x.com → copy `Cookie` header value.

### Security notes

- `auth_token` is a login credential — **never share publicly**
- Valid for months; expires when password is changed
- `ct0` is auto-generated if not provided
- Set `chmod 600` on credentials.json

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
