# Context Menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a custom right-click context menu to the Oxyde chat app that replaces the browser default and offers context-aware copy actions on room names, message authors, and message bodies.

**Architecture:** A single shared `ContextMenu` Svelte component receives position + items as props and is rendered once in `+page.svelte`. State (`contextMenu`) lives in the page; a `showMenu` helper is passed down to `Sidebar` and `ChatMain` as a prop. Each trigger calls `showMenu` with the right items.

**Tech Stack:** Svelte 5 runes (`$state`, `$props`), `navigator.clipboard`, CSS custom properties already defined in `+page.svelte`.

---

## File Map

| File | Change |
|---|---|
| `src/lib/types.ts` | Add `ContextMenuItem` interface |
| `src/lib/components/ContextMenu.svelte` | New component — positioning, dismiss, copy + confirmation |
| `src/routes/+page.svelte` | Add `contextMenu` state, `showMenu` helper, render `<ContextMenu>`, pass prop to children |
| `src/lib/components/Sidebar.svelte` | Add `onShowMenu` prop, wire `oncontextmenu` on `.room-item` buttons |
| `src/lib/components/ChatMain.svelte` | Add `onShowMenu` prop, wire `oncontextmenu` on `.msg` div and `.msg-author` span |

---

### Task 1: Add `ContextMenuItem` type

**Files:**
- Modify: `src/lib/types.ts`

- [ ] **Step 1: Add the interface**

Open `src/lib/types.ts`. Append one line:

```ts
export interface User    { id: any; username: string; email: string; avatar?: string; created: string; }
export interface Room    { id: any; name: string; created: string; }
export interface Message { id: any; room: any; author: any; author_username?: string; body: string; created: string; }
export interface LiveEvent { action: 'Create' | 'Update' | 'Delete'; data: Message; }
export interface ContextMenuItem { label: string; action: () => void; }
```

- [ ] **Step 2: Verify TypeScript accepts it**

Run: `cd /home/qdust41/Oxyde && npx tsc --noEmit 2>&1 | head -20`
Expected: no errors (or only pre-existing errors unrelated to this file)

- [ ] **Step 3: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat: add ContextMenuItem interface to types"
```

---

### Task 2: Create `ContextMenu.svelte` component

**Files:**
- Create: `src/lib/components/ContextMenu.svelte`

- [ ] **Step 1: Create the component**

Create `src/lib/components/ContextMenu.svelte` with the following content:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { ContextMenuItem } from '$lib/types';

  interface Props {
    x: number;
    y: number;
    items: ContextMenuItem[];
    onclose: () => void;
  }

  let { x, y, items, onclose }: Props = $props();

  let menuEl: HTMLElement;
  let copiedIndex = $state<number | null>(null);

  // Flip position if menu would overflow viewport
  onMount(() => {
    if (!menuEl) return;
    const rect = menuEl.getBoundingClientRect();
    if (rect.right > window.innerWidth)  menuEl.style.left = (x - rect.width)  + 'px';
    if (rect.bottom > window.innerHeight) menuEl.style.top = (y - rect.height) + 'px';
  });

  async function handleItem(item: ContextMenuItem, index: number) {
    await navigator.clipboard.writeText('');   // reset
    item.action();
    copiedIndex = index;
    setTimeout(onclose, 1200);
  }

  function onWindowClick() { onclose(); }
  function onWindowKey(e: KeyboardEvent) { if (e.key === 'Escape') onclose(); }
  function onWindowContext(e: MouseEvent) { e.preventDefault(); onclose(); }
</script>

<svelte:window
  onclick={onWindowClick}
  onkeydown={onWindowKey}
  oncontextmenu={onWindowContext}
/>

<ul
  class="ctx-menu"
  bind:this={menuEl}
  style="left:{x}px; top:{y}px"
  onclick={(e) => e.stopPropagation()}
  oncontextmenu={(e) => e.stopPropagation()}
  role="menu"
>
  {#each items as item, i}
    <li role="menuitem">
      <button
        class="ctx-item"
        class:copied={copiedIndex === i}
        onclick={() => handleItem(item, i)}
      >
        {copiedIndex === i ? 'Copied!' : item.label}
      </button>
    </li>
  {/each}
</ul>

<style>
  .ctx-menu {
    position: fixed;
    list-style: none;
    min-width: 160px;
    padding: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r);
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
    z-index: 9999;
    animation: rise 0.15s ease;
  }
  @keyframes rise {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .ctx-item {
    display: block;
    width: 100%;
    padding: 7px 12px;
    background: none;
    border: none;
    border-left: 2px solid transparent;
    border-radius: var(--r);
    color: var(--text-2);
    font-family: inherit;
    font-size: 11px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .ctx-item:hover {
    background: var(--surface-2);
    color: var(--text);
    border-left-color: var(--accent);
  }
  .ctx-item.copied {
    color: var(--accent);
    background: var(--accent-soft);
  }
</style>
```

- [ ] **Step 2: Verify no TypeScript errors**

Run: `cd /home/qdust41/Oxyde && npx tsc --noEmit 2>&1 | head -20`
Expected: no new errors

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ContextMenu.svelte
git commit -m "feat: add ContextMenu component with copy confirmation and viewport overflow guard"
```

---

### Task 3: Wire context menu state into `+page.svelte`

**Files:**
- Modify: `src/routes/+page.svelte`

The page needs:
1. `contextMenu` state (nullable position + items object)
2. `showMenu` helper called by children
3. `<ContextMenu>` rendered inside the `{:else}` (app) block
4. `onShowMenu` prop passed to `Sidebar` and `ChatMain`

- [ ] **Step 1: Add import and state**

In `src/routes/+page.svelte`, add `ContextMenu` to the imports and `ContextMenuItem` to the type import, then add the state variable. Edit the `<script>` block:

```svelte
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
```

(Keep all existing functions — `init`, `signin`, `signup`, `signout`, `loadRooms`, `selectRoom`, `createRoom`, `sendMessage`, `onMount`, `onDestroy` — unchanged.)

- [ ] **Step 2: Pass `onShowMenu` to children and render `<ContextMenu>`**

Replace the `{:else}` block template (the `.app` div and its children) with:

```svelte
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
    />
    <ChatMain
      {activeRoom}
      {messages}
      {err}
      bind:fMsg
      onSendMessage={sendMessage}
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
```

- [ ] **Step 3: Verify no TypeScript errors**

Run: `cd /home/qdust41/Oxyde && npx tsc --noEmit 2>&1 | head -20`
Expected: errors about `onShowMenu` being unknown on `Sidebar` and `ChatMain` — these will be fixed in tasks 4 and 5.

- [ ] **Step 4: Commit**

```bash
git add src/routes/+page.svelte
git commit -m "feat: wire contextMenu state and showMenu helper in page, render ContextMenu"
```

---

### Task 4: Add `onShowMenu` to `Sidebar` and wire room item right-click

**Files:**
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Add prop to interface and destructuring**

In `Sidebar.svelte`, replace the `<script>` section:

```svelte
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
```

- [ ] **Step 2: Wire `oncontextmenu` on `.room-item` buttons**

In the template, replace the `.room-item` button:

```svelte
<button
  class="room-item"
  class:active={activeRoom && full(room.id) === full(activeRoom.id)}
  onclick={() => onSelectRoom(room)}
  oncontextmenu={(e) => onShowMenu(e, [{ label: 'Copy room name', action: () => navigator.clipboard.writeText(room.name) }])}
>
```

- [ ] **Step 3: Verify no TypeScript errors**

Run: `cd /home/qdust41/Oxyde && npx tsc --noEmit 2>&1 | head -20`
Expected: no errors from Sidebar. The `ChatMain` error may remain until task 5.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Sidebar.svelte
git commit -m "feat: add onShowMenu prop to Sidebar, wire room item right-click"
```

---

### Task 5: Add `onShowMenu` to `ChatMain` and wire message/author right-click

**Files:**
- Modify: `src/lib/components/ChatMain.svelte`

- [ ] **Step 1: Add prop to interface and destructuring**

In `ChatMain.svelte`, replace the `<script>` section top (props only):

```svelte
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
```

(Keep `msgEl`, `inputEl`, `scrollBottom`, `autoResize`, `onKey`, `isGrouped`, and both `$effect` calls unchanged.)

- [ ] **Step 2: Wire `oncontextmenu` on `.msg` div and `.msg-author` span**

Replace the message loop in the template. The relevant section currently reads:

```svelte
{#each messages as msg, i (full(msg.id))}
  <div class="msg" class:grouped={isGrouped(i)}>
    {#if !isGrouped(i)}
      <div class="msg-header">
        <span class="msg-author">{msg.author_username ?? sid(msg.author)}</span>
        <span class="msg-time">{fmt(msg.created)}</span>
      </div>
    {/if}
    <p class="msg-body">{msg.body}</p>
  </div>
{/each}
```

Replace with:

```svelte
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
```

- [ ] **Step 3: Verify no TypeScript errors**

Run: `cd /home/qdust41/Oxyde && npx tsc --noEmit 2>&1 | head -20`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ChatMain.svelte
git commit -m "feat: add onShowMenu prop to ChatMain, wire message and author right-click"
```

---

## Self-Review

**Spec coverage check:**

| Spec requirement | Covered by |
|---|---|
| `ContextMenuItem` type in `types.ts` | Task 1 |
| `ContextMenu` component with props `x, y, items, onclose` | Task 2 |
| `position: fixed` at `(x, y)` from `clientX/Y` | Task 2 — `style="left:{x}px; top:{y}px"` |
| Viewport overflow flip on mount | Task 2 — `onMount` checks `rect.right > window.innerWidth` and `rect.bottom > window.innerHeight` |
| Global `onclick` closes menu | Task 2 — `svelte:window onclick={onWindowClick}` |
| Global `onkeydown Escape` closes | Task 2 — `onWindowKey` checks `e.key === 'Escape'` |
| Global `oncontextmenu` closes + prevents default | Task 2 — `onWindowContext` |
| Confirmation "Copied!" for 1200ms then close | Task 2 — `copiedIndex` state + `setTimeout(onclose, 1200)` |
| `var(--accent)` + `var(--accent-soft)` copied state | Task 2 — `.ctx-item.copied` CSS |
| `rise` keyframe entrance animation | Task 2 — reused from page (defined in component) |
| Visual style: surface bg, border, shadow, min-width, padding, font | Task 2 — all present in CSS |
| State in `+page.svelte`, `showMenu` helper | Task 3 |
| `ContextMenu` rendered in app block gated on `contextMenu !== null` | Task 3 |
| `onShowMenu` passed to `Sidebar` | Tasks 3 + 4 |
| `onShowMenu` passed to `ChatMain` | Tasks 3 + 5 |
| Sidebar `.room-item` right-click → "Copy room name" → `room.name` | Task 4 |
| ChatMain `.msg-author` right-click → "Copy username" → `author_username ?? sid(author)` | Task 5 |
| ChatMain `.msg` right-click → "Copy message" → `msg.body` | Task 5 |
| Author `stopPropagation` prevents `.msg` handler | Task 5 — `e.stopPropagation()` on author handler |

All spec requirements covered. No placeholders. Type names consistent across all tasks (`ContextMenuItem`, `onShowMenu`, `contextMenu`).
