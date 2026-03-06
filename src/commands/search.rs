use anyhow::Result;
use crate::api::client::TwitterClient;

pub async fn search(client: &TwitterClient, query: &str, limit: u32) -> Result<()> {
    let resp = client.search_tweets(query, limit, None).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}
