<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import type { User, Room, ContextMenuItem, UserSearchResult } from '$lib/types';
  import { full, sid } from '$lib/helpers';

  interface Props {
    user: User | null;
    rooms: Room[];
    contacts: User[];
    activeRoom: Room | null;
    showNewRoom: boolean;
    fRoom: string;
    fRoomKind: 'public' | 'private';
    unreadCounts: Record<string, number>;
    onSelectRoom: (room: Room) => void;
    onCreateRoom: () => void;
    onSignout: () => void;
    onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void;
    onUpdateProfile: (fields: { username?: string; avatar?: string }) => Promise<void>;
    onAddContact: (userId: string) => Promise<void>;
    onSearchUsers: (query: string) => Promise<UserSearchResult[]>;
    onStartDirectMessage: (userId: string) => Promise<void>;
    onInviteToRoom: (userId: string) => Promise<void>;
  }

  let {
    user,
    rooms,
    contacts,
    activeRoom,
    showNewRoom = $bindable(),
    fRoom       = $bindable(),
    fRoomKind   = $bindable(),
    unreadCounts,
    onSelectRoom,
    onCreateRoom,
    onSignout,
    onShowMenu,
    onUpdateProfile,
    onAddContact,
    onSearchUsers,
    onStartDirectMessage,
    onInviteToRoom,
  }: Props = $props();

  function roomLabel(room: Room): string {
    if (room.kind === 'direct') return room.other_user?.username ?? room.name ?? 'direct message';
    return room.name ?? 'untitled';
  }

  const minSidebarWidth = 228;
  const maxSidebarWidth = 440;
  let sidebarWidth = $state(282);
  let resizing = $state(false);

  function clampSidebarWidth(width: number) {
    return Math.min(maxSidebarWidth, Math.max(minSidebarWidth, width));
  }

  function setSidebarWidth(width: number) {
    sidebarWidth = clampSidebarWidth(width);
    localStorage.setItem('oxyde.sidebarWidth', String(sidebarWidth));
  }

  function onResizeMove(e: PointerEvent) {
    if (!resizing) return;
    setSidebarWidth(e.clientX);
  }

  function stopResize() {
    resizing = false;
  }

  function startResize(e: PointerEvent) {
    resizing = true;
    e.preventDefault();
  }

  function onResizeKey(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      setSidebarWidth(sidebarWidth - 16);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      setSidebarWidth(sidebarWidth + 16);
    }
  }

  onMount(() => {
    const stored = Number(localStorage.getItem('oxyde.sidebarWidth'));
    if (Number.isFinite(stored)) sidebarWidth = clampSidebarWidth(stored);
    window.addEventListener('pointermove', onResizeMove);
    window.addEventListener('pointerup', stopResize);
  });

  onDestroy(() => {
    window.removeEventListener('pointermove', onResizeMove);
    window.removeEventListener('pointerup', stopResize);
  });

  // ── Profile edit ──────────────────────────────────────────────────────────
  let showEditProfile  = $state(false);
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

  // ── Add contact ───────────────────────────────────────────────────────────
  let showAddContact = $state(false);
  let fContactQuery  = $state('');
  let searchResults  = $state<UserSearchResult[]>([]);
  let searchBusy     = $state(false);
  let contactErr     = $state('');
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  async function runUserSearch() {
    const query = fContactQuery.trim();
    searchResults = [];
    if (query.length < 2) return;
    contactErr = '';
    searchBusy = true;
    try {
      searchResults = await onSearchUsers(query);
    } catch (e) { contactErr = String(e); }
    finally { searchBusy = false; }
  }

  function scheduleUserSearch() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(runUserSearch, 220);
  }

  async function submitContact(userId: string) {
    contactErr = '';
    try {
      await onAddContact(userId);
      fContactQuery = '';
      searchResults = [];
      showAddContact = false;
    } catch (e) { contactErr = String(e); }
  }

  async function startDm(userId: string) {
    contactErr = '';
    try {
      await onStartDirectMessage(userId);
      showAddContact = false;
    } catch (e) { contactErr = String(e); }
  }

  async function invite(userId: string) {
    contactErr = '';
    try {
      await onInviteToRoom(userId);
    } catch (e) { contactErr = String(e); }
  }
</script>

<aside class="sidebar" class:resizing style="width:{sidebarWidth}px; min-width:{sidebarWidth}px">

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
    <div class="panel-form">
      <div class="panel-title">new room</div>
      <input class="field-sm" placeholder="room name" bind:value={fRoom}
        onkeydown={(e) => e.key === 'Enter' && onCreateRoom()} />
      <div class="form-row">
        <div class="segmented" aria-label="room visibility">
          <button class:active={fRoomKind === 'public'} onclick={() => fRoomKind = 'public'}>public</button>
          <button class:active={fRoomKind === 'private'} onclick={() => fRoomKind = 'private'}>private</button>
        </div>
        <button class="btn-xs" onclick={onCreateRoom}>create</button>
      </div>
    </div>
  {/if}

  <!-- Rooms -->
  <div class="section-label">ROOMS</div>
  <nav class="room-list">
    {#each rooms.filter((room) => room.kind !== 'direct') as room (full(room.id))}
      <button
        class="room-item"
        class:active={activeRoom && full(room.id) === full(activeRoom.id)}
        onclick={() => onSelectRoom(room)}
        oncontextmenu={(e) => onShowMenu(e, [{ label: 'Copy room name', action: () => navigator.clipboard.writeText(roomLabel(room)) }])}
      >
        <span class="hash">{room.kind === 'direct' ? '@' : '#'}</span>
        <span class="room-name">{roomLabel(room)}</span>
        {#if unreadCounts[sid(room.id)]}
          <span class="unread">{unreadCounts[sid(room.id)]}</span>
        {/if}
      </button>
    {:else}
      <p class="list-empty">no rooms — create one above</p>
    {/each}
  </nav>

  <!-- Direct messages -->
  <div class="section-label">DIRECT</div>
  <nav class="dm-list">
    {#each rooms.filter((room) => room.kind === 'direct') as room (full(room.id))}
      <button
        class="room-item"
        class:active={activeRoom && full(room.id) === full(activeRoom.id)}
        onclick={() => onSelectRoom(room)}
        oncontextmenu={(e) => onShowMenu(e, [{ label: 'Copy name', action: () => navigator.clipboard.writeText(roomLabel(room)) }])}
      >
        <span class="hash">@</span>
        <span class="room-name">{roomLabel(room)}</span>
        {#if unreadCounts[sid(room.id)]}
          <span class="unread">{unreadCounts[sid(room.id)]}</span>
        {/if}
      </button>
    {:else}
      <p class="list-empty">no direct messages</p>
    {/each}
  </nav>

  <!-- Contacts -->
  <div class="section-label-row">
    <span class="section-label">CONTACTS</span>
    <button class="icon-btn" title="Add contact" onclick={() => { showAddContact = !showAddContact; }}>
      {showAddContact ? '×' : '+'}
    </button>
  </div>
  {#if showAddContact}
    <div class="panel-form">
      <div class="panel-title">find people</div>
      <input class="field-sm" placeholder="search username" bind:value={fContactQuery}
        oninput={scheduleUserSearch}
        onkeydown={(e) => e.key === 'Enter' && runUserSearch()} />
      <div class="form-row">
        <span class="helper-text">2+ characters</span>
        <button class="btn-xs" onclick={runUserSearch} disabled={searchBusy}>
          {searchBusy ? '...' : 'find'}
        </button>
      </div>
    </div>
    {#if contactErr}<p class="form-err" style="padding: 0 12px 6px">{contactErr}</p>{/if}
    {#if searchResults.length > 0}
      <div class="search-results">
        {#each searchResults as result (full(result.id))}
          <div class="search-result">
            <span class="avatar mini">{result.username[0]?.toUpperCase() ?? '?'}</span>
            <span class="contact-name">{result.username}</span>
            <div class="row-actions">
              {#if activeRoom && activeRoom.kind !== 'direct'}
                <button class="mini-action" title="Invite" onclick={() => invite(sid(result.id))}>invite</button>
              {/if}
              <button class="mini-action" title="Add contact" onclick={() => submitContact(sid(result.id))}>add</button>
              <button class="mini-action primary" title="Message" onclick={() => startDm(sid(result.id))}>msg</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
  {#if contacts.length > 0}
    <div class="contact-list">
      {#each contacts as c (full(c.id))}
        <div class="contact-item">
          <span class="presence online"></span>
          <span class="contact-name">{c.username}</span>
          <button class="mini-action contact-action" onclick={() => startDm(sid(c.id))}>msg</button>
        </div>
      {/each}
    </div>
  {/if}

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

  <button
    class="resize-handle"
    aria-label="Resize sidebar"
    title="Resize sidebar"
    onpointerdown={startResize}
    onkeydown={onResizeKey}
  ></button>

</aside>

<style>
  .sidebar {
    position: relative;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--border);
    display: flex; flex-direction: column;
    overflow: hidden;
  }
  .sidebar.resizing,
  .sidebar.resizing * { cursor: col-resize; user-select: none; }
  .resize-handle {
    position: absolute; top: 0; right: -3px; bottom: 0;
    width: 6px; cursor: col-resize; z-index: 4;
    padding: 0; border: 0; background: transparent;
  }
  .resize-handle::after {
    content: ''; position: absolute; top: 0; right: 2px; bottom: 0;
    width: 1px; background: transparent;
    transition: background 0.12s;
  }
  .resize-handle:hover::after,
  .resize-handle:focus-visible::after,
  .sidebar.resizing .resize-handle::after { background: var(--accent); }
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

  .panel-form {
    display: flex; flex-direction: column; gap: 7px;
    padding: 10px 12px 11px;
    border-bottom: 1px solid var(--border-subtle);
    animation: rise 0.15s ease;
  }
  .panel-title {
    font-size: 9px; letter-spacing: 0.14em;
    color: var(--muted); text-transform: uppercase;
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
  .btn-xs:disabled { opacity: 0.45; cursor: wait; }
  .form-row {
    display: flex; align-items: center; justify-content: space-between;
    gap: 7px; min-width: 0;
  }
  .helper-text {
    color: var(--muted); font-size: 10px;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .segmented {
    display: flex; min-width: 0;
    border: 1px solid var(--border); border-radius: var(--r);
    overflow: hidden; background: var(--bg);
  }
  .segmented button {
    padding: 5px 8px; background: transparent; border: none;
    border-right: 1px solid var(--border);
    color: var(--muted); font-family: inherit; font-size: 10px;
    cursor: pointer;
  }
  .segmented button:last-child { border-right: 0; }
  .segmented button.active {
    background: var(--accent-soft); color: var(--accent);
  }

  .section-label {
    padding: 14px 14px 5px;
    font-size: 9px; letter-spacing: 0.14em;
    color: var(--muted); font-weight: 500;
  }

  .room-list { flex: 1; min-height: 70px; overflow-y: auto; padding: 3px 8px; }
  .dm-list { max-height: 28%; overflow-y: auto; padding: 3px 8px; flex-shrink: 0; }
  .room-list::-webkit-scrollbar,
  .dm-list::-webkit-scrollbar { width: 0; }

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
  .unread {
    min-width: 18px; height: 18px; margin-left: auto;
    display: inline-flex; align-items: center; justify-content: center;
    border-radius: var(--r); background: var(--accent); color: #fff;
    font-size: 10px; padding: 0 5px;
  }
  .list-empty { padding: 8px 7px; color: var(--muted); font-size: 10.5px; }

  .contact-list { padding: 3px 8px; max-height: 24%; overflow-y: auto; flex-shrink: 0; }
  .contact-item {
    display: flex; align-items: center; gap: 7px;
    padding: 5px 7px; color: var(--muted); font-size: 12px;
    border-left: 2px solid transparent;
  }
  .contact-item:hover { background: var(--surface); color: var(--text-2); }
  .contact-action { margin-left: auto; }
  .search-results {
    padding: 4px 8px 8px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .search-result {
    display: grid; grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center; gap: 7px;
    padding: 5px 4px; color: var(--text-2); font-size: 11px;
  }
  .row-actions { display: flex; gap: 4px; justify-content: flex-end; }
  .mini-action {
    padding: 3px 6px; background: transparent;
    border: 1px solid var(--border); border-radius: var(--r);
    color: var(--muted); font-family: inherit; font-size: 10px;
    cursor: pointer; white-space: nowrap;
  }
  .mini-action:hover { border-color: var(--accent); color: var(--accent); }
  .mini-action.primary {
    background: var(--accent-soft); border-color: rgba(181, 98, 26, 0.28);
    color: var(--accent);
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
  .user-pill {
    display: flex; align-items: center; gap: 8px; min-width: 0;
    background: none; border: none; cursor: pointer; padding: 0;
    font-family: inherit; text-align: left;
  }
  .user-pill:hover .user-name { color: var(--text); }

  .section-label-row {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 14px 5px 14px;
  }
  .section-label-row .section-label { padding: 0; }

  .edit-profile-form {
    display: flex; flex-direction: column; gap: 6px;
    padding: 10px 12px;
    border-top: 1px solid var(--border-subtle);
    animation: rise 0.15s ease;
  }
  .btn-ghost {
    background: transparent; border: 1px solid var(--border); color: var(--muted);
  }
  .btn-ghost:hover { opacity: 0.8; border-color: var(--muted); }
  .form-err {
    font-size: 10px; color: var(--danger);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .avatar {
    width: 26px; height: 26px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent); border-radius: var(--r);
    color: #fff; font-size: 11px; font-weight: 600;
  }
  .avatar.mini { width: 20px; height: 20px; font-size: 9px; }
  .user-name {
    font-size: 12px; color: var(--text-2);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
</style>
