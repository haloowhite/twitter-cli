use anyhow::Result;
use crate::api::client::TwitterClient;

pub async fn get_following(client: &TwitterClient, user_id: &str, limit: u32) -> Result<()> {
    let resp = client.get_following(user_id, limit, None).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn get_followers(client: &TwitterClient, user_id: &str, limit: u32) -> Result<()> {
    let resp = client.get_followers(user_id, limit, None).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn follow(client: &TwitterClient, user_id: &str) -> Result<()> {
    let resp = client.follow_user(user_id).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn unfollow(client: &TwitterClient, user_id: &str) -> Result<()> {
    let resp = client.unfollow_user(user_id).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn lookup_user(client: &TwitterClient, screen_name: &str) -> Result<()> {
    let resp = client.get_user_by_screen_name(screen_name).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}
