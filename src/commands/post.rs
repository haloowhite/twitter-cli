use anyhow::Result;
use crate::api::client::TwitterClient;

pub async fn post(client: &TwitterClient, text: &str) -> Result<()> {
    let resp = client.create_tweet(text, None, None).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn reply(client: &TwitterClient, tweet_id: &str, text: &str) -> Result<()> {
    let resp = client.create_tweet(text, Some(tweet_id), None).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn quote(client: &TwitterClient, tweet_id: &str, text: &str) -> Result<()> {
    let resp = client.create_tweet(text, None, Some(tweet_id)).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}
