use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::browser::generate_csrf_token;
use crate::api::headers::BEARER_TOKEN;
use crate::api::types::Credentials;

const LOGIN_URL: &str = "https://api.twitter.com/1.1/onboarding/task.json";
const GUEST_ACTIVATE_URL: &str = "https://api.twitter.com/1.1/guest/activate.json";

struct LoginFlow {
    client: reqwest::Client,
    guest_token: String,
    flow_token: String,
}

impl LoginFlow {
    async fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        // Step 1: Get guest token
        let resp = client
            .post(GUEST_ACTIVATE_URL)
            .header("authorization", BEARER_TOKEN)
            .send()
            .await
            .context("Failed to activate guest token")?;

        let body: Value = resp.json().await?;
        let guest_token = body["guest_token"]
            .as_str()
            .context("Failed to get guest_token")?
            .to_string();

        // Step 2: Start login flow
        let resp = client
            .post(format!("{LOGIN_URL}?flow_name=login"))
            .header("authorization", BEARER_TOKEN)
            .header("x-guest-token", &guest_token)
            .header("content-type", "application/json")
            .json(&json!({
                "input_flow_data": {
                    "flow_context": {
                        "debug_overrides": {},
                        "start_location": {"location": "manual_link"}
                    }
                }
            }))
            .send()
            .await
            .context("Failed to start login flow")?;

        let body: Value = resp.json().await?;
        let flow_token = body["flow_token"]
            .as_str()
            .context("Failed to get flow_token")?
            .to_string();

        Ok(Self {
            client,
            guest_token,
            flow_token,
        })
    }

    async fn task(&mut self, body: Value) -> Result<(Value, Vec<String>)> {
        let resp = self
            .client
            .post(LOGIN_URL)
            .header("authorization", BEARER_TOKEN)
            .header("x-guest-token", &self.guest_token)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Login task request failed")?;

        // Extract Set-Cookie headers
        let cookies: Vec<String> = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        let text = resp.text().await?;
        let body: Value = serde_json::from_str(&text).with_context(|| {
            format!("Failed to parse login response: {}", &text[..text.len().min(300)])
        })?;

        if let Some(ft) = body["flow_token"].as_str() {
            self.flow_token = ft.to_string();
        }

        Ok((body, cookies))
    }

    async fn submit_username(&mut self, username: &str) -> Result<(Value, Vec<String>)> {
        self.task(json!({
            "flow_token": self.flow_token,
            "subtask_inputs": [{
                "subtask_id": "LoginEnterUserIdentifierSSO",
                "settings_list": {
                    "setting_responses": [{
                        "key": "user_identifier",
                        "response_data": {
                            "text_data": {"result": username}
                        }
                    }],
                    "link": "next_link"
                }
            }]
        }))
        .await
    }

    async fn submit_password(&mut self, password: &str) -> Result<(Value, Vec<String>)> {
        self.task(json!({
            "flow_token": self.flow_token,
            "subtask_inputs": [{
                "subtask_id": "LoginEnterPassword",
                "enter_password": {
                    "password": password,
                    "link": "next_link"
                }
            }]
        }))
        .await
    }

    async fn submit_2fa(&mut self, code: &str) -> Result<(Value, Vec<String>)> {
        self.task(json!({
            "flow_token": self.flow_token,
            "subtask_inputs": [{
                "subtask_id": "LoginTwoFactorAuthChallenge",
                "enter_text": {
                    "text": code,
                    "link": "next_link"
                }
            }]
        }))
        .await
    }

    async fn handle_duplication_check(&mut self) -> Result<(Value, Vec<String>)> {
        self.task(json!({
            "flow_token": self.flow_token,
            "subtask_inputs": [{
                "subtask_id": "AccountDuplicationCheck",
                "check_logged_in_account": {
                    "link": "AccountDuplicationCheck_false"
                }
            }]
        }))
        .await
    }

    async fn submit_alternate_id(&mut self, identifier: &str) -> Result<(Value, Vec<String>)> {
        self.task(json!({
            "flow_token": self.flow_token,
            "subtask_inputs": [{
                "subtask_id": "LoginEnterAlternateIdentifierSubtask",
                "enter_text": {
                    "text": identifier,
                    "link": "next_link"
                }
            }]
        }))
        .await
    }
}

fn extract_cookie_value(cookies: &[String], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    for cookie in cookies {
        // Each Set-Cookie header: "name=value; Path=...; ..."
        if let Some(rest) = cookie.strip_prefix(&prefix) {
            if let Some(value) = rest.split(';').next() {
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
        // Also check if cookie starts with whitespace or has other prefix
        for part in cookie.split(';') {
            let trimmed = part.trim();
            if let Some(rest) = trimmed.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.starts_with(';') {
                    let val = rest.split(';').next().unwrap_or(rest);
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_subtask_ids(body: &Value) -> Vec<String> {
    body["subtasks"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s["subtask_id"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn login(
    username: &str,
    password: &str,
    totp_code: Option<&str>,
) -> Result<Credentials> {
    eprintln!("Starting login flow...");
    let mut flow = LoginFlow::new().await?;
    let mut all_cookies = Vec::new();

    // Submit username
    eprintln!("Submitting username...");
    let (body, cookies) = flow.submit_username(username).await?;
    all_cookies.extend(cookies);
    let subtasks = get_subtask_ids(&body);

    // Handle alternate identifier challenge (e.g. email/phone verification)
    if subtasks.contains(&"LoginEnterAlternateIdentifierSubtask".to_string()) {
        let alt_id = match totp_code {
            Some(_) => {
                // If 2FA code provided, ask for alternate ID from stdin
                eprintln!("Twitter requires additional verification.");
                eprint!("Enter your email or phone number: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            }
            None => {
                eprint!("Twitter requires additional verification.\nEnter your email or phone number: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            }
        };
        let (_, cookies) = flow.submit_alternate_id(&alt_id).await?;
        all_cookies.extend(cookies);
    }

    // Submit password
    eprintln!("Submitting password...");
    let (body, cookies) = flow.submit_password(password).await?;
    all_cookies.extend(cookies);
    let subtasks = get_subtask_ids(&body);

    // Handle various post-password subtasks
    if subtasks.contains(&"LoginTwoFactorAuthChallenge".to_string()) {
        let code = match totp_code {
            Some(c) => c.to_string(),
            None => {
                eprint!("Enter 2FA code: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            }
        };
        let (body, cookies) = flow.submit_2fa(&code).await?;
        all_cookies.extend(cookies);
        let subtasks = get_subtask_ids(&body);

        if subtasks.contains(&"AccountDuplicationCheck".to_string()) {
            let (_, cookies) = flow.handle_duplication_check().await?;
            all_cookies.extend(cookies);
        }
    } else if subtasks.contains(&"AccountDuplicationCheck".to_string()) {
        let (_, cookies) = flow.handle_duplication_check().await?;
        all_cookies.extend(cookies);
    } else if subtasks.contains(&"LoginAcid".to_string()) {
        anyhow::bail!("Twitter requires email verification. Please verify your email on twitter.com first.");
    } else if subtasks.contains(&"DenyLoginSubtask".to_string()) {
        anyhow::bail!("Login denied by Twitter. Your account may be locked or suspended.");
    }

    // Check for errors in the response
    if let Some(errors) = body.get("errors").and_then(|e| e.as_array()) {
        if let Some(first) = errors.first() {
            let msg = first["message"].as_str().unwrap_or("Unknown error");
            let code = first["code"].as_i64().unwrap_or(-1);
            anyhow::bail!("Login error {code}: {msg}");
        }
    }

    // Extract auth_token and ct0 from collected cookies
    let auth_token = extract_cookie_value(&all_cookies, "auth_token")
        .context("Login failed: auth_token not found in response. Check your credentials.")?;

    let ct0 = extract_cookie_value(&all_cookies, "ct0")
        .unwrap_or_else(generate_csrf_token);

    // Build extra cookies
    let extra_parts: Vec<String> = ["guest_id", "kdt", "twid"]
        .iter()
        .filter_map(|name| {
            extract_cookie_value(&all_cookies, name).map(|v| format!("{name}={v}"))
        })
        .collect();

    let extra_cookies = if extra_parts.is_empty() {
        None
    } else {
        Some(extra_parts.join("; "))
    };

    Ok(Credentials {
        auth_token,
        ct0,
        extra_cookies,
    })
}
