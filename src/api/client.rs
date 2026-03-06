use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};

use super::endpoints;
use super::features;
use super::headers::{build_cookie_header, build_headers};
use super::types::Credentials;

pub struct TwitterClient {
    http: Client,
    creds: Credentials,
}

impl TwitterClient {
    pub fn new(creds: Credentials) -> Result<Self> {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { http, creds })
    }

    // ---- GraphQL GET helper ----

    async fn graphql_get(
        &self,
        url: &str,
        variables: Value,
        features: Value,
        field_toggles: Option<Value>,
    ) -> Result<Value> {
        let headers = build_headers(&self.creds.ct0);
        let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

        let mut params = vec![
            ("variables", serde_json::to_string(&variables)?),
            ("features", serde_json::to_string(&features)?),
        ];
        if let Some(ft) = field_toggles {
            params.push(("fieldToggles", serde_json::to_string(&ft)?));
        }

        let resp = self
            .http
            .get(url)
            .headers(headers)
            .header("cookie", &cookie)
            .query(&params)
            .send()
            .await
            .context("HTTP GET failed")?;

        let status = resp.status();
        let text = resp.text().await.context("Failed to read response body")?;

        if status == 429 {
            anyhow::bail!("Rate limited (429)");
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse JSON (status {status}): {}", &text[..text.len().min(200)]))?;

        if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                let code = errors[0].get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
                let msg = errors[0]
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown");
                if code == 326 {
                    anyhow::bail!("Account banned (error 326)");
                }
                if code == 64 {
                    anyhow::bail!("Account suspended (error 64)");
                }
                // If we also have data, treat as soft error
                if json.get("data").is_none() {
                    anyhow::bail!("API error {code}: {msg}");
                }
            }
        }

        Ok(json)
    }

    // ---- GraphQL POST helper ----

    async fn graphql_post(&self, url: &str, body: Value) -> Result<Value> {
        let headers = build_headers(&self.creds.ct0);
        let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

        let resp = self
            .http
            .post(url)
            .headers(headers)
            .header("cookie", &cookie)
            .json(&body)
            .send()
            .await
            .context("HTTP POST failed")?;

        let status = resp.status();
        let text = resp.text().await?;

        if status == 429 {
            anyhow::bail!("Rate limited (429)");
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse JSON (status {status})"))?;

        Ok(json)
    }

    // ---- REST POST helper (for v1.1 endpoints) ----

    async fn rest_post(&self, url: &str, form: &[(&str, &str)]) -> Result<Value> {
        let headers = build_headers(&self.creds.ct0);
        let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

        let resp = self
            .http
            .post(url)
            .headers(headers)
            .header("cookie", &cookie)
            .header("content-type", "application/x-www-form-urlencoded")
            .form(form)
            .send()
            .await
            .context("HTTP POST failed")?;

        let text = resp.text().await?;
        let json: Value = serde_json::from_str(&text).unwrap_or(json!({"raw": text}));
        Ok(json)
    }

    fn default_field_toggles() -> Value {
        json!({
            "withArticleRichContentState": true,
            "withArticlePlainText": false,
            "withGrokAnalyze": false,
            "withDisallowedReplyControls": false,
        })
    }

    // ===== Read APIs =====

    pub async fn get_user_tweets(
        &self,
        user_id: &str,
        count: u32,
        cursor: Option<&str>,
    ) -> Result<Value> {
        let mut variables = json!({
            "userId": user_id,
            "count": count,
            "includePromotedContent": true,
            "withQuickPromoteEligibilityTweetFields": true,
            "withVoice": true,
            "withV2Timeline": true,
        });
        if let Some(c) = cursor {
            variables["cursor"] = json!(c);
        }
        self.graphql_get(
            endpoints::USER_TWEETS,
            variables,
            features::features_tweets(),
            Some(Self::default_field_toggles()),
        )
        .await
    }

    pub async fn get_user_replies(
        &self,
        user_id: &str,
        count: u32,
        cursor: Option<&str>,
    ) -> Result<Value> {
        let mut variables = json!({
            "userId": user_id,
            "count": count,
            "includePromotedContent": true,
            "withCommunity": true,
            "withVoice": true,
        });
        if let Some(c) = cursor {
            variables["cursor"] = json!(c);
        }
        self.graphql_get(
            endpoints::USER_TWEETS_AND_REPLIES,
            variables,
            features::features_replies(),
            Some(Self::default_field_toggles()),
        )
        .await
    }

    pub async fn get_followers(
        &self,
        user_id: &str,
        count: u32,
        cursor: Option<&str>,
    ) -> Result<Value> {
        let mut variables = json!({
            "userId": user_id,
            "count": count,
            "includePromotedContent": false,
        });
        if let Some(c) = cursor {
            variables["cursor"] = json!(c);
        }
        self.graphql_get(
            endpoints::FOLLOWERS,
            variables,
            features::features_followers(),
            None,
        )
        .await
    }

    pub async fn get_following(
        &self,
        user_id: &str,
        count: u32,
        cursor: Option<&str>,
    ) -> Result<Value> {
        let mut variables = json!({
            "userId": user_id,
            "count": count,
            "includePromotedContent": false,
        });
        if let Some(c) = cursor {
            variables["cursor"] = json!(c);
        }
        self.graphql_get(
            endpoints::FOLLOWING,
            variables,
            features::features_followings(),
            None,
        )
        .await
    }

    pub async fn search_tweets(
        &self,
        query: &str,
        count: u32,
        cursor: Option<&str>,
    ) -> Result<Value> {
        let mut variables = json!({
            "rawQuery": query,
            "count": count,
            "querySource": "typed_query",
            "product": "Latest",
        });
        if let Some(c) = cursor {
            variables["cursor"] = json!(c);
        }
        self.graphql_get(
            endpoints::SEARCH_TIMELINE,
            variables,
            features::features_search(),
            Some(Self::default_field_toggles()),
        )
        .await
    }

    pub async fn get_tweet_detail(&self, tweet_id: &str) -> Result<Value> {
        let variables = json!({
            "tweetId": tweet_id,
            "withCommunity": false,
            "includePromotedContent": false,
            "withVoice": false,
        });
        self.graphql_get(
            endpoints::TWEET_RESULT_BY_REST_ID,
            variables,
            features::features_tweet_detail(),
            Some(Self::default_field_toggles()),
        )
        .await
    }

    pub async fn get_tweet_detail_with_context(&self, tweet_id: &str) -> Result<Value> {
        let variables = json!({
            "focalTweetId": tweet_id,
            "with_rux_injections": false,
            "rankingMode": "Relevance",
            "includePromotedContent": true,
            "withCommunity": true,
            "withQuickPromoteEligibilityTweetFields": true,
            "withBirdwatchNotes": true,
            "withVoice": true,
        });
        self.graphql_get(
            endpoints::TWEET_DETAIL,
            variables,
            features::features_tweet_detail_context(),
            Some(Self::default_field_toggles()),
        )
        .await
    }

    pub async fn get_user_by_screen_name(&self, screen_name: &str) -> Result<Value> {
        let variables = json!({
            "screen_name": screen_name,
            "withSafetyModeUserFields": true,
            "withSuperFollowsUserFields": true,
        });
        self.graphql_get(
            endpoints::USER_BY_SCREEN_NAME,
            variables,
            features::features_tweets(), // reuse basic features
            None,
        )
        .await
    }

    // ===== Write APIs =====

    pub async fn create_tweet(
        &self,
        text: &str,
        reply_to: Option<&str>,
        quote_tweet_id: Option<&str>,
    ) -> Result<Value> {
        let mut variables = json!({
            "tweet_text": text,
            "dark_request": false,
            "media": {
                "media_entities": [],
                "possibly_sensitive": false,
            },
            "semantic_annotation_ids": [],
        });

        if let Some(reply_id) = reply_to {
            variables["reply"] = json!({
                "in_reply_to_tweet_id": reply_id,
                "exclude_reply_user_ids": [],
            });
        }

        if let Some(quote_id) = quote_tweet_id {
            variables["attachment_url"] =
                json!(format!("https://x.com/i/web/status/{quote_id}"));
        }

        let body = json!({
            "variables": variables,
            "features": features::features_create_tweet(),
            "queryId": "oB-5XsHNAbjvARJEc8CZFw",
        });

        self.graphql_post(endpoints::CREATE_TWEET, body).await
    }

    pub async fn like_tweet(&self, tweet_id: &str) -> Result<Value> {
        let body = json!({
            "variables": {"tweet_id": tweet_id},
            "queryId": "lI07N6OdyUlbRl84p-7-nQ",
        });
        self.graphql_post(endpoints::FAVORITE_TWEET, body).await
    }

    pub async fn unlike_tweet(&self, tweet_id: &str) -> Result<Value> {
        let body = json!({
            "variables": {"tweet_id": tweet_id},
            "queryId": "ZYKSe-w7KEslx3JhSIk5LA",
        });
        self.graphql_post(endpoints::UNFAVORITE_TWEET, body).await
    }

    pub async fn retweet(&self, tweet_id: &str) -> Result<Value> {
        let body = json!({
            "variables": {"tweet_id": tweet_id, "dark_request": false},
            "queryId": "ojPdsZsimiJrUGLR1sjUtA",
        });
        self.graphql_post(endpoints::CREATE_RETWEET, body).await
    }

    pub async fn unretweet(&self, tweet_id: &str) -> Result<Value> {
        let body = json!({
            "variables": {"source_tweet_id": tweet_id, "dark_request": false},
            "queryId": "iQtK4dl5hBmXewYZuEOKVw",
        });
        self.graphql_post(endpoints::DELETE_RETWEET, body).await
    }

    pub async fn follow_user(&self, user_id: &str) -> Result<Value> {
        self.rest_post(
            endpoints::CREATE_FRIENDSHIP,
            &[("user_id", user_id), ("include_profile_interstitial_type", "1")],
        )
        .await
    }

    pub async fn unfollow_user(&self, user_id: &str) -> Result<Value> {
        self.rest_post(
            endpoints::DESTROY_FRIENDSHIP,
            &[("user_id", user_id), ("include_profile_interstitial_type", "1")],
        )
        .await
    }
}
