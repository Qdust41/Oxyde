# Oxyde — Tauri + SurrealDB Scaffold Design

**Date:** 2026-04-14
**Stack:** Tauri v2, SvelteKit (Svelte 5), SurrealDB 2.x (remote WebSocket), pnpm

---

## 1. Architecture & File Layout

### Rust (`src-tauri/src/`)

```
src-tauri/src/
├── main.rs          # #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
│                    # fn main() → oxyde_lib::run()
├── lib.rs           # Tauri Builder: managed state, register all commands, plugin init
├── db.rs            # AppState struct + init_db()
├── models.rs        # User, Room, Message, Contact — serde + SurrealDB derives
├── error.rs         # AppError enum → impl From<AppError> for String
└── commands/
    ├── mod.rs       # pub mod user; pub mod chat;
    ├── user.rs      # signup, signin, signout, get_me, update_profile,
    │                #   get_contacts, add_contact
    └── chat.rs      # send_message, get_messages, delete_message,
                     #   get_rooms, create_room, subscribe_room, unsubscribe_room
```

### `db.rs` — AppState

```rust
pub struct AppState {
    pub db: Arc<Surreal<Client>>,
    pub token: Mutex<Option<String>>,
    pub subscriptions: Mutex<HashMap<Uuid, JoinHandle<()>>>,
}
```

- `db`: single long-lived WebSocket connection to SurrealDB
- `token`: JWT returned by SurrealDB Record Auth on signin, cleared on signout
- `subscriptions`: tracks spawned LIVE query tasks by UUID for clean cancellation

`init_db()` connects to `ws://localhost:8000`, selects namespace and database.

### `lib.rs`

Wires `AppState` into `tauri::Builder::manage()`, registers all commands via `invoke_handler`, initialises plugins.

### SurrealDB files (`surreal/`)

```
surreal/
├── schema.surql     # DEFINE TABLE + DEFINE FIELD for all tables
└── auth.surql       # DEFINE ACCESS account (Record Auth, JWT HS512)
```

### `Cargo.toml` additions

```toml
surrealdb = "2"
tokio     = { version = "1", features = ["full"] }
thiserror = "1"
uuid      = { version = "1", features = ["v4"] }
```

---

## 2. Data Flow — LIVE Queries → Frontend Events

```
Frontend                    Tauri Command               SurrealDB
   │                              │                         │
   │──invoke("subscribe_room")───▶│                         │
   │                              │──LIVE SELECT * FROM─────▶│
   │                              │   message WHERE          │
   │                              │   room = $room_id        │
   │                              │                         │
   │                              │  spawn tokio::task       │
   │                              │  (holds LIVE stream)     │
   │                              │◀────────────────────────│
   │◀──Ok(live_query_id: String)──│                         │
   │                              │                         │
   │  [new message inserted]      │                         │
   │                              │◀──LIVE notification─────│
   │                              │                         │
   │                              │ app_handle.emit(         │
   │                              │   "chat:message",        │
   │                              │   MessagePayload)        │
   │◀──Tauri event───────────────│                         │
   │  listen("chat:message", cb)  │                         │
```

**Key details:**
- `subscribe_room` returns a `String` (LIVE query UUID) to the frontend
- Frontend stores UUID and calls `unsubscribe_room(uuid)` on component unmount
- Spawned task holds `AppHandle` clone — required to emit events from background
- `unsubscribe_room` aborts the `JoinHandle` and sends `KILL <uuid>` to SurrealDB
- One task per room; map lives in `AppState.subscriptions`

---

## 3. SurrealQL Schema & Auth

### `surreal/schema.surql`

```sql
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string;
DEFINE FIELD email    ON user TYPE string;
DEFINE FIELD password ON user TYPE string;
DEFINE FIELD avatar   ON user TYPE option<string>;
DEFINE FIELD created  ON user TYPE datetime DEFAULT time::now();
DEFINE INDEX email_idx ON user FIELDS email UNIQUE;

DEFINE TABLE room SCHEMAFULL;
DEFINE FIELD name    ON room TYPE string;
DEFINE FIELD created ON room TYPE datetime DEFAULT time::now();

DEFINE TABLE message SCHEMAFULL;
DEFINE FIELD room    ON message TYPE record<room>;
DEFINE FIELD author  ON message TYPE record<user>;
DEFINE FIELD body    ON message TYPE string;
DEFINE FIELD created ON message TYPE datetime DEFAULT time::now();

DEFINE TABLE contact SCHEMAFULL;
DEFINE FIELD owner  ON contact TYPE record<user>;
DEFINE FIELD target ON contact TYPE record<user>;
DEFINE INDEX unique_contact ON contact FIELDS owner, target UNIQUE;
```

### `surreal/auth.surql`

```sql
DEFINE ACCESS account ON DATABASE TYPE RECORD
  SIGNUP (
    CREATE user SET
      email    = $email,
      username = $username,
      password = crypto::argon2::generate($password)
  )
  SIGNIN (
    SELECT * FROM user
    WHERE email = $email
    AND crypto::argon2::compare(password, $password)
  )
  WITH JWT ALGORITHM HS512 KEY $jwt_secret;
```

**Note:** `$jwt_secret` must be injected via env var or Tauri's secure store — never hardcoded.

**Schema decisions:**
- `password` is an explicit field (required in schemafull mode)
- `contact` uses a separate table with `owner`/`target` record links — supports bidirectional queries without array fields on `user`

---

## 4. Error Handling

### `error.rs`

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] surrealdb::Error),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Subscription error: {0}")]
    Subscription(String),
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

pub fn into_err<E: Into<AppError>>(e: E) -> String {
    e.into().to_string()
}
```

All Tauri commands return `Result<T, String>`. Use `.map_err(into_err)` at command boundaries. Internal functions use `Result<T, AppError>`.

---

## 5. Command Surface

### `commands/user.rs`
| Command | Args | Returns |
|---|---|---|
| `signup` | email, username, password | `User` |
| `signin` | email, password | `String` (JWT) |
| `signout` | — | `()` |
| `get_me` | — | `User` |
| `update_profile` | username?, avatar? | `User` |
| `get_contacts` | — | `Vec<User>` |
| `add_contact` | user_id | `Contact` (stub) |

### `commands/chat.rs`
| Command | Args | Returns |
|---|---|---|
| `create_room` | name | `Room` |
| `get_rooms` | — | `Vec<Room>` |
| `send_message` | room_id, body | `Message` |
| `get_messages` | room_id | `Vec<Message>` |
| `delete_message` | message_id | `()` |
| `subscribe_room` | room_id | `String` (UUID) — `AppHandle` injected by Tauri |
| `unsubscribe_room` | uuid | `()` |

---

## 6. Models (`models.rs`)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Thing,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
    pub created: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: Thing,
    pub name: String,
    pub created: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Thing,
    pub room: Thing,
    pub author: Thing,
    pub body: String,
    pub created: Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: Thing,
    pub owner: Thing,
    pub target: Thing,
}
```

`Thing` and `Datetime` from `surrealdb::sql`.
