///! This file contains the JSON objects that can be retrieved

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserStats{
    pub post_count: u64,
    pub del_post_count: u64,
    pub edit_count: u64,
    pub favorite_count: u64,
    pub wiki_count: u64,
    pub forum_post_count: u64,
    pub note_count: u64,
    pub comment_count: u64,
    pub blip_count: u64,
    pub set_count: u64,
    pub pool_update_count: u64,
    pub pos_user_records: u64,
    pub neutral_user_records: u64,
    pub neg_user_records: u64
}

use serde_json::value::Value as JsonValue;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User{
    pub id: u64,
    pub name: String,
    pub level: u64,
    pub avatar_id: u64,
    pub stats: UserStats,

    /// Time of register.
    ///
    /// NOTE: This timestamp is already pretty-printed, in the "YYYY-MM-DD HH:MM" format,
    /// unlike the ones found in both posts and pools, which use the UNIX epoch as a base
    pub created_at: String,

    /// Artists tags this user is linked to
    pub artist_tags: Vec<String>,

    /// According to the API reference this is a JSON Object for user tag subscriptions.
    /// NOTE: Requires further research in order to be made functional.
    pub tag_subscriptions: JsonValue
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment{
    pub id: u64,
    pub post_id: u64,

    /// Time of creation, when this comment was posted.
    ///
    /// NOTE: This timestamp is already pretty-printed, in the "YYYY-MM-DD HH:MM" format,
    /// unlike the ones found in both posts and pools, which use the UNIX epoch as a base
    pub created_at: String,

    pub creator: String,
    pub creator_id: u64,

    pub body: String,
    pub score: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommentPool(pub Vec<Comment>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Timestamp{
    /// (Ignored) Should always be "Time"
    pub json_class: String,
    /// Seconds elapsed since the UNIX epoch
    pub s: u64,
    /// Nanoseconds elapsed since the last second
    pub n: u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Post{
    /// Unique identification number
    pub id: u64,
    pub tags: String,
    pub description: String,
    pub author: String,
    pub source: String,
    pub md5: String,

    pub artist: Vec<String>,
    pub sources: Vec<String>,

    /// "Safety" rating of the post, either:
    ///     "s" => Safe
    ///     "q" => Questionable
    ///     "e" => Explicit
    ///
    /// Should be retrieved using Post::rating()
    pub rating: String,
    pub score: i64,
    pub fav_count: u64,

    pub file_size: u64,
    pub file_url: String,
    pub file_ext: String,

    pub width: u64,
    pub height: u64,

    pub preview_url: String,
    pub preview_width: u64,
    pub preview_height: u64,

    pub sample_url: String,
    pub sample_width: u64,
    pub sample_height: u64,

    pub has_comments: bool,
    pub has_notes: bool,
    pub has_children: bool,
    pub children: String,
    pub parent_id: Option<u64>,

    pub created_at: Timestamp,
    pub creator_id: u64,
    pub change: u64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pool{
    pub id: u64,
    pub name: String,
    pub description: String,
    pub post_count: u64,

    pub posts: Vec<Post>,

    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub is_active: bool,
    pub is_locked: bool,
    pub user_id: u64
}
