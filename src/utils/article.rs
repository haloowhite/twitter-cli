use serde_json::Value;

/// Check if a tweet is an article tweet.
pub fn is_article_tweet(tweet_data: &Value) -> bool {
    tweet_data
        .get("article")
        .and_then(|a| a.get("article_results"))
        .is_some()
}

/// Extract article title from tweet data.
pub fn extract_article_title(tweet_data: &Value) -> Option<String> {
    tweet_data
        .get("article")?
        .get("article_results")?
        .get("result")?
        .get("title")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

/// Extract article cover image URL.
pub fn extract_article_cover_image_url(tweet_data: &Value) -> Option<String> {
    tweet_data
        .get("article")?
        .get("article_results")?
        .get("result")?
        .get("cover_media")?
        .get("media_info")?
        .get("original_img_url")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
}

/// Process article content into markdown.
pub fn process_article_content(tweet_data: &Value) -> Option<ArticleContent> {
    if !is_article_tweet(tweet_data) {
        return None;
    }

    let title = extract_article_title(tweet_data);
    let cover_image_url = extract_article_cover_image_url(tweet_data);

    let result = tweet_data
        .get("article")?
        .get("article_results")?
        .get("result")?;

    let content_state = result.get("content_state").unwrap_or(&Value::Null);
    let blocks = content_state
        .get("blocks")
        .and_then(|b| b.as_array())
        .cloned()
        .unwrap_or_default();

    let text = convert_blocks_to_markdown(&blocks);

    Some(ArticleContent {
        title,
        cover_image_url,
        text,
    })
}

#[derive(Debug)]
pub struct ArticleContent {
    pub title: Option<String>,
    pub cover_image_url: Option<String>,
    pub text: String,
}

fn convert_blocks_to_markdown(blocks: &[Value]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut ordered_counter = 0u32;

    for block in blocks {
        let block_type = block
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("unstyled");

        if block_type == "atomic" {
            continue;
        }

        let text = block
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        if text.is_empty() {
            continue;
        }

        if block_type != "ordered-list-item" {
            ordered_counter = 0;
        }

        let formatted = match block_type {
            "header-one" => format!("# {text}"),
            "header-two" => format!("## {text}"),
            "header-three" => format!("### {text}"),
            "blockquote" => format!("> {text}"),
            "unordered-list-item" => format!("- {text}"),
            "ordered-list-item" => {
                ordered_counter += 1;
                format!("{ordered_counter}. {text}")
            }
            "code-block" => format!("```\n{text}\n```"),
            _ => text.to_string(),
        };

        parts.push(formatted);
    }

    parts.join("\n\n")
}
