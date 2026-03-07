mod api;
mod auth;
mod commands;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

use api::client::TwitterClient;
use auth::storage::load_credentials;

#[derive(Parser)]
#[command(name = "twitter-cli", about = "Twitter/X CLI tool", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with Twitter
    Auth {
        /// Extract cookies from browser (chrome, firefox, edge, safari)
        #[arg(long)]
        browser: Option<String>,

        /// Provide auth_token directly
        #[arg(long)]
        token: Option<String>,
    },

    /// Get user tweets
    Tweets {
        /// User ID
        user_id: String,

        /// Number of tweets to fetch
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Get user replies
    Replies {
        /// User ID
        user_id: String,

        /// Number of replies to fetch
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Get user's following list
    Following {
        /// User ID
        user_id: String,

        /// Number of results
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Get user's followers list
    Followers {
        /// User ID
        user_id: String,

        /// Number of results
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Search tweets
    Search {
        /// Search query
        query: String,

        /// Number of results
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Get tweet detail
    Detail {
        /// Tweet ID
        tweet_id: String,

        /// Include conversation context
        #[arg(long)]
        context: bool,
    },

    /// Post a new tweet
    Post {
        /// Tweet content
        text: String,
    },

    /// Reply to a tweet
    Reply {
        /// Tweet ID to reply to
        tweet_id: String,

        /// Reply content
        text: String,
    },

    /// Quote a tweet
    Quote {
        /// Tweet ID to quote
        tweet_id: String,

        /// Quote content
        text: String,
    },

    /// Like a tweet
    Like {
        /// Tweet ID
        tweet_id: String,
    },

    /// Unlike a tweet
    Unlike {
        /// Tweet ID
        tweet_id: String,
    },

    /// Retweet a tweet
    Retweet {
        /// Tweet ID
        tweet_id: String,
    },

    /// Undo retweet
    Unretweet {
        /// Tweet ID
        tweet_id: String,
    },

    /// Follow a user
    Follow {
        /// User ID
        user_id: String,
    },

    /// Unfollow a user
    Unfollow {
        /// User ID
        user_id: String,
    },

    /// Look up user by screen name
    User {
        /// Screen name (handle)
        screen_name: String,
    },

    /// Get current authenticated user info
    Me,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { browser, token } => {
            if let Some(browser) = browser {
                commands::auth::run_auth_browser(&browser)?;
            } else if let Some(token) = token {
                commands::auth::run_auth_token(&token)?;
            } else {
                anyhow::bail!("Provide --browser or --token");
            }
        }

        // All other commands require credentials
        cmd => {
            let creds = load_credentials()?;
            let client = TwitterClient::new(creds).await?;

            match cmd {
                Commands::Tweets { user_id, limit } => {
                    commands::tweets::get_tweets(&client, &user_id, limit).await?;
                }
                Commands::Replies { user_id, limit } => {
                    commands::tweets::get_replies(&client, &user_id, limit).await?;
                }
                Commands::Following { user_id, limit } => {
                    commands::users::get_following(&client, &user_id, limit).await?;
                }
                Commands::Followers { user_id, limit } => {
                    commands::users::get_followers(&client, &user_id, limit).await?;
                }
                Commands::Search { query, limit } => {
                    commands::search::search(&client, &query, limit).await?;
                }
                Commands::Detail { tweet_id, context } => {
                    let resp = if context {
                        client.get_tweet_detail_with_context(&tweet_id).await?
                    } else {
                        client.get_tweet_detail(&tweet_id).await?
                    };
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                }
                Commands::Post { text } => {
                    commands::post::post(&client, &text).await?;
                }
                Commands::Reply { tweet_id, text } => {
                    commands::post::reply(&client, &tweet_id, &text).await?;
                }
                Commands::Quote { tweet_id, text } => {
                    commands::post::quote(&client, &tweet_id, &text).await?;
                }
                Commands::Like { tweet_id } => {
                    commands::interact::like(&client, &tweet_id).await?;
                }
                Commands::Unlike { tweet_id } => {
                    commands::interact::unlike(&client, &tweet_id).await?;
                }
                Commands::Retweet { tweet_id } => {
                    commands::interact::retweet(&client, &tweet_id).await?;
                }
                Commands::Unretweet { tweet_id } => {
                    commands::interact::unretweet(&client, &tweet_id).await?;
                }
                Commands::Follow { user_id } => {
                    commands::users::follow(&client, &user_id).await?;
                }
                Commands::Unfollow { user_id } => {
                    commands::users::unfollow(&client, &user_id).await?;
                }
                Commands::User { screen_name } => {
                    commands::users::lookup_user(&client, &screen_name).await?;
                }
                Commands::Me => {
                    commands::users::get_me(&client).await?;
                }
                Commands::Auth { .. } => unreachable!(),
            }
        }
    }

    Ok(())
}
