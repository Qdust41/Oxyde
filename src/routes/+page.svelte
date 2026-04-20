<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import LoadingScreen from "$lib/components/LoadingScreen.svelte";
    import AuthCard from "$lib/components/AuthCard.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import ChatMain from "$lib/components/ChatMain.svelte";
    import ContextMenu from "$lib/components/ContextMenu.svelte";
    import type {
        User,
        Room,
        Message,
        LiveEvent,
        ContextMenuItem,
        UserSearchResult,
    } from "$lib/types";
    import { sid, full, cmd } from "$lib/helpers";

    // ─── State ────────────────────────────────────────────────────────────────
    let user = $state<User | null>(null);
    let rooms = $state<Room[]>([]);
    let activeRoom = $state<Room | null>(null);
    let messages = $state<Message[]>([]);
    let contacts = $state<User[]>([]);
    let subId = $state<string | null>(null);
    let unlisten = $state<(() => void) | null>(null);
    let hasOlderMessages = $state(false);
    let isLoadingOlder = $state(false);
    let unreadCounts = $state<Record<string, number>>({});
    let roomSelectionToken = 0;

    let view = $state<"loading" | "auth" | "app">("loading");
    let authMode = $state<"signin" | "signup">("signin");
    let showNewRoom = $state(false);
    let err = $state("");

    let fEmail = $state("");
    let fPass = $state("");
    let fUser = $state("");
    let fMsg = $state("");
    let fRoom = $state("");
    let fRoomKind = $state<"public" | "private">("public");
    let replyTo = $state<Message | null>(null);

    let contextMenu = $state<{
        x: number;
        y: number;
        items: ContextMenuItem[];
    } | null>(null);

    function showMenu(e: MouseEvent, items: ContextMenuItem[]) {
        e.preventDefault();
        contextMenu = { x: e.clientX, y: e.clientY, items };
    }

    // ─── Auth ─────────────────────────────────────────────────────────────────
    async function init() {
        try {
            user = await cmd<User>("restore_session");
            view = "app";
            await loadRooms();
            contacts = await cmd<User[]>("get_contacts").catch(() => []);
            requestNotificationPermission();
        } catch {
            view = "auth";
        }
    }

    async function signin() {
        err = "";
        try {
            await cmd("signin", { email: fEmail, password: fPass });
            user = await cmd<User>("get_me");
            view = "app";
            await loadRooms();
            contacts = await cmd<User[]>("get_contacts").catch(() => []);
            requestNotificationPermission();
        } catch (e) {
            err = String(e);
        }
    }

    async function signup() {
        err = "";
        try {
            user = await cmd<User>("signup", {
                email: fEmail,
                username: fUser,
                password: fPass,
            });
            view = "app";
            await loadRooms();
            requestNotificationPermission();
        } catch (e) {
            err = String(e);
        }
    }

    function requestNotificationPermission() {
        if ("Notification" in window && Notification.permission === "default") {
            Notification.requestPermission().catch(() => {});
        }
    }

    async function signout() {
        roomSelectionToken += 1;
        await cmd("signout").catch(() => {});
        if (subId) {
            await cmd("unsubscribe_room", { subId }).catch(() => {});
            subId = null;
        }
        if (unlisten) {
            unlisten();
            unlisten = null;
        }
        user = null;
        rooms = [];
        messages = [];
        activeRoom = null;
        unreadCounts = {};
        view = "auth";
    }

    // ─── Rooms ────────────────────────────────────────────────────────────────
    function isCurrentRoomSelection(token: number, roomId: string) {
        return (
            token === roomSelectionToken &&
            activeRoom !== null &&
            sid(activeRoom.id) === roomId
        );
    }

    function onlyRoomMessages(roomId: string, source: Message[]) {
        return source.filter((message) => sid(message.room) === roomId);
    }

    async function loadRooms() {
        rooms = await cmd<Room[]>("get_rooms");
        if (rooms.length && !activeRoom) await selectRoom(rooms[0]);
    }

    async function selectRoom(room: Room) {
        const token = ++roomSelectionToken;
        const roomId = sid(room.id);
        const previousSubId = subId;
        const previousUnlisten = unlisten;

        subId = null;
        unlisten = null;

        activeRoom = room;
        messages = [];
        hasOlderMessages = false;
        isLoadingOlder = false;
        replyTo = null;

        if (previousSubId) {
            await cmd("unsubscribe_room", { subId: previousSubId }).catch(
                () => {},
            );
        }
        if (previousUnlisten) {
            previousUnlisten();
        }
        if (token !== roomSelectionToken) return;

        const cached = await cmd<Message[]>("get_cached_messages", { roomId });
        if (!isCurrentRoomSelection(token, roomId)) return;
        if (cached.length > 0) {
            messages = onlyRoomMessages(roomId, cached);
            hasOlderMessages = false;
        }

        const fresh = await cmd<Message[]>("get_messages", {
            roomId,
            limit: 50,
        });
        if (!isCurrentRoomSelection(token, roomId)) return;
        messages = onlyRoomMessages(roomId, fresh);
        hasOlderMessages = fresh.length === 50;
        unreadCounts = { ...unreadCounts, [roomId]: 0 };
        await cmd("mark_room_read", { roomId }).catch(() => {});
        if (!isCurrentRoomSelection(token, roomId)) return;

        const nextSubId = await cmd<string>("subscribe_room", { roomId });
        if (!isCurrentRoomSelection(token, roomId)) {
            await cmd("unsubscribe_room", { subId: nextSubId }).catch(() => {});
            return;
        }
        subId = nextSubId;
        const { listen } = await import("@tauri-apps/api/event");
        const nextUnlisten = await listen<LiveEvent>(
            "chat:message",
            ({ payload }) => {
                const { action, data } = payload;
                const eventRoomId = sid(data.room);
                const currentRoomId = activeRoom ? sid(activeRoom.id) : "";
                if (eventRoomId !== currentRoomId) {
                    unreadCounts = {
                        ...unreadCounts,
                        [eventRoomId]: (unreadCounts[eventRoomId] ?? 0) + 1,
                    };
                    if (
                        "Notification" in window &&
                        Notification.permission === "granted" &&
                        document.hidden
                    ) {
                        new Notification(
                            data.author_username ?? "New message",
                            {
                                body: data.body || "New message",
                            },
                        );
                    }
                    return;
                }
                if (action === "Create") {
                    messages = [...messages, data];
                } else if (action === "Delete") {
                    messages = messages.filter(
                        (m) => full(m.id) !== full(data.id),
                    );
                } else if (action === "Update") {
                    messages = messages.map((m) =>
                        full(m.id) === full(data.id) ? data : m,
                    );
                }
                cmd("mark_room_read", { roomId: currentRoomId }).catch(
                    () => {},
                );
            },
        );
        if (!isCurrentRoomSelection(token, roomId)) {
            nextUnlisten();
            if (subId === nextSubId) {
                await cmd("unsubscribe_room", { subId: nextSubId }).catch(
                    () => {},
                );
                subId = null;
            }
            return;
        }
        unlisten = nextUnlisten;
    }

    async function loadOlderMessages() {
        if (
            !activeRoom ||
            isLoadingOlder ||
            !hasOlderMessages ||
            messages.length === 0
        )
            return;
        const roomId = sid(activeRoom.id);
        const token = roomSelectionToken;
        isLoadingOlder = true;
        try {
            const older = await cmd<Message[]>("get_messages", {
                roomId,
                before: messages[0].created,
                limit: 50,
            });
            if (!isCurrentRoomSelection(token, roomId)) return;
            messages = [...onlyRoomMessages(roomId, older), ...messages];
            hasOlderMessages = older.length === 50;
        } catch (e) {
            err = String(e);
        } finally {
            if (isCurrentRoomSelection(token, roomId)) {
                isLoadingOlder = false;
            }
        }
    }

    async function createRoom() {
        if (!fRoom.trim()) return;
        err = "";
        try {
            const r = await cmd<Room>("create_room", {
                name: fRoom.trim(),
                kind: fRoomKind,
            });
            rooms = [r, ...rooms];
            fRoom = "";
            showNewRoom = false;
            await selectRoom(r);
        } catch (e) {
            err = String(e);
        }
    }

    // ─── Messages ─────────────────────────────────────────────────────────────
    async function sendMessage() {
        if (!fMsg.trim() || !activeRoom) return;
        err = "";
        try {
            await cmd("send_message", {
                roomId: sid(activeRoom.id),
                body: fMsg.trim(),
                replyTo: replyTo ? sid(replyTo.id) : null,
            });
            fMsg = "";
            replyTo = null;
        } catch (e) {
            err = String(e);
        }
    }

    async function deleteMessage(msgId: string) {
        err = "";
        try {
            await cmd("delete_message", { messageId: msgId });
            messages = messages.filter((m) => full(m.id) !== msgId);
        } catch (e) {
            err = String(e);
        }
    }

    async function editMessage(msgId: string, body: string) {
        err = "";
        try {
            const updated = await cmd<Message>("edit_message", {
                messageId: msgId,
                body,
            });
            messages = messages.map((m) =>
                full(m.id) === full(updated.id) ? updated : m,
            );
        } catch (e) {
            err = String(e);
        }
    }

    async function toggleReaction(msgId: string, emoji: string) {
        err = "";
        try {
            await cmd("toggle_reaction", { messageId: msgId, emoji });
            if (activeRoom) {
                const roomId = sid(activeRoom.id);
                const token = roomSelectionToken;
                const refreshed = await cmd<Message[]>("get_messages", {
                    roomId,
                    limit: Math.max(50, Math.min(messages.length, 100)),
                });
                if (isCurrentRoomSelection(token, roomId)) {
                    messages = onlyRoomMessages(roomId, refreshed);
                }
            }
        } catch (e) {
            err = String(e);
        }
    }

    async function updateProfile(fields: {
        username?: string;
        avatar?: string;
    }) {
        user = await cmd<User>("update_profile", fields);
    }

    async function addContact(userId: string) {
        await cmd("add_contact", { userId });
        contacts = await cmd<User[]>("get_contacts").catch(() => []);
    }

    async function searchUsers(query: string) {
        return await cmd<UserSearchResult[]>("search_users", { query });
    }

    async function startDirectMessage(userId: string) {
        err = "";
        try {
            const room = await cmd<Room>("get_or_create_direct_room", {
                userId,
            });
            if (!rooms.some((r) => full(r.id) === full(room.id)))
                rooms = [room, ...rooms];
            await selectRoom(room);
        } catch (e) {
            err = String(e);
        }
    }

    async function inviteToActiveRoom(userId: string) {
        if (!activeRoom || activeRoom.kind === "direct") return;
        err = "";
        try {
            await cmd("invite_to_room", { roomId: sid(activeRoom.id), userId });
        } catch (e) {
            err = String(e);
        }
    }

    onMount(init);
    onDestroy(async () => {
        if (subId) await cmd("unsubscribe_room", { subId }).catch(() => {});
        if (unlisten) unlisten();
    });
</script>

{#if view === "loading"}
    <LoadingScreen />
{:else if view === "auth"}
    <AuthCard
        {authMode}
        {err}
        bind:fEmail
        bind:fPass
        bind:fUser
        onSignin={signin}
        onSignup={signup}
        onToggleMode={() => {
            authMode = authMode === "signin" ? "signup" : "signin";
            err = "";
        }}
    />
{:else}
    <div class="app">
        <Sidebar
            {user}
            {rooms}
            {contacts}
            {activeRoom}
            bind:showNewRoom
            bind:fRoom
            bind:fRoomKind
            {unreadCounts}
            onSelectRoom={selectRoom}
            onCreateRoom={createRoom}
            onSignout={signout}
            onShowMenu={showMenu}
            onUpdateProfile={updateProfile}
            onAddContact={addContact}
            onSearchUsers={searchUsers}
            onStartDirectMessage={startDirectMessage}
            onInviteToRoom={inviteToActiveRoom}
        />
        <ChatMain
            {activeRoom}
            {messages}
            {user}
            {err}
            {hasOlderMessages}
            {isLoadingOlder}
            bind:fMsg
            bind:replyTo
            onLoadOlderMessages={loadOlderMessages}
            onSendMessage={sendMessage}
            onDeleteMessage={deleteMessage}
            onEditMessage={editMessage}
            onToggleReaction={toggleReaction}
            onShowMenu={showMenu}
        />
    </div>
    {#if contextMenu}
        <ContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            items={contextMenu.items}
            onclose={() => (contextMenu = null)}
        />
    {/if}
{/if}

<style>
    /* ─── Reset & base ──────────────────────────────────────────────────────── */
    :global(*, *::before, *::after) {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }
    :global(html, body) {
        width: 100%;
        height: 100%;
        overflow: hidden;
        background: #09090b;
        font-family: "Martian Mono", "Courier New", monospace;
        font-size: 13px;
        color: #ddd8d0;
        -webkit-font-smoothing: antialiased;
    }

    /* ─── Design tokens ─────────────────────────────────────────────────────── */
    :global(:root) {
        --bg: #09090b;
        --sidebar-bg: #0d0d10;
        --surface: #111115;
        --surface-2: #161619;
        --border: #1c1c22;
        --border-subtle: #161619;
        --accent: #b5621a;
        --accent-glow: rgba(181, 98, 26, 0.14);
        --accent-soft: rgba(181, 98, 26, 0.08);
        --text: #ddd8d0;
        --text-2: #9994a0;
        --muted: #46464f;
        --online: #3cb870;
        --danger: #b83030;
        --r: 2px;
    }

    .app {
        display: flex;
        height: 100vh;
        width: 100%;
        animation: rise 0.2s ease;
    }
    @keyframes rise {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
</style>
