---
name: twitter
description: Use Twitter/X CLI tool for reading and writing tweets, managing follows, and interacting with content. Use when user asks about Twitter operations like fetching tweets, posting, liking, retweeting, following, or searching.
---

# Twitter CLI Tool

## Setup
The binary is located at: /Users/white/PycharmProjects/twitter-cli/target/release/twitter-cli

If not built yet, build with:
```bash
cd /Users/white/PycharmProjects/twitter-cli && source ~/.cargo/env && cargo build --release
```

## Authentication

### Extract cookies from browser (recommended)
```bash
twitter-cli auth --browser chrome
# Also supports: firefox, edge, safari
```

### Provide auth_token directly
```bash
twitter-cli auth --token "your_auth_token_here"
```

Credentials are stored at `~/.twitter-cli/credentials.json`.

## Reading Data

### Get user tweets
```bash
twitter-cli tweets <user_id> [--limit N]
```

### Get user replies
```bash
twitter-cli replies <user_id> [--limit N]
```

### Get user's following list
```bash
twitter-cli following <user_id> [--limit N]
```

### Get user's followers
```bash
twitter-cli followers <user_id> [--limit N]
```

### Search tweets
```bash
twitter-cli search "query string" [--limit N]
```

### Get tweet detail
```bash
twitter-cli detail <tweet_id>
twitter-cli detail <tweet_id> --context  # includes conversation thread
```

### Look up user by handle
```bash
twitter-cli user <screen_name>
```

## Writing / Interactions

### Post a tweet
```bash
twitter-cli post "Hello world"
```

### Reply to a tweet
```bash
twitter-cli reply <tweet_id> "reply text"
```

### Quote tweet
```bash
twitter-cli quote <tweet_id> "quote text"
```

### Like / Unlike
```bash
twitter-cli like <tweet_id>
twitter-cli unlike <tweet_id>
```

### Retweet / Undo retweet
```bash
twitter-cli retweet <tweet_id>
twitter-cli unretweet <tweet_id>
```

### Follow / Unfollow
```bash
twitter-cli follow <user_id>
twitter-cli unfollow <user_id>
```

## Notes
- All commands output JSON to stdout
- Status messages go to stderr
- user_id is the numeric Twitter user ID (not the handle)
- Use `twitter-cli user <handle>` to look up a user's ID from their screen name
- Default limit is 20 for all list commands
