use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId};
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct User {
    pub id: RecordId,
    pub username: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub created: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Room {
    pub id: RecordId,
    pub name: Option<String>,
    pub kind: String,
    pub created_by: Option<RecordId>,
    pub direct_key: Option<String>,
    pub created: Datetime,
    pub updated: Option<Datetime>,
    pub last_message: Option<Message>,
    pub unread_count: Option<i64>,
    pub other_user: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[allow(dead_code)]
pub struct RoomMember {
    pub id: RecordId,
    pub room: RecordId,
    pub user: RecordId,
    pub role: String,
    pub joined: Datetime,
    pub last_read_at: Option<Datetime>,
    pub muted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Message {
    pub id: RecordId,
    pub room: RecordId,
    pub author: RecordId,
    pub author_username: Option<String>,
    pub body: String,
    pub created: Datetime,
    pub updated: Option<Datetime>,
    pub deleted: Option<bool>,
    pub reply_to: Option<RecordId>,
    pub reactions: Option<Vec<MessageReactionSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MessageReaction {
    pub id: RecordId,
    pub message: RecordId,
    pub user: RecordId,
    pub emoji: String,
    pub created: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MessageReactionSummary {
    pub emoji: String,
    pub count: i64,
    pub reacted_by_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Contact {
    pub id: RecordId,
    pub owner: RecordId,
    pub target: RecordId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_compile() {
        // Structural compile check — verifies field types resolve correctly.
        fn _assert_serialize<T: Serialize + for<'de> Deserialize<'de>>() {}
        _assert_serialize::<User>();
        _assert_serialize::<Room>();
        _assert_serialize::<RoomMember>();
        _assert_serialize::<Message>();
        _assert_serialize::<MessageReaction>();
        _assert_serialize::<MessageReactionSummary>();
        _assert_serialize::<Contact>();
    }
}
