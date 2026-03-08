// GraphQL endpoint URLs with query IDs extracted from Twitter main.js (2026-03-07).

// Read endpoints
pub const USER_TWEETS: &str = "https://x.com/i/api/graphql/ix7iRrsAvfXyGUQ06Z7krA/UserTweets";
pub const USER_TWEETS_AND_REPLIES: &str =
    "https://x.com/i/api/graphql/RCpRL9JyzOSO5qS6YDOg7w/UserTweetsAndReplies";
pub const FOLLOWERS: &str = "https://x.com/i/api/graphql/ggGqWO5y_c4Iu58dyHnbzg/Followers";
pub const FOLLOWING: &str = "https://x.com/i/api/graphql/NElglO5nnh78FWMvYQuwDw/Following";
pub const SEARCH_TIMELINE: &str =
    "https://api.x.com/graphql/nWemVnGJ6A5eQAR5-oQeAg/SearchTimeline";
pub const TWEET_RESULT_BY_REST_ID: &str =
    "https://x.com/i/api/graphql/u-HcuvGsZT0ZoxfFDQWNeg/TweetResultByRestId";
pub const TWEET_DETAIL: &str =
    "https://x.com/i/api/graphql/16nxv6mC_2VaBvBwY2V85g/TweetDetail";
pub const USER_BY_SCREEN_NAME: &str =
    "https://x.com/i/api/graphql/pLsOiyHJ1eFwPJlNmLp4Bg/UserByScreenName";

pub const HOME_TIMELINE: &str =
    "https://x.com/i/api/graphql/snvCaalBp51MiDb3-nGblg/HomeTimeline";

// Write endpoints (POST)
pub const CREATE_TWEET: &str =
    "https://x.com/i/api/graphql/uY34Pldm6W89yqswRmPMSQ/CreateTweet";
pub const FAVORITE_TWEET: &str =
    "https://x.com/i/api/graphql/lI07N6Otwv1PhnEgXILM7A/FavoriteTweet";
pub const UNFAVORITE_TWEET: &str =
    "https://x.com/i/api/graphql/ZYKSe-w7KEslx3JhSIk5LA/UnfavoriteTweet";
pub const CREATE_RETWEET: &str =
    "https://x.com/i/api/graphql/mbRO74GrOvSfRcJnlMapnQ/CreateRetweet";
pub const DELETE_RETWEET: &str =
    "https://x.com/i/api/graphql/ZyZigVsNiFO6v1dEks1eWg/DeleteRetweet";
pub const CREATE_FRIENDSHIP: &str = "https://x.com/i/api/1.1/friendships/create.json";
pub const DESTROY_FRIENDSHIP: &str = "https://x.com/i/api/1.1/friendships/destroy.json";

// Account
pub const ACCOUNT_MULTI_LIST: &str = "https://x.com/i/api/1.1/account/multi/list.json";
