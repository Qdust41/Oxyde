use serde::{Deserialize, Serialize};
use surrealdb::types::{Datetime, RecordId};
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct User {
    pub id: RecordId,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
    pub created: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Room {
    pub id: RecordId,
    pub name: String,
    pub created: Datetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Message {
    pub id: RecordId,
    pub room: RecordId,
    pub author: RecordId,
    pub author_username: Option<String>,
    pub body: String,
    pub created: Datetime,
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
        _assert_serialize::<Message>();
        _assert_serialize::<Contact>();
    }
}
