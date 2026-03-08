use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output::{self, CompactTweet};

pub async fn search(client: &TwitterClient, query: &str, limit: u32, compact: bool) -> Result<()> {
    let resp = client.search_tweets(query, limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    if compact {
        let compact: Vec<CompactTweet> = tweets.iter().map(CompactTweet::from_tweet).collect();
        println!("{}", serde_json::to_string_pretty(&compact)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&tweets)?);
    }
    Ok(())
}
