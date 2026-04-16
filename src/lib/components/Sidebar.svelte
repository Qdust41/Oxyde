<script lang="ts">
  import type { User, Room, ContextMenuItem } from '$lib/types';
  import { full } from '$lib/helpers';

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
</script>

<aside class="sidebar">

  <!-- Header -->
  <div class="sidebar-head">
    <span class="sidebar-brand">OXYDE</span>
    <button class="icon-btn" title="New room"
      onclick={() => { showNewRoom = !showNewRoom; }}>
      {showNewRoom ? '×' : '+'}
    </button>
  </div>

  <!-- New room form -->
  {#if showNewRoom}
    <div class="new-room-form">
      <input class="field-sm" placeholder="room name" bind:value={fRoom}
        onkeydown={(e) => e.key === 'Enter' && onCreateRoom()} />
      <button class="btn-xs" onclick={onCreateRoom}>create</button>
    </div>
  {/if}

  <!-- Rooms -->
  <div class="section-label">ROOMS</div>
  <nav class="room-list">
    {#each rooms as room (full(room.id))}
      <button
        class="room-item"
        class:active={activeRoom && full(room.id) === full(activeRoom.id)}
        onclick={() => onSelectRoom(room)}
        oncontextmenu={(e) => onShowMenu(e, [{ label: 'Copy room name', action: () => navigator.clipboard.writeText(room.name) }])}
      >
        <span class="hash">#</span>
        <span class="room-name">{room.name}</span>
      </button>
    {:else}
      <p class="list-empty">no rooms — create one above</p>
    {/each}
  </nav>

  <!-- Contacts -->
  {#if contacts.length > 0}
    <div class="section-label">CONTACTS</div>
    <div class="contact-list">
      {#each contacts as c}
        <div class="contact-item">
          <span class="presence online"></span>
          <span class="contact-name">{c.username}</span>
        </div>
      {/each}
    </div>
  {/if}

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

</aside>

<style>
  .sidebar {
    width: 282px; min-width: 282px;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border);
    display: flex; flex-direction: column;
    overflow: hidden;
  }
  .sidebar-head {
    display: flex; align-items: center; justify-content: space-between;
    padding: 16px 14px 14px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .sidebar-brand {
    font-family: 'Cormorant Garamond', Georgia, serif;
    font-size: 17px; font-weight: 700;
    color: var(--accent); letter-spacing: 0.2em;
  }
  .icon-btn {
    width: 22px; height: 22px;
    display: flex; align-items: center; justify-content: center;
    background: none; border: 1px solid var(--border);
    border-radius: var(--r); color: var(--muted);
    font-size: 15px; line-height: 1;
    cursor: pointer; transition: border-color 0.12s, color 0.12s;
    font-family: inherit;
  }
  .icon-btn:hover { border-color: var(--accent); color: var(--accent); }
  .icon-btn.signout:hover { border-color: var(--danger); color: var(--danger); }

  .new-room-form {
    display: flex; gap: 6px; align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-subtle);
    animation: rise 0.15s ease;
  }
  @keyframes rise {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .field-sm {
    flex: 1; padding: 6px 10px;
    background: var(--bg); border: 1px solid var(--border);
    border-radius: var(--r); color: var(--text);
    font-family: inherit; font-size: 11px; outline: none;
    transition: border-color 0.12s;
  }
  .field-sm:focus { border-color: var(--accent); }
  .field-sm::placeholder { color: var(--muted); }
  .btn-xs {
    padding: 6px 10px; flex-shrink: 0;
    background: var(--accent); border: none;
    border-radius: var(--r); color: #fff;
    font-family: inherit; font-size: 11px; cursor: pointer;
    transition: opacity 0.12s;
  }
  .btn-xs:hover { opacity: 0.82; }

  .section-label {
    padding: 14px 14px 5px;
    font-size: 9px; letter-spacing: 0.14em;
    color: var(--muted); font-weight: 500;
  }

  .room-list { flex: 1; overflow-y: auto; padding: 3px 8px; }
  .room-list::-webkit-scrollbar { width: 0; }

  .room-item {
    display: flex; align-items: center; gap: 5px;
    width: 100%; padding: 5px 7px; margin-bottom: 1px;
    background: none; border: none;
    border-left: 2px solid transparent;
    border-radius: 0 var(--r) var(--r) 0;
    color: var(--muted); font-family: inherit; font-size: 13px;
    cursor: pointer; text-align: left; transition: all 0.1s;
  }
  .room-item:hover { background: var(--surface); color: var(--text-2); }
  .room-item.active {
    background: var(--accent-soft);
    border-left-color: var(--accent);
    color: var(--text);
  }
  .hash { color: var(--muted); font-size: 14px; flex-shrink: 0; }
  .room-item.active .hash { color: var(--accent); }
  .room-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .list-empty { padding: 8px 7px; color: var(--muted); font-size: 10.5px; }

  .contact-list { padding: 3px 8px; }
  .contact-item {
    display: flex; align-items: center; gap: 7px;
    padding: 5px 7px; color: var(--muted); font-size: 12px;
  }
  .presence {
    width: 6px; height: 6px; border-radius: 50%;
    background: var(--muted); flex-shrink: 0;
  }
  .presence.online { background: var(--online); box-shadow: 0 0 5px var(--online); }
  .contact-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .user-footer {
    display: flex; align-items: center; justify-content: space-between;
    padding: 10px 12px;
    border-top: 1px solid var(--border);
    background: var(--surface); margin-top: auto;
  }
  .user-pill { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .avatar {
    width: 26px; height: 26px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent); border-radius: var(--r);
    color: #fff; font-size: 11px; font-weight: 600;
  }
  .user-name {
    font-size: 12px; color: var(--text-2);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
</style>
