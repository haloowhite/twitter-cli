use anyhow::Result;
use crate::api::client::TwitterClient;
use crate::output;

pub async fn get_following(client: &TwitterClient, user_id: &str, limit: u32) -> Result<()> {
    let resp = client.get_following(user_id, limit, None).await?;
    let users = output::extract_users(&resp);
    println!("{}", serde_json::to_string_pretty(&users)?);
    Ok(())
}

pub async fn get_followers(client: &TwitterClient, user_id: &str, limit: u32) -> Result<()> {
    let resp = client.get_followers(user_id, limit, None).await?;
    let users = output::extract_users(&resp);
    println!("{}", serde_json::to_string_pretty(&users)?);
    Ok(())
}

pub async fn follow(client: &TwitterClient, user_id: &str) -> Result<()> {
    let resp = client.follow_user(user_id).await?;
    let result = output::extract_action_result(&resp, "follow");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn unfollow(client: &TwitterClient, user_id: &str) -> Result<()> {
    let resp = client.unfollow_user(user_id).await?;
    let result = output::extract_action_result(&resp, "unfollow");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn lookup_user(client: &TwitterClient, screen_name: &str) -> Result<()> {
    let resp = client.get_user_by_screen_name(screen_name).await?;
    match output::extract_single_user(&resp) {
        Some(u) => println!("{}", serde_json::to_string_pretty(&u)?),
        None => {
            eprintln!("Warning: could not extract user, printing raw response");
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}

pub async fn get_me(client: &TwitterClient) -> Result<()> {
    let resp = client.get_me().await?;
    match output::extract_me_user(&resp) {
        Some(u) => println!("{}", serde_json::to_string_pretty(&u)?),
        None => {
            eprintln!("Warning: could not extract user, printing raw response");
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}
