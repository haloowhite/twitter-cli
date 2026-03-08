use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output;

pub async fn search(client: &TwitterClient, query: &str, limit: u32) -> Result<()> {
    let resp = client.search_tweets(query, limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    println!("{}", serde_json::to_string_pretty(&tweets)?);
    Ok(())
}
