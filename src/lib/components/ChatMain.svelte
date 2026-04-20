<script lang="ts">
    import { tick } from "svelte";
    import type { User, Room, Message, ContextMenuItem } from "$lib/types";
    import { full, sid, fmt } from "$lib/helpers";

    interface Props {
        activeRoom: Room | null;
        messages: Message[];
        user: User | null;
        err: string;
        hasOlderMessages: boolean;
        isLoadingOlder: boolean;
        fMsg: string;
        replyTo: Message | null;
        onLoadOlderMessages: () => void;
        onSendMessage: () => void;
        onDeleteMessage: (msgId: string) => void;
        onEditMessage: (msgId: string, body: string) => void;
        onToggleReaction: (msgId: string, emoji: string) => void;
        onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
    }

    let {
        activeRoom,
        messages,
        user,
        err,
        hasOlderMessages,
        isLoadingOlder,
        fMsg = $bindable(),
        replyTo = $bindable(),
        onLoadOlderMessages,
        onSendMessage,
        onDeleteMessage,
        onEditMessage,
        onToggleReaction,
        onShowMenu,
    }: Props = $props();

    let msgEl: HTMLElement;
    let inputEl: HTMLTextAreaElement;
    let editingId = $state<string | null>(null);
    let editBody = $state("");

    function scrollBottom() {
        tick().then(() => {
            if (msgEl) msgEl.scrollTop = msgEl.scrollHeight;
        });
    }

    function autoResize() {
        if (!inputEl) return;
        inputEl.style.height = "auto";
        inputEl.style.height = Math.min(inputEl.scrollHeight, 160) + "px";
    }

    function onKey(e: KeyboardEvent) {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            onSendMessage();
        }
    }

    function roomLabel(room: Room | null): string {
        if (!room) return "select a room";
        if (room.kind === "direct")
            return room.other_user?.username ?? room.name ?? "direct message";
        return room.name ?? "untitled";
    }

    function isGrouped(i: number): boolean {
        if (i === 0) return false;
        if (messages[i].deleted || messages[i - 1].deleted) return false;
        return full(messages[i].author) === full(messages[i - 1].author);
    }

    function beginEdit(msg: Message) {
        editingId = full(msg.id);
        editBody = msg.body;
    }

    function submitEdit(msg: Message) {
        if (!editBody.trim()) return;
        onEditMessage(full(msg.id), editBody.trim());
        editingId = null;
        editBody = "";
    }

    function quickReact(msg: Message) {
        onToggleReaction(full(msg.id), "+1");
    }

    // Scroll to bottom when messages change
    $effect(() => {
        messages.length; // track length
        scrollBottom();
    });

    // Reset textarea height after message is cleared
    $effect(() => {
        if (fMsg === "") autoResize();
    });
</script>

<main class="main">
    <!-- Channel header -->
    <header class="channel-header">
        <span class="ch-hash">{activeRoom?.kind === "direct" ? "@" : "#"}</span>
        <span class="ch-name">{roomLabel(activeRoom)}</span>
        {#if err}<span class="header-err">{err}</span>{/if}
    </header>

    <!-- Message list -->
    <div class="messages" bind:this={msgEl}>
        {#if !activeRoom}
            <div class="empty-state">
                <span class="empty-icon">#</span>
                <p>select a room to start chatting</p>
            </div>
        {:else if messages.length === 0}
            <div class="empty-state">
                <span class="empty-icon">#</span>
                <p>no messages yet — say hello</p>
            </div>
        {:else}
            {#if hasOlderMessages}
                <button
                    class="load-older"
                    onclick={onLoadOlderMessages}
                    disabled={isLoadingOlder}
                >
                    {isLoadingOlder ? "loading..." : "load older messages"}
                </button>
            {/if}
            {#each messages as msg, i (full(msg.id))}
                <div
                    class="msg"
                    class:grouped={isGrouped(i)}
                    role="listitem"
                    oncontextmenu={(e) => {
                        const items: ContextMenuItem[] = [
                            {
                                label: "Copy message",
                                action: () =>
                                    navigator.clipboard.writeText(msg.body),
                            },
                            { label: "Reply", action: () => (replyTo = msg) },
                            {
                                label: "React +1",
                                action: () =>
                                    onToggleReaction(full(msg.id), "+1"),
                            },
                        ];
                        if (
                            user &&
                            full(msg.author) === full(user.id) &&
                            !msg.deleted
                        ) {
                            items.push({
                                label: "Edit message",
                                action: () => beginEdit(msg),
                            });
                            items.push({
                                label: "Delete message",
                                action: () => onDeleteMessage(full(msg.id)),
                            });
                        }
                        onShowMenu(e, items);
                    }}
                >
                    {#if !isGrouped(i)}
                        <div class="msg-header">
                            <span
                                class="msg-author"
                                role="button"
                                tabindex="0"
                                oncontextmenu={(e) => {
                                    e.stopPropagation();
                                    onShowMenu(e, [
                                        {
                                            label: "Copy username",
                                            action: () =>
                                                navigator.clipboard.writeText(
                                                    msg.author_username ??
                                                        sid(msg.author),
                                                ),
                                        },
                                        {
                                            label: "Copy user ID",
                                            action: () =>
                                                navigator.clipboard.writeText(
                                                    sid(msg.author),
                                                ),
                                        },
                                    ]);
                                }}
                                >{msg.author_username ?? sid(msg.author)}</span
                            >
                            <span class="msg-time">{fmt(msg.created)}</span>
                            {#if msg.updated}<span class="msg-time">edited</span
                                >{/if}
                        </div>
                    {/if}
                    {#if msg.reply_to}
                        <div class="reply-chip">
                            {#if msg.replied_to_message}
                                replying to {msg.replied_to_message.author_username ?? 'unknown'}: {msg.replied_to_message.body.length > 80 ? msg.replied_to_message.body.slice(0, 80) + '…' : msg.replied_to_message.body}
                            {:else}
                                replying to message
                            {/if}
                        </div>
                    {/if}
                    {#if !msg.deleted}
                        <div class="msg-actions" aria-label="message actions">
                            <button
                                title="Reply"
                                onclick={() => (replyTo = msg)}>reply</button
                            >
                            <button
                                title="React"
                                onclick={() => quickReact(msg)}>+1</button
                            >
                            {#if user && full(msg.author) === full(user.id)}
                                <button
                                    title="Edit"
                                    onclick={() => beginEdit(msg)}>edit</button
                                >
                            {/if}
                        </div>
                    {/if}
                    {#if msg.deleted}
                        <p class="msg-body deleted">message deleted</p>
                    {:else if editingId === full(msg.id)}
                        <div class="edit-row">
                            <textarea
                                class="edit-input"
                                bind:value={editBody}
                                rows="2"
                            ></textarea>
                            <button
                                class="mini-btn"
                                onclick={() => submitEdit(msg)}>save</button
                            >
                            <button
                                class="mini-btn ghost"
                                onclick={() => (editingId = null)}
                                >cancel</button
                            >
                        </div>
                    {:else}
                        <p class="msg-body">{msg.body}</p>
                    {/if}
                    {#if msg.reactions?.length}
                        <div class="reactions">
                            {#each msg.reactions as reaction}
                                <button
                                    class="reaction"
                                    class:mine={reaction.reacted_by_me}
                                    onclick={() =>
                                        onToggleReaction(
                                            full(msg.id),
                                            reaction.emoji,
                                        )}
                                >
                                    {reaction.emoji}
                                    {reaction.count}
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/each}
        {/if}
    </div>

    <!-- Input bar -->
    {#if replyTo}
        <div class="reply-bar">
            <span
                >replying to {replyTo.author_username ??
                    sid(replyTo.author)}</span
            >
            <button class="mini-btn ghost" onclick={() => (replyTo = null)}
                >cancel</button
            >
        </div>
    {/if}
    <div class="input-bar">
        <textarea
            bind:this={inputEl}
            class="msg-input"
            placeholder={activeRoom
                ? `message ${activeRoom.kind === "direct" ? "@" : "#"}${roomLabel(activeRoom)}`
                : "select a room first"}
            bind:value={fMsg}
            onkeydown={onKey}
            oninput={autoResize}
            disabled={!activeRoom}
            rows="1"
        ></textarea>
        <button
            title=""
            class="send-btn"
            onclick={onSendMessage}
            disabled={!activeRoom || !fMsg.trim()}
        >
            <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
            >
                <line x1="22" y1="2" x2="11" y2="13" />
                <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
        </button>
    </div>
</main>

<style>
    .main {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        background: var(--bg);
    }
    .channel-header {
        display: flex;
        align-items: center;
        gap: 9px;
        padding: 0 24px;
        height: 50px;
        border-bottom: 1px solid var(--border);
        flex-shrink: 0;
    }
    .ch-hash {
        font-size: 17px;
        color: var(--muted);
    }
    .ch-name {
        font-size: 14px;
        font-weight: 500;
        color: var(--text);
    }
    .header-err {
        margin-left: auto;
        font-size: 10px;
        color: #d98080;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        max-width: 280px;
    }

    .messages {
        flex: 1;
        overflow-y: auto;
        padding: 20px 24px 8px;
        display: flex;
        flex-direction: column;
    }
    .messages::-webkit-scrollbar {
        width: 4px;
    }
    .messages::-webkit-scrollbar-thumb {
        background: var(--surface-2);
        border-radius: 2px;
    }
    .messages::-webkit-scrollbar-track {
        background: transparent;
    }
    .load-older {
        align-self: center;
        margin-bottom: 12px;
        padding: 6px 10px;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--r);
        color: var(--text-2);
        font-family: inherit;
        font-size: 11px;
        cursor: pointer;
    }
    .load-older:hover {
        border-color: var(--accent);
        color: var(--text);
    }
    .load-older:disabled {
        opacity: 0.5;
        cursor: wait;
    }

    .empty-state {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 12px;
        color: var(--muted);
    }
    .empty-icon {
        font-size: 32px;
        opacity: 0.2;
        font-family: "Cormorant Garamond", Georgia, serif;
    }
    .empty-state p {
        font-size: 11px;
        letter-spacing: 0.07em;
    }

    .msg {
        padding: 1px 0;
    }
    .msg:hover .msg-actions,
    .msg:focus-within .msg-actions {
        opacity: 1;
        pointer-events: auto;
    }
    .msg.grouped {
        padding-top: 1px;
    }

    .msg-header {
        display: flex;
        align-items: baseline;
        gap: 9px;
        margin-top: 16px;
        margin-bottom: 3px;
    }
    .msg-author {
        font-size: 12px;
        font-weight: 500;
        color: var(--accent);
    }
    .msg-time {
        font-size: 9.5px;
        color: var(--muted);
    }

    .msg-body {
        color: var(--text);
        font-size: 13px;
        line-height: 1.6;
        white-space: pre-wrap;
        word-break: break-word;
        animation: msgIn 0.14s ease;
    }
    .msg.grouped .msg-body {
        color: var(--text-2);
    }
    .msg-body.deleted {
        color: var(--muted);
        font-style: italic;
    }
    .msg-actions {
        float: right;
        display: flex;
        gap: 4px;
        margin-left: 8px;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.1s;
    }
    .msg-actions button {
        padding: 2px 5px;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--r);
        color: var(--muted);
        font-family: inherit;
        font-size: 9.5px;
        cursor: pointer;
    }
    .msg-actions button:hover {
        border-color: var(--accent);
        color: var(--accent);
    }
    .reply-chip {
        display: inline-flex;
        margin: 2px 0 3px;
        padding: 3px 6px;
        border-left: 2px solid var(--accent);
        background: var(--surface);
        color: var(--muted);
        font-size: 10px;
    }
    .edit-row {
        display: flex;
        align-items: flex-end;
        gap: 6px;
        margin-top: 3px;
    }
    .edit-input {
        flex: 1;
        resize: vertical;
        min-height: 44px;
        max-height: 120px;
        padding: 7px 9px;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--r);
        color: var(--text);
        font-family: inherit;
        font-size: 12px;
    }
    .mini-btn {
        padding: 6px 8px;
        background: var(--accent);
        border: none;
        border-radius: var(--r);
        color: #fff;
        font-family: inherit;
        font-size: 10px;
        cursor: pointer;
    }
    .mini-btn.ghost {
        background: transparent;
        border: 1px solid var(--border);
        color: var(--muted);
    }
    .reactions {
        display: flex;
        gap: 5px;
        margin-top: 4px;
        flex-wrap: wrap;
    }
    .reaction {
        padding: 2px 6px;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--r);
        color: var(--text-2);
        font-family: inherit;
        font-size: 10px;
        cursor: pointer;
    }
    .reaction.mine {
        border-color: var(--accent);
        color: var(--accent);
    }
    @keyframes msgIn {
        from {
            opacity: 0;
            transform: translateY(3px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    .input-bar {
        display: flex;
        align-items: flex-end;
        gap: 8px;
        padding: 12px 24px 16px;
        border-top: 1px solid var(--border);
        flex-shrink: 0;
    }
    .reply-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 8px 24px;
        border-top: 1px solid var(--border);
        color: var(--text-2);
        font-size: 11px;
        background: var(--surface);
    }
    .msg-input {
        flex: 1;
        resize: none;
        padding: 9px 13px;
        background: var(--surface);
        border: 1px solid var(--border);
        border-radius: var(--r);
        color: var(--text);
        font-family: inherit;
        font-size: 13px;
        line-height: 1.55;
        outline: none;
        transition: border-color 0.12s;
        max-height: 160px;
        overflow-y: auto;
    }
    .msg-input:focus {
        border-color: var(--accent);
    }
    .msg-input:disabled {
        opacity: 0.35;
        cursor: not-allowed;
    }
    .msg-input::placeholder {
        color: var(--muted);
    }
    .msg-input::-webkit-scrollbar {
        width: 0;
    }

    .send-btn {
        width: 34px;
        height: 34px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--accent);
        border: none;
        border-radius: var(--r);
        color: #fff;
        cursor: pointer;
        transition:
            opacity 0.12s,
            transform 0.08s;
    }
    .send-btn:hover {
        opacity: 0.82;
    }
    .send-btn:active {
        transform: scale(0.93);
    }
    .send-btn:disabled {
        opacity: 0.25;
        cursor: not-allowed;
        transform: none;
    }
</style>
