use rquest::header::{HeaderMap, HeaderValue};

pub const BEARER_TOKEN: &str = "Bearer AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA";

pub const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";

pub fn build_headers(csrf_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("authority", HeaderValue::from_static("twitter.com"));
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("zh-CN,zh;q=0.9,zh-TW;q=0.8"),
    );
    headers.insert(
        "authorization",
        HeaderValue::from_static(BEARER_TOKEN),
    );
    headers.insert("content-type", HeaderValue::from_static("application/json"));
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Google Chrome\";v=\"136\", \"Chromium\";v=\"136\", \"Not.A/Brand\";v=\"99\"",
        ),
    );
    headers.insert("sec-ch-ua-arch", HeaderValue::from_static("\"arm\""));
    headers.insert("sec-ch-ua-bitness", HeaderValue::from_static("\"64\""));
    headers.insert(
        "sec-ch-ua-full-version",
        HeaderValue::from_static("\"136.0.7103.113\""),
    );
    headers.insert(
        "sec-ch-ua-full-version-list",
        HeaderValue::from_static("\"Google Chrome\";v=\"136.0.7103.113\", \"Chromium\";v=\"136.0.7103.113\", \"Not.A/Brand\";v=\"99.0.0.0\""),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-model", HeaderValue::from_static("\"\""));
    headers.insert(
        "sec-ch-ua-platform",
        HeaderValue::from_static("\"macOS\""),
    );
    headers.insert(
        "sec-ch-ua-platform-version",
        HeaderValue::from_static("\"26.1.0\""),
    );
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert(
        "x-twitter-active-user",
        HeaderValue::from_static("yes"),
    );
    headers.insert(
        "x-twitter-auth-type",
        HeaderValue::from_static("OAuth2Session"),
    );
    headers.insert(
        "x-twitter-client-language",
        HeaderValue::from_static("en"),
    );
    headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));

    if let Ok(val) = HeaderValue::from_str(csrf_token) {
        headers.insert("x-csrf-token", val);
    }

    headers
}

pub fn build_cookie_header(auth_token: &str, csrf_token: &str) -> String {
    format!("auth_token={auth_token}; ct0={csrf_token}")
}
