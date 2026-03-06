use anyhow::Result;
use crate::api::client::TwitterClient;

pub async fn like(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.like_tweet(tweet_id).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn unlike(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.unlike_tweet(tweet_id).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn retweet(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.retweet(tweet_id).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn unretweet(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.unretweet(tweet_id).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}
