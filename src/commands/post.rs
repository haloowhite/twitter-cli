use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output;

pub async fn post(client: &TwitterClient, text: &str) -> Result<()> {
    let resp = client.create_tweet(text, None, None).await?;
    match output::extract_single_tweet(&resp) {
        Some(t) => println!("{}", serde_json::to_string_pretty(&t)?),
        None => {
            eprintln!("Warning: could not extract tweet from response");
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}

pub async fn reply(client: &TwitterClient, tweet_id: &str, text: &str) -> Result<()> {
    let resp = client.create_tweet(text, Some(tweet_id), None).await?;
    match output::extract_single_tweet(&resp) {
        Some(t) => println!("{}", serde_json::to_string_pretty(&t)?),
        None => {
            eprintln!("Warning: could not extract tweet from response");
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}

pub async fn quote(client: &TwitterClient, tweet_id: &str, text: &str) -> Result<()> {
    let resp = client.create_tweet(text, None, Some(tweet_id)).await?;
    match output::extract_single_tweet(&resp) {
        Some(t) => println!("{}", serde_json::to_string_pretty(&t)?),
        None => {
            eprintln!("Warning: could not extract tweet from response");
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}
