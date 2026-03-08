use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output;

pub async fn like(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.like_tweet(tweet_id).await?;
    let result = output::extract_action_result(&resp, "like");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn unlike(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.unlike_tweet(tweet_id).await?;
    let result = output::extract_action_result(&resp, "unlike");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn retweet(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.retweet(tweet_id).await?;
    let result = output::extract_action_result(&resp, "retweet");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn unretweet(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.unretweet(tweet_id).await?;
    let result = output::extract_action_result(&resp, "unretweet");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn delete(client: &TwitterClient, tweet_id: &str) -> Result<()> {
    let resp = client.delete_tweet(tweet_id).await?;
    let result = output::extract_action_result(&resp, "delete");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
