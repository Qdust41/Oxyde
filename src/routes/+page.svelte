<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import LoadingScreen from '$lib/components/LoadingScreen.svelte';
  import AuthCard      from '$lib/components/AuthCard.svelte';
  import Sidebar       from '$lib/components/Sidebar.svelte';
  import ChatMain      from '$lib/components/ChatMain.svelte';
  import ContextMenu   from '$lib/components/ContextMenu.svelte';
  import type { User, Room, Message, LiveEvent, ContextMenuItem } from '$lib/types';
  import { sid, full, cmd } from '$lib/helpers';

  // ─── State ────────────────────────────────────────────────────────────────
  let user       = $state<User | null>(null);
  let rooms      = $state<Room[]>([]);
  let activeRoom = $state<Room | null>(null);
  let messages   = $state<Message[]>([]);
  let contacts   = $state<User[]>([]);
  let subId      = $state<string | null>(null);
  let unlisten   = $state<(() => void) | null>(null);

  let view       = $state<'loading' | 'auth' | 'app'>('loading');
  let authMode   = $state<'signin' | 'signup'>('signin');
  let showNewRoom= $state(false);
  let err        = $state('');

  let fEmail = $state(''); let fPass  = $state('');
  let fUser  = $state(''); let fMsg   = $state('');
  let fRoom  = $state('');

  let contextMenu = $state<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

  function showMenu(e: MouseEvent, items: ContextMenuItem[]) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, items };
  }

  // ─── Auth ─────────────────────────────────────────────────────────────────
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

  async function signin() {
    err = '';
    try {
      await cmd('signin', { email: fEmail, password: fPass });
      user = await cmd<User>('get_me');
      view = 'app';
      await loadRooms();
      contacts = await cmd<User[]>('get_contacts').catch(() => []);
    } catch (e) { err = String(e); }
  }

  async function signup() {
    err = '';
    try {
      user = await cmd<User>('signup', { email: fEmail, username: fUser, password: fPass });
      view = 'app';
      await loadRooms();
    } catch (e) { err = String(e); }
  }

  async function signout() {
    await cmd('signout').catch(() => {});
    if (subId)   { await cmd('unsubscribe_room', { subId }).catch(() => {}); subId = null; }
    if (unlisten){ unlisten(); unlisten = null; }
    user = null; rooms = []; messages = []; activeRoom = null;
    view = 'auth';
  }

  // ─── Rooms ────────────────────────────────────────────────────────────────
  async function loadRooms() {
    rooms = await cmd<Room[]>('get_rooms');
    if (rooms.length && !activeRoom) await selectRoom(rooms[0]);
  }

  async function selectRoom(room: Room) {
    if (subId)   { await cmd('unsubscribe_room', { subId }).catch(() => {}); subId = null; }
    if (unlisten){ unlisten(); unlisten = null; }

    activeRoom = room;
    messages = await cmd<Message[]>('get_messages', { roomId: sid(room.id) });

    subId = await cmd<string>('subscribe_room', { roomId: sid(room.id) });
    const { listen } = await import('@tauri-apps/api/event');
    unlisten = await listen<LiveEvent>('chat:message', ({ payload }) => {
      const { action, data } = payload;
      if (action === 'Create')      { messages = [...messages, data]; }
      else if (action === 'Delete') { messages = messages.filter(m => full(m.id) !== full(data.id)); }
      else if (action === 'Update') { messages = messages.map(m => full(m.id) === full(data.id) ? data : m); }
    });
  }

  async function createRoom() {
    if (!fRoom.trim()) return;
    err = '';
    try {
      const r = await cmd<Room>('create_room', { name: fRoom.trim() });
      rooms = [r, ...rooms];
      fRoom = ''; showNewRoom = false;
      await selectRoom(r);
    } catch (e) { err = String(e); }
  }

  // ─── Messages ─────────────────────────────────────────────────────────────
  async function sendMessage() {
    if (!fMsg.trim() || !activeRoom) return;
    err = '';
    try {
      await cmd('send_message', { roomId: sid(activeRoom.id), body: fMsg.trim() });
      fMsg = '';
    } catch (e) { err = String(e); }
  }

  async function deleteMessage(msgId: string) {
    err = '';
    try {
      await cmd('delete_message', { messageId: msgId });
      messages = messages.filter(m => full(m.id) !== msgId);
    } catch (e) { err = String(e); }
  }

  async function updateProfile(fields: { username?: string; avatar?: string }) {
    user = await cmd<User>('update_profile', fields);
  }

  async function addContact(userId: string) {
    await cmd('add_contact', { userId });
    contacts = await cmd<User[]>('get_contacts').catch(() => []);
  }

  onMount(init);
  onDestroy(async () => {
    if (subId)   await cmd('unsubscribe_room', { subId }).catch(() => {});
    if (unlisten) unlisten();
  });
</script>

{#if view === 'loading'}
  <LoadingScreen />

{:else if view === 'auth'}
  <AuthCard
    {authMode}
    {err}
    bind:fEmail
    bind:fPass
    bind:fUser
    onSignin={signin}
    onSignup={signup}
    onToggleMode={() => { authMode = authMode === 'signin' ? 'signup' : 'signin'; err = ''; }}
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
      onSelectRoom={selectRoom}
      onCreateRoom={createRoom}
      onSignout={signout}
      onShowMenu={showMenu}
      onUpdateProfile={updateProfile}
      onAddContact={addContact}
    />
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
  </div>
  {#if contextMenu}
    <ContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      items={contextMenu.items}
      onclose={() => contextMenu = null}
    />
  {/if}
{/if}

<style>
  /* ─── Reset & base ──────────────────────────────────────────────────────── */
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(html, body) {
    width: 100%; height: 100%; overflow: hidden;
    background: #09090b;
    font-family: 'Martian Mono', 'Courier New', monospace;
    font-size: 13px;
    color: #ddd8d0;
    -webkit-font-smoothing: antialiased;
  }

  /* ─── Design tokens ─────────────────────────────────────────────────────── */
  :global(:root) {
    --bg:            #09090b;
    --sidebar-bg:    #0d0d10;
    --surface:       #111115;
    --surface-2:     #161619;
    --border:        #1c1c22;
    --border-subtle: #161619;
    --accent:        #b5621a;
    --accent-glow:   rgba(181, 98, 26, 0.14);
    --accent-soft:   rgba(181, 98, 26, 0.08);
    --text:          #ddd8d0;
    --text-2:        #9994a0;
    --muted:         #46464f;
    --online:        #3cb870;
    --danger:        #b83030;
    --r:             2px;
  }

  .app {
    display: flex; height: 100vh; width: 100%;
    animation: rise 0.2s ease;
  }
  @keyframes rise {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
  }
</style>
