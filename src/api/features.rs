use serde_json::{json, Value};

fn core_features() -> Value {
    json!({
        "rweb_tipjar_consumption_enabled": true,
        "verified_phone_label_enabled": false,
        "creator_subscriptions_tweet_preview_api_enabled": true,
        "responsive_web_graphql_timeline_navigation_enabled": true,
        "responsive_web_graphql_skip_user_profile_image_extensions_enabled": false,
        "communities_web_enable_tweet_community_results_fetch": true,
        "c9s_tweet_anatomy_moderator_badge_enabled": true,
        "articles_preview_enabled": true,
        "responsive_web_edit_tweet_api_enabled": true,
        "graphql_is_translatable_rweb_tweet_is_translatable_enabled": true,
        "view_counts_everywhere_api_enabled": true,
        "longform_notetweets_consumption_enabled": true,
        "responsive_web_twitter_article_tweet_consumption_enabled": true,
        "tweet_awards_web_tipping_enabled": false,
        "creator_subscriptions_quote_tweet_preview_enabled": false,
        "freedom_of_speech_not_reach_fetch_enabled": true,
        "standardized_nudges_misinfo": true,
        "tweet_with_visibility_results_prefer_gql_limited_actions_policy_enabled": true,
        "longform_notetweets_rich_text_read_enabled": true,
        "longform_notetweets_inline_media_enabled": true,
        "responsive_web_enhance_cards_enabled": false,
        "post_ctas_fetch_enabled": true,
        "responsive_web_grok_annotations_enabled": false,
    })
}

fn grok_features() -> Value {
    json!({
        "responsive_web_grok_analyze_button_fetch_trends_enabled": false,
        "responsive_web_grok_analyze_post_followups_enabled": true,
        "responsive_web_jetfuel_frame": true,
        "responsive_web_grok_share_attachment_enabled": true,
        "responsive_web_grok_show_grok_translated_post": false,
        "responsive_web_grok_analysis_button_from_backend": true,
        "responsive_web_grok_image_annotation_enabled": true,
        "responsive_web_grok_imagine_annotation_enabled": true,
        "responsive_web_grok_community_note_auto_translation_is_enabled": false,
    })
}

fn profile_features() -> Value {
    json!({
        "profile_label_improvements_pcf_label_in_post_enabled": true,
        "responsive_web_profile_redirect_enabled": false,
        "premium_content_api_read_enabled": false,
        "rweb_video_screen_enabled": false,
    })
}

fn merge(base: Value, extra: Value) -> Value {
    let mut result = base;
    if let (Some(r), Some(e)) = (result.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            r.insert(k.clone(), v.clone());
        }
    }
    result
}

pub fn features_followers() -> Value {
    merge(
        core_features(),
        json!({
            "responsive_web_graphql_exclude_directive_enabled": true,
            "tweetypie_unmention_optimization_enabled": true,
            "rweb_video_timestamps_enabled": true,
            "tweet_with_visibility_results_prefer_gql_media_interstitial_enabled": true,
        }),
    )
}

pub fn features_followings() -> Value {
    features_followers()
}

pub fn features_tweets() -> Value {
    merge(
        core_features(),
        json!({
            "responsive_web_graphql_exclude_directive_enabled": true,
            "tweetypie_unmention_optimization_enabled": true,
            "rweb_video_timestamps_enabled": true,
        }),
    )
}

pub fn features_replies() -> Value {
    let mut f = merge(core_features(), profile_features());
    f = merge(f, grok_features());
    f = merge(f, json!({"payments_enabled": false}));
    f
}

pub fn features_search() -> Value {
    let mut f = merge(core_features(), grok_features());
    f = merge(
        f,
        json!({
            "rweb_video_screen_enabled": false,
            "payments_enabled": false,
            "rweb_xchat_enabled": false,
            "profile_label_improvements_pcf_label_in_post_enabled": true,
            "premium_content_api_read_enabled": false,
            "responsive_web_grok_analysis_button_from_backend": false,
        }),
    );
    f
}

pub fn features_tweet_detail() -> Value {
    let mut f = merge(core_features(), grok_features());
    f = merge(
        f,
        json!({
            "premium_content_api_read_enabled": false,
            "profile_label_improvements_pcf_label_in_post_enabled": true,
            "responsive_web_profile_redirect_enabled": false,
            "responsive_web_grok_analyze_post_followups_enabled": false,
        }),
    );
    f
}

pub fn features_tweet_detail_context() -> Value {
    let f = merge(core_features(), profile_features());
    merge(f, grok_features())
}

/// Features for CreateTweet mutation
pub fn features_create_tweet() -> Value {
    merge(
        core_features(),
        json!({
            "responsive_web_graphql_exclude_directive_enabled": true,
            "tweetypie_unmention_optimization_enabled": true,
            "rweb_video_timestamps_enabled": true,
        }),
    )
}
