# Persistent Login + Unused Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist the JWT session token across app launches so users sign in once per machine, and wire the three unused backend commands (`delete_message`, `update_profile`, `add_contact`) into the frontend UI.

**Architecture:** The Rust backend gains `tauri-plugin-store` for token persistence; `signin`/`signup`/`signout` commands save/clear the token, and a new `restore_session` command re-authenticates on startup. Frontend UI adds a message delete option to the context menu, an inline profile-edit form in the sidebar footer, and an inline add-contact form in the sidebar contacts section.

**Tech Stack:** Rust/Tauri 2, tauri-plugin-store 2, SurrealDB 3, Svelte 5, TypeScript

---

## File Map

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Add `tauri-plugin-store = "2"` |
| `src-tauri/src/lib.rs` | Register store plugin; add `restore_session` to invoke_handler |
| `src-tauri/src/commands/user.rs` | Add `AppHandle` to `signin`/`signup`/`signout`; save/clear token in store; add `restore_session` command |
| `src/routes/+page.svelte` | Replace `get_me` with `restore_session` in `init()`; add `deleteMessage`, `updateProfile`, `addContact` handlers; thread new props to components |
| `src/lib/components/ChatMain.svelte` | Add `user` prop and `onDeleteMessage` prop; add delete item to message context menu |
| `src/lib/components/Sidebar.svelte` | Add `onUpdateProfile` and `onAddContact` props; add inline profile-edit form; add inline add-contact form |

---

## Task 1: Add tauri-plugin-store dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the crate to Cargo.toml**

Open `src-tauri/Cargo.toml`. In the `[dependencies]` section, add after `tauri-plugin-opener`:

```toml
tauri-plugin-store = "2"
```

- [ ] **Step 2: Register the plugin in lib.rs**

Open `src-tauri/src/lib.rs`. The current builder chain is:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .setup(|app| {
```

Change it to:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_store::Builder::default().build())
    .setup(|app| {
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1
```

Expected: no errors. You will see it downloading and compiling tauri-plugin-store. If you see "use of undeclared crate or module `tauri_plugin_store`", double-check the Cargo.toml spelling.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs
git commit -m "feat: add tauri-plugin-store for session persistence"
```

---

## Task 2: Update user commands for token persistence

**Files:**
- Modify: `src-tauri/src/commands/user.rs`
- Modify: `src-tauri/src/lib.rs`

This task updates `signin`, `signup`, `signout` to save/clear the token in the store, and adds the new `restore_session` command.

- [ ] **Step 1: Replace the contents of commands/user.rs**

Replace the entire file with:

```rust
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::db::{AppState, SURREAL_ACCESS, SURREAL_DB, SURREAL_NS};
use crate::error::{into_err, AppError};
use crate::models::{Contact, User};

const SESSION_STORE: &str = "session.json";
const TOKEN_KEY: &str = "token";

/// Create a new user account via SurrealDB Record Auth SIGNUP.
/// Returns the created User record. Persists the JWT token to disk.
#[tauri::command]
pub async fn signup(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    email: String,
    username: String,
    password: String,
) -> Result<User, String> {
    let credentials = surrealdb::opt::auth::Record {
        access: SURREAL_ACCESS.to_string(),
        namespace: SURREAL_NS.to_string(),
        database: SURREAL_DB.to_string(),
        params: serde_json::json!({
            "email": email,
            "username": username,
            "password": password,
        }),
    };
    let token = state.db.signup(credentials).await.map_err(into_err)?;
    let token_str = token.access.into_insecure_token();
    *state.token.lock().unwrap() = Some(token_str.clone());
    save_token(&app_handle, &token_str)?;

    let mut result: Vec<User> = state
        .db
        .query("SELECT * FROM $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::Auth("signup succeeded but $auth not set".into())))
}

/// Authenticate an existing user via SurrealDB Record Auth SIGNIN.
/// Returns the JWT token string. Persists the token to disk.
#[tauri::command]
pub async fn signin(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    email: String,
    password: String,
) -> Result<String, String> {
    let credentials = surrealdb::opt::auth::Record {
        access: SURREAL_ACCESS.to_string(),
        namespace: SURREAL_NS.to_string(),
        database: SURREAL_DB.to_string(),
        params: serde_json::json!({
            "email": email,
            "password": password,
        }),
    };
    let token_str = state.db.signin(credentials).await.map_err(into_err)?.access.into_insecure_token();
    *state.token.lock().unwrap() = Some(token_str.clone());
    save_token(&app_handle, &token_str)?;
    Ok(token_str)
}

/// Clear the current session. Invalidates the token in state and removes it from disk.
#[tauri::command]
pub async fn signout(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    state.db.invalidate().await.map_err(into_err)?;
    *state.token.lock().unwrap() = None;
    clear_token(&app_handle)?;
    Ok(())
}

/// Attempt to restore a previous session from the persisted token on disk.
/// Authenticates the DB connection with the stored JWT.
/// Returns the authenticated User on success, or an error if no token exists
/// or the token is expired/invalid (in which case the stored token is also cleared).
#[tauri::command]
pub async fn restore_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<User, String> {
    let token_str = load_token(&app_handle)?.ok_or_else(|| {
        AppError::Auth("no saved session".into()).to_string()
    })?;

    match state.db.authenticate(surrealdb::opt::auth::Jwt::from(token_str.clone())).await {
        Ok(_) => {
            *state.token.lock().unwrap() = Some(token_str);

            let mut result: Vec<User> = state
                .db
                .query("SELECT * FROM $auth")
                .await
                .map_err(into_err)?
                .take(0)
                .map_err(into_err)?;

            result.pop().ok_or_else(|| into_err(AppError::Auth("session restored but $auth not set".into())))
        }
        Err(_) => {
            // Token expired or invalid — purge it so next launch shows auth screen.
            let _ = clear_token(&app_handle);
            *state.token.lock().unwrap() = None;
            Err(AppError::Auth("session expired, please sign in again".into()).to_string())
        }
    }
}

/// Fetch the currently authenticated user record.
#[tauri::command]
pub async fn get_me(state: State<'_, AppState>) -> Result<User, String> {
    let mut result: Vec<User> = state
        .db
        .query("SELECT * FROM $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::Auth("not authenticated".into())))
}

/// Update mutable profile fields. Only provided fields are changed.
#[tauri::command]
pub async fn update_profile(
    state: State<'_, AppState>,
    username: Option<String>,
    avatar: Option<String>,
) -> Result<User, String> {
    let mut result: Vec<User> = state
        .db
        .query(
            "UPDATE $auth SET
                username = $username ?? username,
                avatar   = $avatar   ?? avatar
             RETURN AFTER",
        )
        .bind(("username", username))
        .bind(("avatar", avatar))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::NotFound("user".into())))
}

/// Return the contacts list for the current user.
#[tauri::command]
pub async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let result: Vec<User> = state
        .db
        .query("SELECT target.* FROM contact WHERE owner = $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    Ok(result)
}

/// Add a user to the current user's contact list.
#[tauri::command]
pub async fn add_contact(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<Contact, String> {
    let mut result: Vec<Contact> = state
        .db
        .query("CREATE contact SET owner = $auth, target = type::record('user', $uid)")
        .bind(("uid", user_id))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::NotFound("contact after create".into())))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn save_token(app: &AppHandle, token: &str) -> Result<(), String> {
    let store = app.store(SESSION_STORE).map_err(|e| e.to_string())?;
    store.set(TOKEN_KEY, serde_json::json!(token));
    store.save().map_err(|e| e.to_string())
}

fn load_token(app: &AppHandle) -> Result<Option<String>, String> {
    let store = app.store(SESSION_STORE).map_err(|e| e.to_string())?;
    Ok(store.get(TOKEN_KEY).and_then(|v| v.as_str().map(String::from)))
}

fn clear_token(app: &AppHandle) -> Result<(), String> {
    let store = app.store(SESSION_STORE).map_err(|e| e.to_string())?;
    store.delete(TOKEN_KEY);
    store.save().map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Add restore_session to the invoke_handler in lib.rs**

Open `src-tauri/src/lib.rs`. The invoke_handler currently lists the commands. Add `commands::user::restore_session` so the handler looks like:

```rust
.invoke_handler(tauri::generate_handler![
    commands::user::signup,
    commands::user::signin,
    commands::user::signout,
    commands::user::get_me,
    commands::user::restore_session,
    commands::user::update_profile,
    commands::user::get_contacts,
    commands::user::add_contact,
    commands::chat::create_room,
    commands::chat::get_rooms,
    commands::chat::send_message,
    commands::chat::get_messages,
    commands::chat::delete_message,
    commands::chat::subscribe_room,
    commands::chat::unsubscribe_room,
])
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1
```

Expected: no errors. Common failure:
- `"method not found in surrealdb::Surreal<Client>"` for `authenticate` — check that `surrealdb::opt::auth::Jwt` is the right type for your surrealdb 3.x version. If `Jwt::from(string)` fails, try `token_str.as_str().parse().unwrap_or_default()` or consult `cargo doc --open` for the `authenticate` signature.
- `"cannot find value TOKEN_KEY"` — make sure the constant is defined at module level, not inside a function.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/user.rs src-tauri/src/lib.rs
git commit -m "feat: persist session token across app restarts"
```

---

## Task 3: Update frontend init() to use restore_session

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Replace the init() function**

In `src/routes/+page.svelte`, find the `init()` function (lines 37–46):

```typescript
  async function init() {
    try {
      user = await cmd<User>('get_me');
      view = 'app';
      await loadRooms();
      contacts = await cmd<User[]>('get_contacts').catch(() => []);
    } catch {
      view = 'auth';
    }
  }
```

Replace with:

```typescript
  async function init() {
    try {
      user = await cmd<User>('restore_session');
      view = 'app';
      await loadRooms();
      contacts = await cmd<User[]>('get_contacts').catch(() => []);
    } catch {
      view = 'auth';
    }
  }
```

Only the `cmd` call changes — `'get_me'` → `'restore_session'`. Everything else stays the same.

- [ ] **Step 2: Verify TypeScript types**

```bash
pnpm check 2>&1
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: restore session on app launch instead of failing to login screen"
```

---

## Task 4: Wire delete_message into the UI

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/ChatMain.svelte`

- [ ] **Step 1: Add deleteMessage handler in +page.svelte**

In `src/routes/+page.svelte`, find the `sendMessage()` function (around line 111). Add the new function directly after it, before `onMount`:

```typescript
  async function deleteMessage(msgId: string) {
    err = '';
    try {
      await cmd('delete_message', { messageId: msgId });
      messages = messages.filter(m => full(m.id) !== msgId);
    } catch (e) { err = String(e); }
  }
```

- [ ] **Step 2: Pass onDeleteMessage and user to ChatMain**

In the same file, find the `<ChatMain ... />` block in the template (around line 156):

```svelte
    <ChatMain
      {activeRoom}
      {messages}
      {err}
      bind:fMsg
      onSendMessage={sendMessage}
      onShowMenu={showMenu}
    />
```

Replace with:

```svelte
    <ChatMain
      {activeRoom}
      {messages}
      {user}
      {err}
      bind:fMsg
      onSendMessage={sendMessage}
      onDeleteMessage={deleteMessage}
      onShowMenu={showMenu}
    />
```

- [ ] **Step 3: Update ChatMain.svelte to accept user and onDeleteMessage props**

Open `src/lib/components/ChatMain.svelte`. Find the `interface Props` block (lines 6–13):

```typescript
  interface Props {
    activeRoom: Room | null;
    messages: Message[];
    err: string;
    fMsg: string;
    onSendMessage: () => void;
    onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
  }
```

Replace with:

```typescript
  interface Props {
    activeRoom: Room | null;
    messages: Message[];
    user: User | null;
    err: string;
    fMsg: string;
    onSendMessage: () => void;
    onDeleteMessage: (msgId: string) => void;
    onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
  }
```

- [ ] **Step 4: Update destructuring in ChatMain.svelte**

Find the destructuring block (lines 16–23):

```typescript
  let {
    activeRoom,
    messages,
    err,
    fMsg      = $bindable(),
    onSendMessage,
    onShowMenu,
  }: Props = $props();
```

Replace with:

```typescript
  let {
    activeRoom,
    messages,
    user,
    err,
    fMsg      = $bindable(),
    onSendMessage,
    onDeleteMessage,
    onShowMenu,
  }: Props = $props();
```

- [ ] **Step 5: Add User import to ChatMain.svelte**

Find the import line at the top of the script block:

```typescript
  import type { Room, Message, ContextMenuItem } from '$lib/types';
```

Replace with:

```typescript
  import type { User, Room, Message, ContextMenuItem } from '$lib/types';
```

- [ ] **Step 6: Add delete option to message context menu**

Find the `oncontextmenu` handler on the message `<div>` (around line 82 of ChatMain.svelte):

```svelte
          oncontextmenu={(e) => onShowMenu(e, [{ label: 'Copy message', action: () => navigator.clipboard.writeText(msg.body) }])}
```

Replace with:

```svelte
          oncontextmenu={(e) => {
            const items: ContextMenuItem[] = [
              { label: 'Copy message', action: () => navigator.clipboard.writeText(msg.body) },
            ];
            if (user && full(msg.author) === full(user.id)) {
              items.push({ label: 'Delete message', action: () => onDeleteMessage(full(msg.id)) });
            }
            onShowMenu(e, items);
          }}
```

- [ ] **Step 7: Verify TypeScript types**

```bash
pnpm check 2>&1
```

Expected: no errors.

- [ ] **Step 8: Commit**

```bash
git add src/routes/+page.svelte src/lib/components/ChatMain.svelte
git commit -m "feat: add delete message to context menu"
```

---

## Task 5: Wire update_profile into the sidebar

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Add updateProfile handler in +page.svelte**

In `src/routes/+page.svelte`, add after `deleteMessage()`:

```typescript
  async function updateProfile(fields: { username?: string; avatar?: string }) {
    user = await cmd<User>('update_profile', fields);
  }
```

- [ ] **Step 2: Pass onUpdateProfile to Sidebar**

Find the `<Sidebar ... />` block in the template:

```svelte
    <Sidebar
      {user}
      {rooms}
      {contacts}
      {activeRoom}
      bind:showNewRoom
      bind:fRoom
      onSelectRoom={selectRoom}
      onCreateRoom={createRoom}
      onSignout={signout}
      onShowMenu={showMenu}
    />
```

Replace with:

```svelte
    <Sidebar
      {user}
      {rooms}
      {contacts}
      {activeRoom}
      bind:showNewRoom
      bind:fRoom
      onSelectRoom={selectRoom}
      onCreateRoom={createRoom}
      onSignout={signout}
      onShowMenu={showMenu}
      onUpdateProfile={updateProfile}
    />
```

- [ ] **Step 3: Add onUpdateProfile prop and profile edit form to Sidebar.svelte**

Open `src/lib/components/Sidebar.svelte`. Make the following changes:

**3a — Add `onUpdateProfile` to the `interface Props` block:**

Find:
```typescript
  interface Props {
    user: User | null;
    rooms: Room[];
    contacts: User[];
    activeRoom: Room | null;
    showNewRoom: boolean;
    fRoom: string;
    onSelectRoom: (room: Room) => void;
    onCreateRoom: () => void;
    onSignout: () => void;
    onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
  }
```

Replace with:
```typescript
  interface Props {
    user: User | null;
    rooms: Room[];
    contacts: User[];
    activeRoom: Room | null;
    showNewRoom: boolean;
    fRoom: string;
    onSelectRoom: (room: Room) => void;
    onCreateRoom: () => void;
    onSignout: () => void;
    onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
    onUpdateProfile: (fields: { username?: string; avatar?: string }) => Promise<void>;
  }
```

**3b — Add `onUpdateProfile` to the destructuring block:**

Find:
```typescript
  let {
    user,
    rooms,
    contacts,
    activeRoom,
    showNewRoom = $bindable(),
    fRoom       = $bindable(),
    onSelectRoom,
    onCreateRoom,
    onSignout,
    onShowMenu,
  }: Props = $props();
```

Replace with:
```typescript
  let {
    user,
    rooms,
    contacts,
    activeRoom,
    showNewRoom = $bindable(),
    fRoom       = $bindable(),
    onSelectRoom,
    onCreateRoom,
    onSignout,
    onShowMenu,
    onUpdateProfile,
  }: Props = $props();
```

**3c — Add local state and submit handler for the profile form. Add after the destructuring block:**

```typescript
  let showEditProfile = $state(false);
  let fProfileUsername = $state('');
  let fProfileAvatar   = $state('');
  let profileErr       = $state('');

  function openEditProfile() {
    fProfileUsername = user?.username ?? '';
    fProfileAvatar   = user?.avatar   ?? '';
    profileErr = '';
    showEditProfile = true;
  }

  async function submitProfile() {
    profileErr = '';
    try {
      await onUpdateProfile({
        username: fProfileUsername.trim() || undefined,
        avatar:   fProfileAvatar.trim()   || undefined,
      });
      showEditProfile = false;
    } catch (e) { profileErr = String(e); }
  }
```

**3d — Update the user footer in the template.** Find the `<!-- User footer -->` section:

```svelte
  <!-- User footer -->
  <div class="user-footer">
    <div class="user-pill">
      <span class="avatar">{user?.username?.[0]?.toUpperCase() ?? '?'}</span>
      <span class="user-name">{user?.username ?? ''}</span>
    </div>
    <button class="icon-btn signout" title="Sign out" onclick={onSignout}>
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
        <polyline points="16 17 21 12 16 7"/>
        <line x1="21" y1="12" x2="9" y2="12"/>
      </svg>
    </button>
  </div>
```

Replace with:

```svelte
  <!-- User footer -->
  {#if showEditProfile}
    <div class="edit-profile-form">
      <input class="field-sm" placeholder="username" bind:value={fProfileUsername}
        onkeydown={(e) => e.key === 'Enter' && submitProfile()}
        onkeyup={(e) => e.key === 'Escape' && (showEditProfile = false)} />
      <input class="field-sm" placeholder="avatar url (optional)" bind:value={fProfileAvatar}
        onkeydown={(e) => e.key === 'Enter' && submitProfile()}
        onkeyup={(e) => e.key === 'Escape' && (showEditProfile = false)} />
      {#if profileErr}<p class="form-err">{profileErr}</p>{/if}
      <div class="form-row">
        <button class="btn-xs" onclick={submitProfile}>save</button>
        <button class="btn-xs btn-ghost" onclick={() => showEditProfile = false}>cancel</button>
      </div>
    </div>
  {/if}
  <div class="user-footer">
    <button class="user-pill" title="Edit profile" onclick={openEditProfile}>
      <span class="avatar">{user?.username?.[0]?.toUpperCase() ?? '?'}</span>
      <span class="user-name">{user?.username ?? ''}</span>
    </button>
    <button class="icon-btn signout" title="Sign out" onclick={onSignout}>
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
        <polyline points="16 17 21 12 16 7"/>
        <line x1="21" y1="12" x2="9" y2="12"/>
      </svg>
    </button>
  </div>
```

**3e — Add styles for the new elements.** In the `<style>` block, add before the final `</style>`:

```css
  .edit-profile-form {
    display: flex; flex-direction: column; gap: 6px;
    padding: 10px 12px;
    border-top: 1px solid var(--border-subtle);
    animation: rise 0.15s ease;
  }
  .form-row {
    display: flex; gap: 6px;
  }
  .btn-ghost {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--muted);
  }
  .btn-ghost:hover { opacity: 0.8; border-color: var(--muted); }
  .form-err {
    font-size: 10px; color: var(--danger);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .user-pill {
    display: flex; align-items: center; gap: 8px; min-width: 0;
    background: none; border: none; cursor: pointer; padding: 0;
    font-family: inherit; text-align: left;
  }
  .user-pill:hover .user-name { color: var(--text); }
```

Note: the existing `.user-pill` in the style block is a `div` style with no cursor. We are replacing the `<div class="user-pill">` with a `<button class="user-pill">` — the new CSS above supersedes any existing `.user-pill` rule. If there's a conflicting rule already in the file, remove the old one.

- [ ] **Step 4: Verify TypeScript types**

```bash
pnpm check 2>&1
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src/routes/+page.svelte src/lib/components/Sidebar.svelte
git commit -m "feat: add inline profile editor to sidebar footer"
```

---

## Task 6: Wire add_contact into the sidebar

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Add addContact handler in +page.svelte**

In `src/routes/+page.svelte`, add after `updateProfile()`:

```typescript
  async function addContact(userId: string) {
    await cmd('add_contact', { userId });
    contacts = await cmd<User[]>('get_contacts').catch(() => []);
  }
```

- [ ] **Step 2: Pass onAddContact to Sidebar**

Find the `<Sidebar ... />` block and add `onAddContact`:

```svelte
    <Sidebar
      {user}
      {rooms}
      {contacts}
      {activeRoom}
      bind:showNewRoom
      bind:fRoom
      onSelectRoom={selectRoom}
      onCreateRoom={createRoom}
      onSignout={signout}
      onShowMenu={showMenu}
      onUpdateProfile={updateProfile}
      onAddContact={addContact}
    />
```

- [ ] **Step 3: Add onAddContact prop and add-contact form to Sidebar.svelte**

**3a — Add `onAddContact` to the `interface Props` block:**

Find in Sidebar.svelte:
```typescript
    onUpdateProfile: (fields: { username?: string; avatar?: string }) => Promise<void>;
  }
```

Replace with:
```typescript
    onUpdateProfile: (fields: { username?: string; avatar?: string }) => Promise<void>;
    onAddContact: (userId: string) => Promise<void>;
  }
```

**3b — Add `onAddContact` to the destructuring block:**

Find:
```typescript
    onUpdateProfile,
  }: Props = $props();
```

Replace with:
```typescript
    onUpdateProfile,
    onAddContact,
  }: Props = $props();
```

**3c — Add local state and handler for the add-contact form. Add after the profile form state (after `submitProfile()`):**

```typescript
  let showAddContact = $state(false);
  let fContactId     = $state('');
  let contactErr     = $state('');

  async function submitContact() {
    if (!fContactId.trim()) return;
    contactErr = '';
    try {
      await onAddContact(fContactId.trim());
      fContactId = '';
      showAddContact = false;
    } catch (e) { contactErr = String(e); }
  }
```

**3d — Update the CONTACTS section header and add the form.** Find:

```svelte
  <!-- Contacts -->
  {#if contacts.length > 0}
    <div class="section-label">CONTACTS</div>
    <div class="contact-list">
```

Replace with:

```svelte
  <!-- Contacts -->
  <div class="section-label-row">
    <span class="section-label">CONTACTS</span>
    <button class="icon-btn" title="Add contact" onclick={() => { showAddContact = !showAddContact; }}>
      {showAddContact ? '×' : '+'}
    </button>
  </div>
  {#if showAddContact}
    <div class="new-room-form">
      <input class="field-sm" placeholder="user id" bind:value={fContactId}
        onkeydown={(e) => e.key === 'Enter' && submitContact()} />
      <button class="btn-xs" onclick={submitContact}>add</button>
    </div>
    {#if contactErr}<p class="form-err" style="padding: 0 12px 6px">{contactErr}</p>{/if}
  {/if}
  {#if contacts.length > 0}
    <div class="contact-list">
```

The original code has `{#if contacts.length > 0}` wrapping both the label and list. We are moving the label outside that condition so the "+" button is always visible. The list itself stays inside `{#if contacts.length > 0}`.

- [ ] **Step 4: Add section-label-row style to Sidebar.svelte**

In the `<style>` block, add:

```css
  .section-label-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 14px 5px 14px;
  }
  .section-label-row .section-label {
    padding: 0;
  }
```

Also update the original `.section-label` rule if needed — the existing rule has `padding: 14px 14px 5px`. Since `.section-label-row` handles padding for the row, the `.section-label` inside it needs `padding: 0`. The `section-label-row .section-label` override above handles this.

- [ ] **Step 5: Verify TypeScript types**

```bash
pnpm check 2>&1
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/routes/+page.svelte src/lib/components/Sidebar.svelte
git commit -m "feat: add contact by user ID from sidebar"
```

---

## Task 7: Final build verification

- [ ] **Step 1: Full Rust build**

```bash
cd src-tauri && cargo build 2>&1
```

Expected: compiles successfully. If you see linker errors on Linux, ensure `libwebkit2gtk-4.1-dev` and related Tauri system deps are installed.

- [ ] **Step 2: Full frontend type check**

```bash
pnpm check 2>&1
```

Expected: no errors.

- [ ] **Step 3: Manual functional test checklist**

Start the app with `pnpm tauri dev`, then verify:

- [ ] Sign in → close app → reopen → lands on app view (not auth screen)
- [ ] Sign in → close app → reopen → username shown in sidebar footer
- [ ] Sign out → close app → reopen → lands on auth screen
- [ ] Send a message as User A → right-click own message → "Delete message" appears → click it → message disappears
- [ ] Send a message as User A → right-click someone else's message → "Delete message" does NOT appear
- [ ] Click username pill in sidebar footer → profile edit form expands → edit username → save → sidebar shows new username
- [ ] Press Escape in profile edit form → form closes without saving
- [ ] Click "+" next to CONTACTS → input appears → type a user ID → click add → contacts list refreshes
- [ ] Invalid user ID in add-contact → error shown in form
