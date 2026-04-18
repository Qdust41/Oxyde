# Design: Persistent Login + Unused Commands UI

**Date:** 2026-04-17
**Status:** Approved

---

## Overview

Two features:

1. **Persistent login** — users sign in once per machine; the session is restored on next app launch unless they explicitly sign out.
2. **Unused commands** — wire `delete_message`, `update_profile`, and `add_contact` into the frontend; currently these backend commands have no UI.

---

## Feature 1: Persistent Login

### Architecture

Use `tauri-plugin-store` (official Tauri v2 JSON store plugin) to persist the JWT token to disk in the app data directory.

Store file: `session.json`
Store key: `"token"` (string value)

### Backend changes

**`Cargo.toml`**
- Add `tauri-plugin-store = "2"`

**`lib.rs`**
- Register `.plugin(tauri_plugin_store::Builder::default().build())` in `tauri::Builder`

**`commands/user.rs`**

- `signin`: add `app_handle: AppHandle` parameter. After setting `state.token`, open the store and write the token string under key `"token"`.
- `signup`: same — after setting `state.token`, write token to store.
- `signout`: add `app_handle: AppHandle` parameter. After `db.invalidate()`, open the store and delete the `"token"` key.
- New command `restore_session`: opens the store, reads `"token"`. If absent → return error (frontend shows auth screen). If present → call `state.db.authenticate(jwt)` with the token, then query `SELECT * FROM $auth`. If the query returns a user → update `state.token` and return `User`. If authenticate fails (expired/invalid) → delete the token from the store, clear `state.token`, return error.

**`lib.rs` invoke_handler**
- Add `commands::user::restore_session` to the handler list.

### Frontend changes

**`src/routes/+page.svelte`**

- `init()`: replace `cmd<User>('get_me')` with `cmd<User>('restore_session')`. Behaviour is identical from the frontend's perspective — success → app view, error → auth view.

No other frontend changes needed for this feature; token save/clear happens entirely on the Rust side inside the existing signin/signup/signout commands.

### Data flow

```
App launch → init() → restore_session
  ├─ token on disk, valid   → User returned → app view
  ├─ token on disk, expired → store cleared → error → auth view
  └─ no token               → error → auth view

signin/signup → token saved to store automatically
signout       → token removed from store automatically
```

---

## Feature 2: Unused Commands UI

### 2a. Delete Message

**Backend:** `delete_message(message_id: String)` already exists. Enforces `WHERE author = $auth` so only the author can delete.

**`src/routes/+page.svelte`**
- Add `deleteMessage(msgId: string)`: calls `cmd('delete_message', { messageId: msgId })`, then filters the deleted message from local `messages` state on success.
- Pass `onDeleteMessage: (msgId: string) => void` prop to `ChatMain`.

**`src/lib/components/ChatMain.svelte`**
- Accept `onDeleteMessage` prop.
- Message context menu: add `{ label: 'Delete message', action: () => onDeleteMessage(full(msg.id)) }` item, only when `full(msg.author) === full(user?.id)` (own messages only — avoids confusing menu items for others' messages; backend already enforces the constraint server-side).
- `ChatMain` needs `user` prop passed from parent to perform the author check.

### 2b. Update Profile

**Backend:** `update_profile(username?: String, avatar?: String)` already exists. Returns updated `User`.

**`src/lib/components/Sidebar.svelte`**
- Add `showEditProfile: boolean` local state (default `false`).
- Clicking the user pill (name/avatar area) in the footer toggles `showEditProfile`.
- When open: render an inline edit form above the footer (same visual pattern as the new-room form — fade-in animation, small field + save button).
  - Fields: `username` (pre-filled with current value), `avatar` (pre-filled, optional URL).
  - Save button calls `onUpdateProfile({ username, avatar })` callback.
  - Cancel button (or pressing Escape) closes the form without saving.
- Add `onUpdateProfile: (fields: { username?: string; avatar?: string }) => Promise<void>` to Sidebar's Props interface.

**`src/routes/+page.svelte`**
- Add `updateProfile(fields)` function: calls `cmd<User>('update_profile', fields)` → updates `user` state with returned value.
- Pass `onUpdateProfile={updateProfile}` to `Sidebar`.

### 2c. Add Contact

**Backend:** `add_contact(user_id: String)` already exists. Returns a `Contact` record.

**`src/lib/components/Sidebar.svelte`**
- Add `showAddContact: boolean` local state (default `false`).
- Add a "+" `icon-btn` next to the CONTACTS section label (always visible even when contacts list is empty) to toggle `showAddContact`.
- When open: render inline form (same pattern as new-room form) with a single text input labelled "user id".
- On submit: call `onAddContact(userId)` callback, then close form.
- Add `onAddContact: (userId: string) => Promise<void>` to Sidebar's Props interface.

**`src/routes/+page.svelte`**
- Add `addContact(userId: string)` function: calls `cmd('add_contact', { userId })` → on success calls `get_contacts` to refresh the contacts list and update `contacts` state.
- Pass `onAddContact={addContact}` to `Sidebar`.

---

## Error handling

- `restore_session`: any error (missing token, expired, network) → auth view. No toast/message needed — user just sees the login screen.
- `deleteMessage`: errors shown in existing `err` state variable (already displayed in channel header).
- `updateProfile`: errors surfaced inside the edit form (local error state in Sidebar).
- `addContact`: errors surfaced inside the add-contact form (local error state in Sidebar).

---

## Out of scope

- User search (no backend command exists; add_contact uses raw user IDs for now)
- Token refresh / expiry detection during an active session
- Avatar image upload (update_profile accepts URL strings only)
