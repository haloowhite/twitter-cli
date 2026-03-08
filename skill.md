---
name: twitter
description: Use x-cli for all Twitter/X operations - reading tweets, posting, replying, quoting, liking, retweeting, following, searching, and user lookups. Invoke when user requests any Twitter interaction.
---

# x-cli — Twitter/X CLI Tool

Binary: `x` (package: x-cli)
Repo: https://github.com/haloowhite/twitter-cli (private)

## Setup

### Install binary

```bash
# Download pre-built binary (macOS ARM64)
curl -L https://github.com/haloowhite/twitter-cli/releases/latest/download/x-macos-arm64 -o /usr/local/bin/x && chmod +x /usr/local/bin/x

# Or build from source
cd /Users/white/PycharmProjects/twitter-cli && cargo build --release
# Binary: target/release/x
```

### Authentication

```bash
# From browser cookies (recommended)
x auth --browser chrome    # also: firefox, edge, safari

# Or provide auth_token directly
x auth --token "your_auth_token"
```

Credentials stored at: `~/.x-cli/credentials.json`

For advanced use, edit credentials.json to add extra cookies:
```json
{
  "auth_token": "xxx",
  "ct0": "xxx",
  "extra_cookies": "guest_id=xxx; kdt=xxx; twid=xxx"
}
```

## Commands

All commands output JSON to stdout. Status messages go to stderr.
User can be specified by screen name (e.g. `elonmusk`) or numeric user ID.
Default `--limit` is 20 for all list commands.

### Read Operations

```bash
# Current authenticated user info
x me

# Look up user by screen name → returns user object with id_str, name, etc.
x user <screen_name>

# User tweets
x tweets <user> [--limit N]

# User replies
x replies <user> [--limit N]

# Following list
x following <user> [--limit N]

# Followers list
x followers <user> [--limit N]

# Search tweets
x search "<query>" [--limit N]

# Tweet detail (single tweet or with conversation context)
x detail <tweet_id> [--context]
```

### Write Operations

```bash
# Post a new tweet
x post "<text>"

# Reply to a tweet
x reply <tweet_id> "<text>"

# Quote tweet
x quote <tweet_id> "<text>"

# Like / unlike
x like <tweet_id>
x unlike <tweet_id>

# Retweet / undo retweet
x retweet <tweet_id>
x unretweet <tweet_id>

# Follow / unfollow (accepts screen name or user ID)
x follow <user>
x unfollow <user>
```

## Common Patterns

```bash
# Get a user's numeric ID from screen name
x user elonmusk | jq -r '.id_str'

# Fetch latest 10 tweets and extract text
x tweets elonmusk --limit 10 | jq '.[].text'

# Post a tweet and get the tweet ID back
x post "Hello world" | jq -r '.rest_id'

# Reply to someone's latest tweet
TWEET_ID=$(x tweets someuser --limit 1 | jq -r '.[0].rest_id')
x reply "$TWEET_ID" "Nice post!"

# Like all tweets from a search
x search "interesting topic" --limit 5 | jq -r '.[].rest_id' | while read id; do x like "$id"; done

# Check who a user follows
x following elonmusk --limit 100 | jq '.[].screen_name'
```

## Output Format

All JSON output follows Twitter's GraphQL response structure. Key fields in tweet objects:

- `rest_id` — tweet ID
- `core.user_results.result.legacy.screen_name` — author handle
- `legacy.full_text` — tweet text
- `legacy.favorite_count` — like count
- `legacy.retweet_count` — retweet count
- `legacy.reply_count` — reply count

Key fields in user objects:

- `id_str` / `rest_id` — user ID
- `screen_name` — handle
- `name` — display name
- `followers_count` — follower count
- `friends_count` — following count

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `No credentials found` | Run `x auth --browser chrome` |
| `auth_token cookie not found` | Log in to X in your browser first |
| 404 on followers/following/search | Transaction ID issue — tool auto-retries |
| Error 226 on write ops | Add full cookies via `extra_cookies` in credentials.json |
| Empty write results | Cookie expired — re-run `x auth` |

## Files

- `~/.x-cli/credentials.json` — auth credentials
- `~/.x-cli/transaction_cache.json` — cached transaction ID (auto-managed)
