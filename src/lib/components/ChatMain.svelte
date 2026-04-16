<script lang="ts">
  import { tick } from 'svelte';
  import type { Room, Message, ContextMenuItem } from '$lib/types';
  import { full, sid, fmt } from '$lib/helpers';

  interface Props {
    activeRoom: Room | null;
    messages: Message[];
    err: string;
    fMsg: string;
    onSendMessage: () => void;
    onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
  }

  let {
    activeRoom,
    messages,
    err,
    fMsg      = $bindable(),
    onSendMessage,
    onShowMenu,
  }: Props = $props();

  let msgEl: HTMLElement;
  let inputEl: HTMLTextAreaElement;

  function scrollBottom() {
    tick().then(() => { if (msgEl) msgEl.scrollTop = msgEl.scrollHeight; });
  }

  function autoResize() {
    if (!inputEl) return;
    inputEl.style.height = 'auto';
    inputEl.style.height = Math.min(inputEl.scrollHeight, 160) + 'px';
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); onSendMessage(); }
  }

  function isGrouped(i: number): boolean {
    if (i === 0) return false;
    return full(messages[i].author) === full(messages[i - 1].author);
  }

  // Scroll to bottom when messages change
  $effect(() => {
    messages.length; // track length
    scrollBottom();
  });

  // Reset textarea height after message is cleared
  $effect(() => {
    if (fMsg === '') autoResize();
  });
</script>

<main class="main">

  <!-- Channel header -->
  <header class="channel-header">
    <span class="ch-hash">#</span>
    <span class="ch-name">{activeRoom?.name ?? 'select a room'}</span>
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
      {#each messages as msg, i (full(msg.id))}
        <div
          class="msg"
          class:grouped={isGrouped(i)}
          oncontextmenu={(e) => onShowMenu(e, [{ label: 'Copy message', action: () => navigator.clipboard.writeText(msg.body) }])}
        >
          {#if !isGrouped(i)}
            <div class="msg-header">
              <span
                class="msg-author"
                oncontextmenu={(e) => { e.stopPropagation(); onShowMenu(e, [{ label: 'Copy username', action: () => navigator.clipboard.writeText(msg.author_username ?? sid(msg.author)) }]); }}
              >{msg.author_username ?? sid(msg.author)}</span>
              <span class="msg-time">{fmt(msg.created)}</span>
            </div>
          {/if}
          <p class="msg-body">{msg.body}</p>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Input bar -->
  <div class="input-bar">
    <textarea
      bind:this={inputEl}
      class="msg-input"
      placeholder={activeRoom ? `message #${activeRoom.name}` : 'select a room first'}
      bind:value={fMsg}
      onkeydown={onKey}
      oninput={autoResize}
      disabled={!activeRoom}
      rows="1"
    ></textarea>
    <button title="" class="send-btn" onclick={onSendMessage} disabled={!activeRoom || !fMsg.trim()}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="22" y1="2" x2="11" y2="13"/>
        <polygon points="22 2 15 22 11 13 2 9 22 2"/>
      </svg>
    </button>
  </div>

</main>

<style>
  .main {
    flex: 1; display: flex; flex-direction: column;
    overflow: hidden; background: var(--bg);
  }
  .channel-header {
    display: flex; align-items: center; gap: 9px;
    padding: 0 24px; height: 50px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .ch-hash { font-size: 17px; color: var(--muted); }
  .ch-name { font-size: 14px; font-weight: 500; color: var(--text); }
  .header-err {
    margin-left: auto; font-size: 10px; color: #d98080;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
    max-width: 280px;
  }

  .messages {
    flex: 1; overflow-y: auto;
    padding: 20px 24px 8px;
    display: flex; flex-direction: column;
  }
  .messages::-webkit-scrollbar { width: 4px; }
  .messages::-webkit-scrollbar-thumb { background: var(--surface-2); border-radius: 2px; }
  .messages::-webkit-scrollbar-track { background: transparent; }

  .empty-state {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 12px; color: var(--muted);
  }
  .empty-icon {
    font-size: 32px; opacity: 0.2;
    font-family: 'Cormorant Garamond', Georgia, serif;
  }
  .empty-state p { font-size: 11px; letter-spacing: 0.07em; }

  .msg { padding: 1px 0; }
  .msg.grouped { padding-top: 1px; }

  .msg-header {
    display: flex; align-items: baseline; gap: 9px;
    margin-top: 16px; margin-bottom: 3px;
  }
  .msg-author { font-size: 12px; font-weight: 500; color: var(--accent); }
  .msg-time   { font-size: 9.5px; color: var(--muted); }

  .msg-body {
    color: var(--text); font-size: 13px;
    line-height: 1.6; white-space: pre-wrap; word-break: break-word;
    animation: msgIn 0.14s ease;
  }
  .msg.grouped .msg-body { color: var(--text-2); }
  @keyframes msgIn {
    from { opacity: 0; transform: translateY(3px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .input-bar {
    display: flex; align-items: flex-end; gap: 8px;
    padding: 12px 24px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .msg-input {
    flex: 1; resize: none;
    padding: 9px 13px;
    background: var(--surface);
    border: 1px solid var(--border); border-radius: var(--r);
    color: var(--text); font-family: inherit; font-size: 13px;
    line-height: 1.55; outline: none;
    transition: border-color 0.12s;
    max-height: 160px; overflow-y: auto;
  }
  .msg-input:focus        { border-color: var(--accent); }
  .msg-input:disabled     { opacity: 0.35; cursor: not-allowed; }
  .msg-input::placeholder { color: var(--muted); }
  .msg-input::-webkit-scrollbar { width: 0; }

  .send-btn {
    width: 34px; height: 34px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent); border: none; border-radius: var(--r);
    color: #fff; cursor: pointer;
    transition: opacity 0.12s, transform 0.08s;
  }
  .send-btn:hover    { opacity: 0.82; }
  .send-btn:active   { transform: scale(0.93); }
  .send-btn:disabled { opacity: 0.25; cursor: not-allowed; transform: none; }
</style>
