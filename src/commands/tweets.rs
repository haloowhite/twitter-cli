use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output;

pub async fn get_tweets(client: &TwitterClient, user_id: &str, limit: u32) -> Result<()> {
    let resp = client.get_user_tweets(user_id, limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    println!("{}", serde_json::to_string_pretty(&tweets)?);
    Ok(())
}

pub async fn get_replies(client: &TwitterClient, user_id: &str, limit: u32) -> Result<()> {
    let resp = client.get_user_replies(user_id, limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    println!("{}", serde_json::to_string_pretty(&tweets)?);
    Ok(())
}
