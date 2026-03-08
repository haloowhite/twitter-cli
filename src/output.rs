use serde::Serialize;
use serde_json::Value;

// ── Output structs ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TweetOutput {
    pub id: String,
    pub url: String,
    pub text: String,
    pub created_at: String,
    pub lang: String,
    pub author: TweetAuthor,
    pub stats: TweetStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_tweet: Option<ReferencedTweet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TweetAuthor {
    pub id: String,
    pub handle: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TweetStats {
    pub views: u64,
    pub likes: u64,
    pub retweets: u64,
    pub replies: u64,
    pub quotes: u64,
    pub bookmarks: u64,
}

#[derive(Debug, Serialize)]
pub struct ReferencedTweet {
    pub id: String,
    #[serde(rename = "type")]
    pub ref_type: String, // "retweet" or "quote"
}

#[derive(Debug, Serialize)]
pub struct UserOutput {
    pub id: String,
    pub screen_name: String,
    pub name: String,
    pub description: String,
    pub followers_count: u64,
    pub following_count: u64,
    pub tweet_count: u64,
    pub is_verified: bool,
    pub created_at: String,
    pub profile_image_url: String,
}

#[derive(Debug, Serialize)]
pub struct ActionResult {
    pub success: bool,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Compact tweet: one-liner per tweet, minimal tokens for LLM consumption.
#[derive(Debug, Serialize)]
pub struct CompactTweet {
    pub id: String,
    pub author: String,   // @handle
    pub text: String,     // truncated, single line
    pub likes: u64,
    pub rts: u64,
    pub time: String,     // short time
}

impl CompactTweet {
    pub fn from_tweet(t: &TweetOutput) -> Self {
        // Collapse newlines, truncate to 140 chars
        let text = t.text.replace('\n', " ");
        let text = if text.len() > 140 { format!("{}...", &text[..text.char_indices().take_while(|(i, _)| *i < 140).last().map(|(i, c)| i + c.len_utf8()).unwrap_or(140)]) } else { text };
        // Short time: "Mar 07 05:51"
        let time = parse_short_time(&t.created_at);
        CompactTweet {
            id: t.id.clone(),
            author: format!("@{}", t.author.handle),
            text,
            likes: t.stats.likes,
            rts: t.stats.retweets,
            time,
        }
    }
}

fn parse_short_time(created_at: &str) -> String {
    // Input: "Sat Mar 07 05:51:02 +0000 2026"
    // Output: "Mar 07 05:51"
    let parts: Vec<&str> = created_at.split_whitespace().collect();
    if parts.len() >= 4 {
        format!("{} {} {}", parts[1], parts[2], &parts[3][..5])
    } else {
        created_at.to_string()
    }
}

// ── Recursive JSON search (ported from heimdall json_praser.py) ─────

fn find_all_by_field(data: &Value, key: &str, target: &str) -> Vec<Value> {
    let mut results = Vec::new();
    match data {
        Value::Object(map) => {
            if map.get(key).and_then(|v| v.as_str()) == Some(target) {
                results.push(data.clone());
            }
            for v in map.values() {
                results.extend(find_all_by_field(v, key, target));
            }
        }
        Value::Array(arr) => {
            for item in arr {
                results.extend(find_all_by_field(item, key, target));
            }
        }
        _ => {}
    }
    results
}

fn find_first_by_field(data: &Value, key: &str, target: &str) -> Option<Value> {
    match data {
        Value::Object(map) => {
            if map.get(key).and_then(|v| v.as_str()) == Some(target) {
                return Some(data.clone());
            }
            for v in map.values() {
                if let Some(r) = find_first_by_field(v, key, target) {
                    return Some(r);
                }
            }
            None
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(r) = find_first_by_field(item, key, target) {
                    return Some(r);
                }
            }
            None
        }
        _ => None,
    }
}

// ── Field helpers ───────────────────────────────────────────────────

fn str_field(v: &Value, path: &[&str]) -> String {
    let mut cur = v;
    for &p in path {
        cur = &cur[p];
    }
    cur.as_str().unwrap_or("").to_string()
}

fn u64_field(v: &Value, path: &[&str]) -> u64 {
    let mut cur = v;
    for &p in path {
        cur = &cur[p];
    }
    cur.as_u64().unwrap_or(0)
}

fn opt_str(v: &Value, path: &[&str]) -> Option<String> {
    let mut cur = v;
    for &p in path {
        cur = &cur[p];
    }
    cur.as_str().map(|s| s.to_string())
}

// ── Single tweet extraction (from raw __typename:"Tweet" object) ────

fn extract_tweet_from_raw(raw: &Value) -> Option<TweetOutput> {
    let tweet_id = str_field(raw, &["rest_id"]);
    if tweet_id.is_empty() {
        return None;
    }

    let legacy = &raw["legacy"];
    let user_result = &raw["core"]["user_results"]["result"];

    // User fields: try core.screen_name first, fallback to legacy.screen_name
    let user_core = &user_result["core"];
    let user_legacy = &user_result["legacy"];

    let handle = opt_str(user_core, &["screen_name"])
        .or_else(|| opt_str(user_legacy, &["screen_name"]))
        .unwrap_or_default();
    let name = opt_str(user_core, &["name"])
        .or_else(|| opt_str(user_legacy, &["name"]))
        .unwrap_or_default();
    let author_id = str_field(user_result, &["rest_id"]);

    // Text: prefer note_tweet (long tweets), fallback to legacy.full_text
    let note_text = opt_str(raw, &["note_tweet", "note_tweet_results", "result", "text"]);
    let text = note_text
        .unwrap_or_else(|| str_field(legacy, &["full_text"]));

    // Views
    let views_str = str_field(raw, &["views", "count"]);
    let views = views_str.parse::<u64>().unwrap_or(0);

    // Referenced tweet
    let referenced_tweet = if legacy.get("retweeted_status_result").is_some() {
        let rt_id = str_field(legacy, &["retweeted_status_result", "result", "rest_id"]);
        if !rt_id.is_empty() {
            Some(ReferencedTweet { id: rt_id, ref_type: "retweet".into() })
        } else {
            None
        }
    } else if raw.get("quoted_status_result").is_some() {
        let qt_id = str_field(raw, &["quoted_status_result", "result", "rest_id"]);
        if !qt_id.is_empty() {
            Some(ReferencedTweet { id: qt_id, ref_type: "quote".into() })
        } else {
            None
        }
    } else {
        None
    };

    let in_reply_to_id = opt_str(legacy, &["in_reply_to_status_id_str"]);

    Some(TweetOutput {
        url: format!("https://x.com/{handle}/status/{tweet_id}"),
        id: tweet_id,
        text,
        created_at: str_field(legacy, &["created_at"]),
        lang: str_field(legacy, &["lang"]),
        author: TweetAuthor { id: author_id, handle, name },
        stats: TweetStats {
            views,
            likes: u64_field(legacy, &["favorite_count"]),
            retweets: u64_field(legacy, &["retweet_count"]),
            replies: u64_field(legacy, &["reply_count"]),
            quotes: u64_field(legacy, &["quote_count"]),
            bookmarks: u64_field(legacy, &["bookmark_count"]),
        },
        referenced_tweet,
        in_reply_to_id,
    })
}

// ── Single user extraction ──────────────────────────────────────────

fn extract_user_from_raw(raw: &Value) -> Option<UserOutput> {
    let id = str_field(raw, &["rest_id"]);
    if id.is_empty() {
        return None;
    }

    let core = &raw["core"];
    let legacy = &raw["legacy"];
    let avatar = &raw["avatar"];

    let screen_name = opt_str(core, &["screen_name"])
        .or_else(|| opt_str(legacy, &["screen_name"]))
        .unwrap_or_default();
    let name = opt_str(core, &["name"])
        .or_else(|| opt_str(legacy, &["name"]))
        .unwrap_or_default();
    let profile_image_url = opt_str(avatar, &["image_url"])
        .or_else(|| opt_str(legacy, &["profile_image_url_https"]))
        .unwrap_or_default();
    let created_at = opt_str(core, &["created_at"])
        .or_else(|| opt_str(legacy, &["created_at"]))
        .unwrap_or_default();

    Some(UserOutput {
        id,
        screen_name,
        name,
        description: str_field(legacy, &["description"]),
        followers_count: u64_field(legacy, &["followers_count"]),
        following_count: u64_field(legacy, &["friends_count"]),
        tweet_count: u64_field(legacy, &["statuses_count"]),
        is_verified: raw.get("is_blue_verified").and_then(|v| v.as_bool()).unwrap_or(false),
        created_at,
        profile_image_url,
    })
}

// ── Public extraction functions ─────────────────────────────────────

/// Extract tweets from any timeline-style response (tweets, replies, search).
/// Only extracts top-level tweet_results entries — does NOT recurse into
/// quoted_status_result or retweeted_status_result to avoid duplicates.
pub fn extract_tweets(resp: &Value) -> Vec<TweetOutput> {
    let tweet_results = find_all_tweet_results(resp);
    let mut seen = std::collections::HashSet::new();
    let mut tweets = Vec::new();
    for raw in &tweet_results {
        if let Some(t) = extract_tweet_from_raw(raw) {
            if seen.insert(t.id.clone()) {
                tweets.push(t);
            }
        }
    }
    tweets
}

/// Find all top-level tweet objects from timeline entries.
/// Looks for "tweet_results" -> "result" objects that have itemType "TimelineTweet",
/// without recursing into quoted/retweeted sub-tweets.
fn find_all_tweet_results(data: &Value) -> Vec<Value> {
    let mut results = Vec::new();
    match data {
        Value::Object(map) => {
            // If this object has itemType == "TimelineTweet" and tweet_results.result,
            // extract the tweet and DON'T recurse deeper (avoids quoted/retweeted dupes)
            if map.get("itemType").and_then(|v| v.as_str()) == Some("TimelineTweet") {
                if let Some(result) = map.get("tweet_results")
                    .and_then(|tr| tr.get("result"))
                {
                    // Handle TweetWithVisibilityResults wrapper
                    let tweet = if result.get("__typename").and_then(|v| v.as_str())
                        == Some("TweetWithVisibilityResults")
                    {
                        result.get("tweet").unwrap_or(result)
                    } else {
                        result
                    };
                    results.push(tweet.clone());
                }
                return results; // Don't recurse into this entry's children
            }
            // Otherwise recurse into children
            for v in map.values() {
                results.extend(find_all_tweet_results(v));
            }
        }
        Value::Array(arr) => {
            for item in arr {
                results.extend(find_all_tweet_results(item));
            }
        }
        _ => {}
    }
    results
}

/// Extract users from followers/following responses.
pub fn extract_users(resp: &Value) -> Vec<UserOutput> {
    // Followers/following entries have userDisplayType: "User"
    let items = find_all_by_field(resp, "userDisplayType", "User");
    let mut users = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in &items {
        // Navigate: item["user_results"]["result"]
        let user_raw = &item["user_results"]["result"];
        if let Some(u) = extract_user_from_raw(user_raw) {
            if seen.insert(u.id.clone()) {
                users.push(u);
            }
        }
    }
    users
}

/// Extract single user from user lookup / me response.
pub fn extract_single_user(resp: &Value) -> Option<UserOutput> {
    // Try data.user.result first
    let user_raw = &resp["data"]["user"]["result"];
    if user_raw.is_object() {
        return extract_user_from_raw(user_raw);
    }
    // Fallback: find first User __typename
    let raw = find_first_by_field(resp, "__typename", "User")?;
    extract_user_from_raw(&raw)
}

/// Extract single tweet from detail or create_tweet response.
pub fn extract_single_tweet(resp: &Value) -> Option<TweetOutput> {
    // TweetResultByRestId: data.tweetResult.result
    let detail = &resp["data"]["tweetResult"]["result"];
    if detail.is_object() && detail.get("rest_id").is_some() {
        return extract_tweet_from_raw(detail);
    }
    // CreateTweet: data.create_tweet.tweet_results.result
    let created = &resp["data"]["create_tweet"]["tweet_results"]["result"];
    if created.is_object() && created.get("rest_id").is_some() {
        return extract_tweet_from_raw(created);
    }
    // Fallback: first __typename == "Tweet"
    let raw = find_first_by_field(resp, "__typename", "Tweet")?;
    extract_tweet_from_raw(&raw)
}

/// Build ActionResult from write operation responses.
pub fn extract_action_result(resp: &Value, action: &str) -> ActionResult {
    match action {
        "like" => {
            let done = resp["data"]["favorite_tweet"].as_str() == Some("Done");
            ActionResult { success: done, action: "like".into(), id: None }
        }
        "unlike" => {
            let done = resp["data"]["unfavorite_tweet"].as_str() == Some("Done");
            ActionResult { success: done, action: "unlike".into(), id: None }
        }
        "retweet" => {
            let id = opt_str(resp, &["data", "create_retweet", "retweet_results", "result", "rest_id"]);
            ActionResult { success: id.is_some(), action: "retweet".into(), id }
        }
        "unretweet" => {
            // DeleteRetweet returns data.unretweet
            let done = resp["data"]["unretweet"].get("source_tweet_results").is_some();
            ActionResult { success: done, action: "unretweet".into(), id: None }
        }
        "follow" | "unfollow" => {
            // REST v1.1 returns user object directly with "id" or "id_str"
            let id = opt_str(resp, &["id_str"]);
            ActionResult { success: id.is_some(), action: action.into(), id }
        }
        _ => ActionResult { success: false, action: action.into(), id: None },
    }
}

/// Extract user from REST v1.1 follow/unfollow response (returns user object directly).
pub fn extract_user_from_rest(resp: &Value) -> Option<UserOutput> {
    let id = opt_str(resp, &["id_str"])?;
    Some(UserOutput {
        id,
        screen_name: str_field(resp, &["screen_name"]),
        name: str_field(resp, &["name"]),
        description: str_field(resp, &["description"]),
        followers_count: u64_field(resp, &["followers_count"]),
        following_count: u64_field(resp, &["friends_count"]),
        tweet_count: u64_field(resp, &["statuses_count"]),
        is_verified: resp.get("verified").and_then(|v| v.as_bool()).unwrap_or(false),
        created_at: str_field(resp, &["created_at"]),
        profile_image_url: str_field(resp, &["profile_image_url_https"]),
    })
}

/// Extract user from /account/multi/list.json (me command).
/// Response is an array of account objects: [{"user": {...}}, ...]
pub fn extract_me_user(resp: &Value) -> Option<UserOutput> {
    if let Some(arr) = resp.as_array() {
        if let Some(first) = arr.first() {
            let user = &first["user"];
            return extract_user_from_rest(user);
        }
    }
    // Fallback: try GraphQL user format
    extract_single_user(resp)
}
