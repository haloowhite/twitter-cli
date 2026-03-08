---
name: twitter
description: Use x-cli for ALL Twitter/X operations — reading tweets, posting, replying, quoting, liking, retweeting, following, searching, user lookups. Invoke whenever user requests any Twitter interaction including browsing timelines, checking followers, posting content, or engaging with tweets.
---

# x-cli — Twitter/X CLI Tool

**Binary:** `x`
**Repo:** https://github.com/haloowhite/twitter-cli (private)
**Credentials:** `~/.x-cli/credentials.json`
**Cache:** `~/.x-cli/transaction_cache.json`

## Setup

```bash
# Install pre-built binary (macOS ARM64)
curl -L https://github.com/haloowhite/twitter-cli/releases/latest/download/x-macos-arm64 -o /usr/local/bin/x && chmod +x /usr/local/bin/x

# Or build from source
cd /Users/white/PycharmProjects/twitter-cli && cargo build --release
# Binary at: target/release/x
```

## Authentication

Must authenticate before any other command.

```bash
# Extract cookies from browser (recommended — gets auth_token + ct0 + extra cookies)
x auth --browser chrome    # also: firefox, edge, safari

# Or provide auth_token directly (auto-generates csrf token)
x auth --token "your_auth_token"
```

**Credentials file format** (`~/.x-cli/credentials.json`):
```json
{
  "auth_token": "hex string from browser cookie",
  "ct0": "csrf token (32-char hex)",
  "extra_cookies": "guest_id=xxx; kdt=xxx; twid=xxx (optional, for write ops)"
}
```

If write operations fail with error 226, manually add `extra_cookies` from browser DevTools (Application > Cookies > x.com, copy all cookie values).

---

## Command Reference

**Convention:** All commands output JSON to **stdout**, status/errors to **stderr**. Pipe stdout to `jq` for field extraction. User arguments accept either **screen_name** (e.g. `elonmusk`) or **numeric user_id** (e.g. `44196397`) — auto-resolved internally.

### Read Operations

#### `x me` — Current authenticated user info

```bash
x me
```

**Output:** Full user object. Extract fields:
```bash
x me | jq '.screen_name'        # your handle
x me | jq '.id_str'             # your user ID
x me | jq '.followers_count'    # follower count
```

---

#### `x user <screen_name>` — Look up user by handle

```bash
x user elonmusk
```

**Output:** User object at `data.user.result`. Key fields:
- `.data.user.result.rest_id` — numeric user ID
- `.data.user.result.legacy.screen_name` — handle
- `.data.user.result.legacy.name` — display name
- `.data.user.result.legacy.description` — bio
- `.data.user.result.legacy.followers_count` — follower count
- `.data.user.result.legacy.friends_count` — following count
- `.data.user.result.legacy.statuses_count` — tweet count
- `.data.user.result.legacy.created_at` — account creation date
- `.data.user.result.is_blue_verified` — verification status

```bash
# Get user ID from handle
x user elonmusk | jq -r '.data.user.result.rest_id'

# Get bio
x user elonmusk | jq -r '.data.user.result.legacy.description'
```

**Errors:**
- User not found → `data.user.result` is null

---

#### `x tweets <user> [--limit N]` — User's timeline tweets

```bash
x tweets elonmusk                 # default 20 tweets
x tweets elonmusk --limit 50      # up to 50
x tweets 44196397 --limit 10      # by user ID
```

**Output:** Raw GraphQL timeline response. Tweet entries nested in:
`data.user.result.timeline_v2.timeline.instructions[].entries[]`

Each tweet entry has:
- `.content.itemContent.tweet_results.result.rest_id` — tweet ID
- `.content.itemContent.tweet_results.result.legacy.full_text` — text
- `.content.itemContent.tweet_results.result.legacy.favorite_count` — likes
- `.content.itemContent.tweet_results.result.legacy.retweet_count` — retweets
- `.content.itemContent.tweet_results.result.legacy.reply_count` — replies
- `.content.itemContent.tweet_results.result.legacy.created_at` — timestamp
- `.content.itemContent.tweet_results.result.legacy.entities.urls[]` — URLs
- `.content.itemContent.tweet_results.result.core.user_results.result.legacy.screen_name` — author

**Note:** Response includes cursor entries (type `TimelineTimelineCursor`) for pagination — these are not tweets. Filter by `entryId` starting with `tweet-`.

```bash
# Extract tweet texts
x tweets elonmusk --limit 5 | jq '[.data.user.result.timeline_v2.timeline.instructions[].entries[] | select(.entryId | startswith("tweet-")) | .content.itemContent.tweet_results.result.legacy.full_text]'

# Get tweet IDs
x tweets elonmusk --limit 5 | jq '[.data.user.result.timeline_v2.timeline.instructions[].entries[] | select(.entryId | startswith("tweet-")) | .content.itemContent.tweet_results.result.rest_id]'
```

---

#### `x replies <user> [--limit N]` — User's replies

```bash
x replies elonmusk --limit 20
```

**Output:** Same structure as `tweets` but includes replies. Uses `UserTweetsAndReplies` endpoint.

---

#### `x followers <user> [--limit N]` — User's followers

```bash
x followers elonmusk --limit 100
```

**Output:** Raw GraphQL response. User entries at:
`data.user.result.timeline.timeline.instructions[].entries[]`

Each user entry:
- `.content.itemContent.user_results.result.rest_id` — user ID
- `.content.itemContent.user_results.result.legacy.screen_name` — handle
- `.content.itemContent.user_results.result.legacy.name` — display name
- `.content.itemContent.user_results.result.legacy.followers_count` — their followers

```bash
# List follower handles
x followers someuser --limit 50 | jq '[.data.user.result.timeline.timeline.instructions[].entries[] | select(.entryId | startswith("user-")) | .content.itemContent.user_results.result.legacy.screen_name]'
```

**Requires:** x-client-transaction-id (auto-generated). May 404 intermittently — tool auto-retries up to 5 times.

---

#### `x following <user> [--limit N]` — Who user follows

```bash
x following elonmusk --limit 100
```

**Output:** Same structure as `followers`.

**Requires:** x-client-transaction-id. Same retry logic.

---

#### `x search "<query>" [--limit N]` — Search tweets

```bash
x search "rust programming" --limit 30
x search "from:elonmusk AI"
x search "#bitcoin since:2026-01-01 lang:en"
```

**Supported search operators** (Twitter native):
- `from:username` — tweets by user
- `to:username` — tweets to user
- `@username` — mentioning user
- `#hashtag` — with hashtag
- `lang:en` — language filter
- `since:YYYY-MM-DD` — after date
- `until:YYYY-MM-DD` — before date
- `min_retweets:N` — minimum retweets
- `min_faves:N` — minimum likes
- `min_replies:N` — minimum replies
- `filter:links` — only tweets with links
- `filter:media` — only tweets with media
- `-filter:replies` — exclude replies
- Combine freely: `from:elonmusk AI since:2026-01-01`

**Output:** Raw GraphQL SearchTimeline response. Tweet entries at:
`data.search_by_raw_query.search_timeline.timeline.instructions[].entries[]`

```bash
# Extract search result texts
x search "rust" --limit 5 | jq '[.data.search_by_raw_query.search_timeline.timeline.instructions[].entries[] | select(.entryId | startswith("tweet-")) | .content.itemContent.tweet_results.result.legacy.full_text]'
```

**Requires:** x-client-transaction-id.

---

#### `x detail <tweet_id> [--context]` — Tweet detail

```bash
x detail 1234567890              # single tweet
x detail 1234567890 --context    # with conversation thread
```

**Without --context:** Single tweet via `TweetResultByRestId`. Output at:
`data.tweetResult.result`

```bash
# Get tweet text
x detail 1234567890 | jq -r '.data.tweetResult.result.legacy.full_text'

# Get like count
x detail 1234567890 | jq '.data.tweetResult.result.legacy.favorite_count'

# Get author handle
x detail 1234567890 | jq -r '.data.tweetResult.result.core.user_results.result.legacy.screen_name'
```

**With --context:** Full conversation via `TweetDetail`. Output at:
`data.threaded_conversation_with_injections_v2.instructions[].entries[]`

Includes parent tweets, focal tweet, and replies in conversation order.

---

### Write Operations

All write operations require valid authentication. If getting error 226 ("automated request detection"), add `extra_cookies` to credentials.json.

#### `x post "<text>"` — Post a new tweet

```bash
x post "Hello from x-cli!"
```

**Output:** CreateTweet response at:
`data.create_tweet.tweet_results.result`

```bash
# Post and get tweet ID
x post "Hello world" | jq -r '.data.create_tweet.tweet_results.result.rest_id'

# Post and get tweet URL
RESULT=$(x post "Hello world")
REST_ID=$(echo "$RESULT" | jq -r '.data.create_tweet.tweet_results.result.rest_id')
SCREEN_NAME=$(echo "$RESULT" | jq -r '.data.create_tweet.tweet_results.result.core.user_results.result.legacy.screen_name')
echo "https://x.com/$SCREEN_NAME/status/$REST_ID"
```

**Limitations:** Text only — no media/image upload support.

**Errors:**
- 226: "This request looks like it might be automated" → add extra_cookies
- 187: Duplicate tweet → you already posted this exact text
- 186: Tweet too long → over 280 characters

---

#### `x reply <tweet_id> "<text>"` — Reply to a tweet

```bash
x reply 1234567890 "Great point!"
```

**Output:** Same as `post` — CreateTweet response with the reply tweet.

```bash
# Reply and confirm
x reply 1234567890 "Nice!" | jq -r '.data.create_tweet.tweet_results.result.rest_id'
```

---

#### `x quote <tweet_id> "<text>"` — Quote tweet

```bash
x quote 1234567890 "Interesting perspective"
```

**Output:** Same as `post`.

Internally attaches `attachment_url: https://x.com/x/status/{tweet_id}` to the CreateTweet request.

---

#### `x like <tweet_id>` — Like a tweet

```bash
x like 1234567890
```

**Output:** FavoriteTweet response.

```bash
x like 1234567890 | jq '.data.favorite_tweet'
# Returns "Done" on success
```

**Idempotent:** Liking an already-liked tweet returns success.

---

#### `x unlike <tweet_id>` — Unlike a tweet

```bash
x unlike 1234567890
```

**Output:** UnfavoriteTweet response.

```bash
x unlike 1234567890 | jq '.data.unfavorite_tweet'
# Returns "Done" on success
```

---

#### `x retweet <tweet_id>` — Retweet

```bash
x retweet 1234567890
```

**Output:** CreateRetweet response.

```bash
x retweet 1234567890 | jq '.data.create_retweet.retweet_results.result.rest_id'
```

**Error:** Retweeting an already-retweeted tweet may return an error.

---

#### `x unretweet <tweet_id>` — Undo retweet

```bash
x unretweet 1234567890
```

**Output:** DeleteRetweet response.

---

#### `x follow <user>` — Follow a user

```bash
x follow elonmusk
x follow 44196397
```

**Output:** REST 1.1 user object (the followed user's info).

```bash
x follow elonmusk | jq '.screen_name'
```

**Accepts:** screen_name or user_id (auto-resolved).
**Idempotent:** Following someone you already follow returns success.

---

#### `x unfollow <user>` — Unfollow a user

```bash
x unfollow elonmusk
```

**Output:** REST 1.1 user object.
**Idempotent:** Unfollowing someone you don't follow returns success.

---

## User Resolution

All commands accepting `<user>` auto-resolve the identifier:

- **All digits** (e.g. `44196397`) → treated as numeric user ID directly
- **Contains non-digits** (e.g. `elonmusk`) → calls `UserByScreenName` GraphQL endpoint to resolve to user ID

```bash
# Both equivalent:
x tweets elonmusk --limit 5
x tweets 44196397 --limit 5
```

**Error on resolution failure:**
```
Could not resolve user_id for @{identifier}
```

**Tip:** To get a user's numeric ID:
```bash
x user elonmusk | jq -r '.data.user.result.rest_id'
```

---

## Error Reference

| Error | Cause | Fix |
|-------|-------|-----|
| `No credentials found at ~/.x-cli/credentials.json. Run 'x auth' first.` | Not authenticated | Run `x auth --browser chrome` |
| `auth_token cookie not found. Are you logged in to X?` | Browser not logged in | Log in to x.com in browser first |
| `Unsupported browser: {name}` | Invalid browser arg | Use: chrome, firefox, edge, safari |
| `Could not resolve user_id for @{name}` | Screen name not found | Check spelling, user may not exist |
| HTTP 226 on write ops | Automated request detection | Add full `extra_cookies` to credentials.json |
| HTTP 404 on followers/following/search | Transaction ID issue | Auto-retried; if persistent, delete `~/.x-cli/transaction_cache.json` |
| HTTP 429 | Rate limited | Wait and retry later (no auto-retry) |
| Error code 64 | Account suspended | Use different account |
| Error code 326 | Account banned | Use different account |
| Error code 187 | Duplicate tweet | Change tweet text |
| Error code 186 | Tweet too long | Keep under 280 characters |
| `Invalid credentials file` | Corrupted JSON | Re-run `x auth` |
| `Failed to read response body` | Network error | Retry command |
| Empty `tweet_results: {}` | Cookie expired or invalid | Re-run `x auth` |

---

## Retry & Transaction ID Behavior

### Automatic Retries
- **GET requests (rquest):** Up to 5 retries on 404
- **GET requests (reqwest fallback):** Up to 3 retries on 404
- **POST requests:** Up to 3 retries on 404, with 1-second delay between retries
- **No retry on 429** (rate limit) — must wait manually

### Transaction ID
- Required for: Followers, Following, Search, Replies, all POST endpoints
- Auto-generated from x.com homepage HTML + JS
- Cached at `~/.x-cli/transaction_cache.json` for 1 hour
- If stale, auto-refreshes on next request
- If broken, delete cache file: `rm ~/.x-cli/transaction_cache.json`

### Dual HTTP Client
- **Primary:** rquest (Chrome 136 TLS fingerprint emulation)
- **Fallback:** reqwest (standard TLS)
- If rquest fails, automatically falls back to reqwest

---

## Common Agent Workflows

### Workflow: Post and verify

```bash
RESULT=$(x post "My tweet text")
TWEET_ID=$(echo "$RESULT" | jq -r '.data.create_tweet.tweet_results.result.rest_id')
echo "Posted tweet: $TWEET_ID"

# Verify
x detail "$TWEET_ID" | jq -r '.data.tweetResult.result.legacy.full_text'
```

### Workflow: Reply to someone's latest tweet

```bash
# Get their latest tweet ID
RESPONSE=$(x tweets targetuser --limit 1)
TWEET_ID=$(echo "$RESPONSE" | jq -r '[.data.user.result.timeline_v2.timeline.instructions[].entries[] | select(.entryId | startswith("tweet-"))][0].content.itemContent.tweet_results.result.rest_id')

# Reply
x reply "$TWEET_ID" "Great post!"
```

### Workflow: Like all tweets from search

```bash
x search "interesting topic" --limit 10 | jq -r '[.data.search_by_raw_query.search_timeline.timeline.instructions[].entries[] | select(.entryId | startswith("tweet-")) | .content.itemContent.tweet_results.result.rest_id] | .[]' | while read id; do
  x like "$id"
  echo "Liked $id" >&2
done
```

### Workflow: Get user info then follow

```bash
# Look up user
USER_DATA=$(x user targethandle)
USER_ID=$(echo "$USER_DATA" | jq -r '.data.user.result.rest_id')
FOLLOWER_COUNT=$(echo "$USER_DATA" | jq '.data.user.result.legacy.followers_count')
echo "User $USER_ID has $FOLLOWER_COUNT followers"

# Follow
x follow "$USER_ID"
```

### Workflow: Monitor a user's new tweets

```bash
# Get latest tweet ID
LAST_ID=$(x tweets targetuser --limit 1 | jq -r '[.data.user.result.timeline_v2.timeline.instructions[].entries[] | select(.entryId | startswith("tweet-"))][0].content.itemContent.tweet_results.result.rest_id')
echo "Latest tweet: $LAST_ID"
```

### Workflow: Get conversation thread

```bash
# Get full conversation including parent and replies
x detail 1234567890 --context | jq '[.data.threaded_conversation_with_injections_v2.instructions[].entries[] | select(.entryId | startswith("conversationthread-") or startswith("tweet-"))]'
```

### Workflow: Check if user follows you

```bash
MY_ID=$(x me | jq -r '.id_str')
x followers "$MY_ID" --limit 200 | jq -r '[.data.user.result.timeline.timeline.instructions[].entries[] | select(.entryId | startswith("user-")) | .content.itemContent.user_results.result.legacy.screen_name] | .[]' | grep -q "targetuser" && echo "Yes" || echo "No"
```

---

## Limitations

- **No media upload** — text-only posts, replies, quotes
- **No DM support** — no direct messaging
- **No bookmark support** — can't bookmark/unbookmark
- **No list management** — can't create/manage Twitter lists
- **No notification reading** — can't check notifications
- **No thread posting** — must post and reply sequentially to create threads
- **No poll creation** — text-only posts
- **Pagination internal only** — no cursor-based manual pagination exposed
- **Rate limits not visible** — no way to check remaining API quota
- **Single account** — one credentials file, one account at a time

## Creating a Thread (Workaround)

Since there's no native thread support, chain posts manually:

```bash
# Post first tweet
FIRST=$(x post "Thread 1/3: ...")
FIRST_ID=$(echo "$FIRST" | jq -r '.data.create_tweet.tweet_results.result.rest_id')

# Reply to create thread
SECOND=$(x reply "$FIRST_ID" "2/3: ...")
SECOND_ID=$(echo "$SECOND" | jq -r '.data.create_tweet.tweet_results.result.rest_id')

# Continue thread
x reply "$SECOND_ID" "3/3: ..."
```

---

## Files

| Path | Purpose | Auto-managed |
|------|---------|-------------|
| `~/.x-cli/credentials.json` | Auth credentials (auth_token, ct0, extra_cookies) | Created by `x auth` |
| `~/.x-cli/transaction_cache.json` | x-client-transaction-id cache (1h TTL) | Auto-created/refreshed |
