use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output::{self, CompactTweet};

fn print_tweets(tweets: &[output::TweetOutput], compact: bool) -> Result<()> {
    if compact {
        let compact: Vec<CompactTweet> = tweets.iter().map(CompactTweet::from_tweet).collect();
        println!("{}", serde_json::to_string_pretty(&compact)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&tweets)?);
    }
    Ok(())
}

pub async fn get_timeline(client: &TwitterClient, limit: u32, compact: bool) -> Result<()> {
    let resp = client.get_home_timeline(limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    print_tweets(&tweets, compact)
}

pub async fn get_tweets(client: &TwitterClient, user_id: &str, limit: u32, compact: bool) -> Result<()> {
    let resp = client.get_user_tweets(user_id, limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    print_tweets(&tweets, compact)
}

pub async fn get_replies(client: &TwitterClient, user_id: &str, limit: u32, compact: bool) -> Result<()> {
    let resp = client.get_user_replies(user_id, limit, None).await?;
    let tweets = output::extract_tweets(&resp);
    print_tweets(&tweets, compact)
}
