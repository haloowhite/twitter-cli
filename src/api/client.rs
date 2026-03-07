use anyhow::{Context, Result};
use rquest::Client;
use rquest_util::Emulation;
use serde_json::{json, Value};

use super::endpoints;
use super::features;
use super::headers::{build_cookie_header, build_headers};
use super::transaction::ClientTransaction;
use super::types::Credentials;

pub struct TwitterClient {
    http: Client,
    creds: Credentials,
    transaction: Option<ClientTransaction>,
}

impl TwitterClient {
    pub async fn new(creds: Credentials) -> Result<Self> {
        let http = Client::builder()
            .emulation(Emulation::Chrome133)
            .cookie_store(false)
            .build()
            .context("Failed to build HTTP client")?;

        let transaction = match ClientTransaction::new().await {
            Ok(ct) => Some(ct),
            Err(e) => {
                eprintln!("Warning: Failed to init transaction ID: {e}");
                None
            }
        };

        Ok(Self {
            http,
            creds,
            transaction,
        })
    }

    fn get_transaction_id(&self, method: &str, url: &str) -> Option<String> {
        let path = url.strip_prefix("https://x.com").unwrap_or(url);
        self.transaction
            .as_ref()
            .map(|ct| ct.generate(method, path))
    }

    // ---- GraphQL GET helper ----

    async fn graphql_get(
        &self,
        url: &str,
        variables: Value,
        features: Value,
        field_toggles: Option<Value>,
    ) -> Result<Value> {
        let params_variables = serde_json::to_string(&variables)?;
        let params_features = serde_json::to_string(&features)?;
        let params_field_toggles = field_toggles
            .as_ref()
            .map(|ft| serde_json::to_string(ft))
            .transpose()?;

        let max_retries = 5;
        let mut last_status = rquest::StatusCode::default();

        for attempt in 0..max_retries {
            let mut headers = build_headers(&self.creds.ct0);
            let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

            if let Some(tid) = self.get_transaction_id("GET", url) {
                headers.insert(
                    "x-client-transaction-id",
                    rquest::header::HeaderValue::from_str(&tid).unwrap(),
                );
            }

            let mut params = vec![
                ("variables", params_variables.clone()),
                ("features", params_features.clone()),
            ];
            if let Some(ref ft) = params_field_toggles {
                params.push(("fieldToggles", ft.clone()));
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

            last_status = resp.status();
            let text = resp.text().await.context("Failed to read response body")?;

            if last_status == 404 && attempt < max_retries - 1 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            if !last_status.is_success() {
                let preview = if text.is_empty() {
                    "(empty body)"
                } else {
                    &text[..text.len().min(500)]
                };
                anyhow::bail!("HTTP {last_status} from {url}\nBody: {preview}");
            }

            return self.parse_graphql_response(url, last_status, &text);
        }

        anyhow::bail!("HTTP {last_status} from {url} after {max_retries} retries");

        unreachable!()
    }

    fn parse_graphql_response(
        &self,
        url: &str,
        status: rquest::StatusCode,
        text: &str,
    ) -> Result<Value> {
        let json: Value = serde_json::from_str(text).with_context(|| {
            format!(
                "Failed to parse JSON (status {status}): {}",
                &text[..text.len().min(200)]
            )
        })?;

        if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
            if !errors.is_empty() {
                let code = errors[0]
                    .get("code")
                    .and_then(|c| c.as_i64())
                    .unwrap_or(-1);
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
                if json.get("data").is_none() {
                    anyhow::bail!("API error {code}: {msg} (from {url})");
                }
            }
        }

        Ok(json)
    }

    // ---- GraphQL POST helper ----

    async fn graphql_post(&self, url: &str, body: Value) -> Result<Value> {
        let max_retries = 5;
        let mut last_status = rquest::StatusCode::default();

        for attempt in 0..max_retries {
            let mut headers = build_headers(&self.creds.ct0);
            let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

            if let Some(tid) = self.get_transaction_id("POST", url) {
                headers.insert(
                    "x-client-transaction-id",
                    rquest::header::HeaderValue::from_str(&tid).unwrap(),
                );
            }

            let resp = self
                .http
                .post(url)
                .headers(headers)
                .header("cookie", &cookie)
                .json(&body)
                .send()
                .await
                .context("HTTP POST failed")?;

            last_status = resp.status();
            let text = resp.text().await?;

            if last_status == 404 && attempt < max_retries - 1 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            if last_status == 429 {
                anyhow::bail!("Rate limited (429)");
            }

            let json: Value = serde_json::from_str(&text)
                .with_context(|| format!("Failed to parse JSON (status {last_status})"))?;

            return Ok(json);
        }

        anyhow::bail!("HTTP {last_status} from {url} after {max_retries} retries");
    }

    // ---- REST POST helper (for v1.1 endpoints) ----

    async fn rest_post(&self, url: &str, form: &[(&str, &str)]) -> Result<Value> {
        let mut headers = build_headers(&self.creds.ct0);
        let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

        if let Some(tid) = self.get_transaction_id("POST", url) {
            headers.insert(
                "x-client-transaction-id",
                rquest::header::HeaderValue::from_str(&tid).unwrap(),
            );
        }

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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
            features::features(),
            Some(features::field_toggles()),
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
                json!(format!("https://x.com/x/status/{quote_id}"));
        }

        let body = json!({
            "variables": variables,
            "features": features::features(),
            "queryId": "uY34Pldm6W89yqswRmPMSQ",
        });

        self.graphql_post(endpoints::CREATE_TWEET, body).await
    }

    pub async fn like_tweet(&self, tweet_id: &str) -> Result<Value> {
        let body = json!({
            "variables": {"tweet_id": tweet_id},
            "queryId": "lI07N6Otwv1PhnEgXILM7A",
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
            "queryId": "mbRO74GrOvSfRcJnlMapnQ",
        });
        self.graphql_post(endpoints::CREATE_RETWEET, body).await
    }

    pub async fn unretweet(&self, tweet_id: &str) -> Result<Value> {
        let body = json!({
            "variables": {"source_tweet_id": tweet_id, "dark_request": false},
            "queryId": "ZyZigVsNiFO6v1dEks1eWg",
        });
        self.graphql_post(endpoints::DELETE_RETWEET, body).await
    }

    pub async fn follow_user(&self, user_id: &str) -> Result<Value> {
        self.rest_post(
            endpoints::CREATE_FRIENDSHIP,
            &[
                ("user_id", user_id),
                ("include_profile_interstitial_type", "1"),
            ],
        )
        .await
    }

    pub async fn unfollow_user(&self, user_id: &str) -> Result<Value> {
        self.rest_post(
            endpoints::DESTROY_FRIENDSHIP,
            &[
                ("user_id", user_id),
                ("include_profile_interstitial_type", "1"),
            ],
        )
        .await
    }

    pub async fn get_me(&self) -> Result<Value> {
        let headers = build_headers(&self.creds.ct0);
        let cookie = build_cookie_header(&self.creds.auth_token, &self.creds.ct0);

        let resp = self
            .http
            .get(endpoints::ACCOUNT_MULTI_LIST)
            .headers(headers)
            .header("cookie", &cookie)
            .send()
            .await
            .context("HTTP GET failed")?;

        let text = resp.text().await.context("Failed to read response body")?;
        let accounts: Value =
            serde_json::from_str(&text).context("Failed to parse account list")?;

        let screen_name = accounts
            .get("users")
            .and_then(|u| u.as_array())
            .and_then(|arr| arr.first())
            .and_then(|u| u.get("screen_name"))
            .and_then(|v| v.as_str())
            .context("screen_name not found in account list")?
            .to_string();

        self.get_user_by_screen_name(&screen_name).await
    }
}
