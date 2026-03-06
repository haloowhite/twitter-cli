// GraphQL endpoint URLs with query IDs from the Twitter web client.
// These query IDs may change when Twitter updates their web app.

// Read endpoints
pub const USER_TWEETS: &str = "https://x.com/i/api/graphql/gQlOy4mD5C8M8fYxqa0FJg/UserTweets";
pub const USER_TWEETS_AND_REPLIES: &str =
    "https://x.com/i/api/graphql/bAO1gMk6B7iDoOp2sDCDyw/UserTweetsAndReplies";
pub const FOLLOWERS: &str = "https://x.com/i/api/graphql/o1YfmoGa-hb8Z6yQhoIBhg/Followers";
pub const FOLLOWING: &str = "https://x.com/i/api/graphql/ZxuX4tC6kWz9M8pe1i-Gdg/Following";
pub const SEARCH_TIMELINE: &str =
    "https://x.com/i/api/graphql/NiZ1seU-Qm1TUiThEaWXKA/SearchTimeline";
pub const TWEET_RESULT_BY_REST_ID: &str =
    "https://api.x.com/graphql/kLXoXTloWpv9d2FSXRg-Tg/TweetResultByRestId";
pub const TWEET_DETAIL: &str =
    "https://x.com/i/api/graphql/97JF30KziU00483E_8elBA/TweetDetail";
pub const USER_BY_SCREEN_NAME: &str =
    "https://x.com/i/api/graphql/mCbpQvZAw6zu_4PvuAUVVQ/UserByScreenName";

// Write endpoints (POST)
pub const CREATE_TWEET: &str =
    "https://x.com/i/api/graphql/oB-5XsHNAbjvARJEc8CZFw/CreateTweet";
pub const FAVORITE_TWEET: &str =
    "https://x.com/i/api/graphql/lI07N6OdyUlbRl84p-7-nQ/FavoriteTweet";
pub const UNFAVORITE_TWEET: &str =
    "https://x.com/i/api/graphql/ZYKSe-w7KEslx3JhSIk5LA/UnfavoriteTweet";
pub const CREATE_RETWEET: &str =
    "https://x.com/i/api/graphql/ojPdsZsimiJrUGLR1sjUtA/CreateRetweet";
pub const DELETE_RETWEET: &str =
    "https://x.com/i/api/graphql/iQtK4dl5hBmXewYZuEOKVw/DeleteRetweet";
pub const CREATE_FRIENDSHIP: &str = "https://x.com/i/api/1.1/friendships/create.json";
pub const DESTROY_FRIENDSHIP: &str = "https://x.com/i/api/1.1/friendships/destroy.json";
