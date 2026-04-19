# Modern Chat App Todo

**Date:** 2026-04-18
**Status:** Draft

## Overview

Oxyde currently has a compact chat foundation: authentication, persistent session restore, public rooms, live message updates, contacts, profile editing, message delete, and context menus. This backlog lists user-facing improvements that would make it feel closer to a modern desktop chat app.

## Core Chat

- [ ] Add message editing, with an edited timestamp or marker.
- [ ] Add replies or lightweight threads so users can respond to a specific message.
- [ ] Add reactions, starting with emoji reactions on messages.
- [ ] Add read receipts or "seen by" state for direct and group conversations.
- [ ] Add typing indicators per room.
- [ ] Add message pagination or infinite scroll instead of loading every message in a room.
- [ ] Add message search across the current room and all rooms.
- [ ] Add link previews for URLs in messages.
- [ ] Add file and image attachments with preview support.
- [ ] Add Markdown-style formatting for code, links, bold text, lists, and multiline blocks.

## Rooms And Conversations

- [ ] Add private direct messages between contacts.
- [ ] Add room membership instead of fully public rooms.
- [ ] Add invite flows for rooms and contacts instead of requiring raw user IDs.
- [ ] Add room settings: rename room, delete room, leave room.
- [ ] Add pinned messages per room.
- [ ] Add room unread counts and last-message previews in the sidebar.
- [ ] Add notification badges when messages arrive outside the active room.
- [ ] Add muted rooms or per-room notification settings.

## Contacts And Identity

- [ ] Replace "add contact by user ID" with user search by username or email.
- [ ] Add contact requests and approval instead of immediately adding contacts.
- [ ] Show real avatars instead of only username initials.
- [ ] Add presence states: online, idle, offline, and do-not-disturb.
- [ ] Add profile cards when clicking or right-clicking a user.
- [ ] Add account settings for email and password changes.
- [ ] Add password reset or recovery flow.

## Reliability And UX

- [ ] Add optimistic sending states: sending, sent, failed, retry.
- [ ] Add offline handling and reconnect indicators.
- [ ] Add local draft persistence per room.
- [ ] Preserve scroll position when switching rooms.
- [ ] Avoid always auto-scrolling if the user is reading older messages.
- [ ] Add empty, error, and loading states for room list, contacts, and messages.
- [ ] Add toast notifications for copy, delete, save, and failed actions.
- [ ] Add keyboard shortcuts: room switcher, search, focus composer, escape modals.
- [ ] Add accessibility pass: focus states, ARIA labels, keyboard context menus.

## Security And Privacy

- [ ] Clarify whether "encrypted" is real; the auth screen says encrypted, but messages currently appear stored as plain text.
- [ ] Add end-to-end encryption or remove encryption claims until implemented.
- [ ] Store session tokens more securely where possible, ideally via OS keychain or credential storage instead of plain app store JSON.
- [ ] Add rate limits or abuse protection for room and message creation.
- [ ] Add validation and length limits for usernames, room names, avatars, and message bodies.
- [ ] Add block and report user flows.

## Desktop App Polish

- [ ] Add native notifications for background messages.
- [ ] Add tray behavior or "minimize to tray" settings.
- [ ] Add app update flow.
- [ ] Add deep links or app links for room invites.
- [ ] Add platform-specific menu items: preferences, quit, about.
- [ ] Add window state persistence: size, position, last active room.
- [ ] Add themes, including light, dark, and system options.
- [ ] Add responsive layout for narrower windows.

## Data Model And Backend

- [ ] Add room membership tables and permissions. Rooms and messages are currently broadly selectable in `surreal/schema.surql`.
- [ ] Add message metadata fields like `updated`, `deleted`, `reply_to`, `attachments`, and `reactions`.
- [ ] Add indexes for common queries: messages by room and created timestamp, contacts by owner, room memberships.
- [ ] Add proper soft delete for messages instead of hard delete.
- [ ] Add migrations or versioning for schema changes.
- [ ] Add tests around auth permissions, contact visibility, message ownership, and live subscriptions.

## Near-Term Best Bets

- [ ] Direct messages.
- [ ] Unread counts and notifications.
- [ ] Message editing, reactions, and replies.
- [ ] User search instead of raw user IDs.
- [ ] Pagination or infinite scroll.
- [ ] Room membership and private rooms.
- [ ] Secure token storage and validation limits.
