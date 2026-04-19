# Near-Term Chat Features Implementation Plan

> **For agentic workers:** Use this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Prefer small, verified slices over one large migration.

**Goal:** Implement the seven highest-impact improvements from the modern chat backlog: direct messages, unread counts and notifications, message editing/reactions/replies, user search, pagination, room membership/private rooms, and stronger token/input safety.

**Architecture:** Move Oxyde from public global rooms toward permissioned conversations. Introduce room membership and room kind first, then layer direct messages, unread state, richer message metadata, search, pagination, notifications, and security hardening on top. Frontend state should keep room summaries separate from loaded message pages so sidebar updates do not force full message reloads.

**Tech Stack:** SvelteKit 5, TypeScript, Tauri 2, Rust, SurrealDB 3, Tauri plugins.

---

## Delivery Order

1. Room membership and private rooms.
2. User search instead of raw user IDs.
3. Direct messages.
4. Message pagination and scroll behavior.
5. Unread counts and notifications.
6. Message editing, reactions, and replies.
7. Secure token storage and validation limits.

This order reduces rework: direct messages and unread counts both depend on membership; richer messages are easier after pagination and room summaries are in place.

---

## Shared Data Model Target

### Tables

| Table | Purpose |
|---|---|
| `room` | Conversation container. Add `kind`, `name`, `created_by`, `created`, `updated`. |
| `room_member` | Membership and per-user room state. Stores `room`, `user`, `role`, `joined`, `last_read_at`, `muted`. |
| `message` | Message body and metadata. Add `updated`, `deleted`, `reply_to`. |
| `message_reaction` | One reaction per user/message/emoji. |
| `contact` | Existing contact graph. Keep, then improve with search/request flows later. |

### Permission Rules

- Users can select rooms only when they are members.
- Users can select messages only for rooms where they are members.
- Users can create messages only in rooms where they are members.
- Users can update/delete only their own messages.
- Users can select room members only for rooms where they are members.
- Direct-message rooms should only include the two participants.

### Suggested Models

Update `src-tauri/src/models.rs` and `src/lib/types.ts` to include:

```ts
export interface Room {
  id: any;
  name?: string;
  kind: 'public' | 'private' | 'direct';
  created_by?: any;
  created: string;
  updated?: string;
  last_message?: Message;
  unread_count?: number;
}

export interface RoomMember {
  id: any;
  room: any;
  user: any;
  role: 'owner' | 'member';
  joined: string;
  last_read_at?: string;
  muted?: boolean;
}

export interface Message {
  id: any;
  room: any;
  author: any;
  author_username?: string;
  body: string;
  created: string;
  updated?: string;
  deleted?: boolean;
  reply_to?: any;
  reactions?: MessageReactionSummary[];
}

export interface MessageReactionSummary {
  emoji: string;
  count: number;
  reacted_by_me: boolean;
}
```

---

## File Map

| File | Change |
|---|---|
| `surreal/schema.surql` | Add membership, richer message fields, reaction table, indexes, permissions, validation. |
| `src-tauri/src/models.rs` | Add `RoomMember`, richer `Room`, richer `Message`, reaction models, room summary structs. |
| `src/lib/types.ts` | Mirror backend types for frontend state. |
| `src-tauri/src/commands/chat.rs` | Add membership-aware room/message queries, pagination, edit/reply/reaction/read commands. |
| `src-tauri/src/commands/user.rs` | Add user search and eventually credential/profile validation improvements. |
| `src-tauri/src/commands/mod.rs` | Register any new command modules if split. |
| `src-tauri/src/lib.rs` | Register new commands and notification/keychain plugins if used. |
| `src/routes/+page.svelte` | Split room summaries, active room, message page state, unread updates, notification hooks. |
| `src/lib/components/Sidebar.svelte` | User search, direct-message entry points, unread counts, private room UI. |
| `src/lib/components/ChatMain.svelte` | Pagination, edit UI, reply UI, reactions, read markers. |
| `src/lib/components/AuthCard.svelte` | Validation and copy changes if encryption wording changes. |
| `src/lib/helpers.ts` | Add date/cursor helpers and user display helpers as needed. |

---

## Task 1: Room Membership And Private Rooms

**Goal:** Replace global room visibility with explicit membership. Support public rooms as joinable conversations and private rooms as invite-only conversations.

**Files:**
- Modify: `surreal/schema.surql`
- Modify: `src-tauri/src/models.rs`
- Modify: `src/lib/types.ts`
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] Add `kind` to `room`: `public`, `private`, `direct`.
- [ ] Add `created_by` and `updated` to `room`.
- [ ] Add `room_member` table with `room`, `user`, `role`, `joined`, `last_read_at`, `muted`.
- [ ] Add unique index on `room_member(room, user)`.
- [ ] Update room permissions so `select` requires membership, except optionally public room discovery.
- [ ] Update message permissions so `select` and `create` require room membership.
- [ ] Update `create_room` to create the room and insert the creator as owner/member in one command.
- [ ] Update `get_rooms` to return only rooms where the current user is a member.
- [ ] Add `join_public_room(room_id)` if public room discovery remains available.
- [ ] Add `invite_to_room(room_id, user_id)` for owner/member invitation, depending on chosen rules.
- [ ] Add UI affordances for private/public room creation.
- [ ] Verify old public-room behavior still works for rooms the user creates.

**Acceptance Criteria:**
- A user cannot see rooms they are not a member of.
- A user cannot fetch or send messages in rooms they are not a member of.
- Creating a room makes the creator a member.
- Existing room list and message send flows still work for the creator.

---

## Task 2: User Search Instead Of Raw User IDs

**Goal:** Let users find people by username or email instead of manually copying record IDs.

**Files:**
- Modify: `surreal/schema.surql`
- Modify: `src-tauri/src/commands/user.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/lib/components/Sidebar.svelte`
- Modify: `src/routes/+page.svelte`

- [ ] Add indexes for searchable fields, at least `username`; keep `email` unique.
- [ ] Add validation and privacy rules for what search returns.
- [ ] Add `search_users(query: String) -> Vec<UserSearchResult>`.
- [ ] Exclude the current user from search results.
- [ ] Return safe user fields only: id, username, avatar, maybe email only if product rules allow it.
- [ ] Replace add-contact raw ID field with a search box and selectable results.
- [ ] Use selected search result IDs for `add_contact`, `invite_to_room`, and direct-message creation.
- [ ] Add debouncing in the frontend so search does not run on every keystroke immediately.

**Acceptance Criteria:**
- Contacts can be added without knowing a raw SurrealDB record ID.
- Empty/short searches do not spam the backend.
- Search results do not expose password/token/internal fields.

---

## Task 3: Direct Messages

**Goal:** Add one-to-one conversations that behave like rooms but are created from contacts or user search.

**Files:**
- Modify: `surreal/schema.surql`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/Sidebar.svelte`
- Modify: `src/lib/components/ChatMain.svelte`

- [ ] Add `room.kind = 'direct'`.
- [ ] Decide direct room naming: no stored name, display as other participant's username.
- [ ] Add a stable uniqueness guard for direct rooms. Use a deterministic participant key if SurrealDB indexes cannot enforce two-member uniqueness directly.
- [ ] Add `get_or_create_direct_room(user_id) -> Room`.
- [ ] Insert both participants into `room_member` when creating a direct room.
- [ ] Add command/query support to hydrate direct-room display names and avatars.
- [ ] Add "Message" action to contacts/search results.
- [ ] Show direct messages in a separate sidebar section or mixed with rooms using clear labels.

**Acceptance Criteria:**
- Starting a DM with the same user opens the existing direct room.
- Both participants can see and send messages in the DM.
- No third user can read or join the DM.

---

## Task 4: Pagination And Scroll Behavior

**Goal:** Avoid loading every message at once and preserve a stable reading experience.

**Files:**
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/ChatMain.svelte`
- Modify: `src/lib/helpers.ts`

- [ ] Change `get_messages(room_id)` into a paginated command, for example `get_messages(room_id, before?: datetime, limit?: number)`.
- [ ] Return messages newest-page aware but render oldest-to-newest in the UI.
- [ ] Add an index on `message(room, created)`.
- [ ] Track `hasOlderMessages`, `isLoadingOlder`, and `oldestCursor` in page state.
- [ ] Load older messages when the user scrolls near the top.
- [ ] Preserve scroll offset after prepending older messages.
- [ ] Only auto-scroll to bottom when the user is already near the bottom or the message is authored by the current user.
- [ ] Keep the live subscription for new messages in the active room.

**Acceptance Criteria:**
- Opening a room loads a bounded number of messages.
- Scrolling upward loads older messages without jumping.
- New incoming messages do not force-scroll users who are reading history.

---

## Task 5: Unread Counts And Notifications

**Goal:** Make missed messages visible in the sidebar and via desktop notifications when appropriate.

**Files:**
- Modify: `surreal/schema.surql`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] Add or use `room_member.last_read_at`.
- [ ] Add `mark_room_read(room_id)` command.
- [ ] Update room summary query to include `last_message` and `unread_count`.
- [ ] Mark the active room read when opened and when the user reaches the bottom.
- [ ] Increment/update unread room summaries when live events arrive for inactive rooms.
- [ ] Add visual unread badges in the sidebar.
- [ ] Add Tauri notification plugin if not already available.
- [ ] Request notification permission at a sensible moment.
- [ ] Send native notifications for messages in inactive rooms when the app is unfocused and the room is not muted.
- [ ] Add `muted` support from `room_member.muted` to suppress notifications.

**Acceptance Criteria:**
- Inactive room messages increase unread count.
- Opening or reading a room clears its unread count for the current user.
- Desktop notifications fire only when useful and respect muted rooms.

---

## Task 6: Message Editing, Reactions, And Replies

**Goal:** Add the message interactions users expect without disrupting the current simple composer.

**Files:**
- Modify: `surreal/schema.surql`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/components/ChatMain.svelte`
- Modify: `src/lib/components/ContextMenu.svelte` if richer menu state is needed.

- [ ] Add `updated`, `deleted`, and `reply_to` fields to `message`.
- [ ] Replace hard delete with soft delete for normal message deletion.
- [ ] Add `edit_message(message_id, body)` command with author-only permission.
- [ ] Add `send_message(room_id, body, reply_to?)`.
- [ ] Add `message_reaction` table with `message`, `user`, `emoji`, `created`.
- [ ] Add unique index on `message_reaction(message, user, emoji)`.
- [ ] Add `toggle_reaction(message_id, emoji)` command.
- [ ] Include reaction summaries when fetching messages.
- [ ] Add context menu actions for edit, reply, delete, copy.
- [ ] Add inline edit mode for the user's own messages.
- [ ] Add reply preview above the composer and reply reference rendering in the message list.
- [ ] Add a small reaction picker or a short default emoji row.
- [ ] Ensure live update events update edited messages, deleted messages, and reactions.

**Acceptance Criteria:**
- Users can edit only their own messages.
- Replies show enough context to identify the parent message.
- Reactions toggle reliably and aggregate counts across users.
- Deleted messages leave a useful placeholder instead of breaking replies.

---

## Task 7: Secure Token Storage And Validation Limits

**Goal:** Reduce security and data-quality risks before the app grows more social features.

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/user.rs`
- Modify: `src-tauri/src/commands/chat.rs`
- Modify: `surreal/schema.surql`
- Modify: `src/lib/components/AuthCard.svelte`
- Modify: `src/lib/components/Sidebar.svelte`
- Modify: `src/lib/components/ChatMain.svelte`

- [ ] Replace plain `tauri-plugin-store` token persistence with OS-backed secure storage where practical.
- [ ] If secure storage is not immediately available on every target platform, isolate token storage behind helper functions so the backend can swap implementations later.
- [ ] Add username length and character validation.
- [ ] Add email length validation.
- [ ] Add password minimum length in signup.
- [ ] Add room name length validation.
- [ ] Add message body length validation.
- [ ] Add avatar URL validation or remove avatar URL until uploads/proxying exist.
- [ ] Add SurrealDB schema assertions where possible, and duplicate key user-facing errors in Rust for better messages.
- [ ] Remove or revise the auth tagline claim `encrypted` unless end-to-end encryption is implemented.
- [ ] Add tests for validation boundaries in Rust command-level helpers where possible.

**Acceptance Criteria:**
- Session tokens are not stored as plain JSON when a supported secure storage path is available.
- Invalid inputs fail before they create malformed records.
- Error messages are useful to users.
- Product copy no longer overclaims encryption.

---

## Verification Plan

- [ ] Run `pnpm check` after each frontend slice.
- [ ] Run `cargo test` or `cargo check` inside `src-tauri` after each Rust slice.
- [ ] Manually test with two users: public room, private room, direct message, message send, edit, reply, reaction, unread clear, notification, and signout/session restore.
- [ ] Test permission failures by trying to fetch a room/message as a non-member.
- [ ] Test scroll pagination with enough messages to require at least three pages.
