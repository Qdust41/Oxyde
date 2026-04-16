# Context Menu — Design Spec
**Date:** 2026-04-15
**Status:** Approved

## Overview

Custom right-click context menu for the Oxyde chat app. Replaces the browser default. Context-aware: menu items differ based on the element right-clicked. Copy-only for now, with a "Copied!" confirmation. Built with Approach A — shared component, state lifted to `+page.svelte`.

---

## 1. New Type

Add to `src/lib/types.ts`:

```ts
export interface ContextMenuItem { label: string; action: () => void }
```

---

## 2. New Component

**File:** `src/lib/components/ContextMenu.svelte`

### Props
```ts
{ x: number; y: number; items: ContextMenuItem[]; onclose: () => void }
```

### Positioning
- `position: fixed` at `(x, y)` from `MouseEvent.clientX/Y`
- On mount: check if menu overflows viewport right or bottom edge; if so, flip left/upward
- Immune to scroll

### Dismiss
- Global `onclick` on `svelte:window` closes menu (menu container stops propagation)
- Global `onkeydown` closes on `Escape`
- Global `oncontextmenu` on `svelte:window` closes and prevents default (stops stale menu persisting on second right-click)
- Selecting an item closes after 1200ms (post-confirmation)

### Copy & Confirmation
- Copy via `navigator.clipboard.writeText()`
- On click: item label changes to `"Copied!"`, color shifts to `var(--accent)` with `var(--accent-soft)` background
- After 1200ms: menu closes
- Uses per-item `copiedIndex` state (index of last-copied item)

---

## 3. State in `+page.svelte`

```ts
let contextMenu = $state<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

function showMenu(e: MouseEvent, items: ContextMenuItem[]) {
  e.preventDefault();
  contextMenu = { x: e.clientX, y: e.clientY, items };
}
```

`ContextMenu` renders at the bottom of the `{:else}` (app) block, gated on `contextMenu !== null`:

```svelte
{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={contextMenu.items}
    onclose={() => contextMenu = null}
  />
{/if}
```

`showMenu` is passed as a prop to both `Sidebar` and `ChatMain`.

---

## 4. Trigger Targets

| Component | Element | Right-click handler | Menu item | Copies |
|---|---|---|---|---|
| `Sidebar` | `.room-item` button | `oncontextmenu` | "Copy room name" | `room.name` |
| `ChatMain` | `.msg-author` span | `oncontextmenu` + `stopPropagation` | "Copy username" | `msg.author_username ?? sid(msg.author)` |
| `ChatMain` | `.msg` div | `oncontextmenu` | "Copy message" | `msg.body` |

Author `stopPropagation` prevents the parent `.msg` handler from also firing.

### Prop additions
- `Sidebar`: `onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void`
- `ChatMain`: `onShowMenu: (e: MouseEvent, items: ContextMenuItem[]) => void`

---

## 5. Visual Style

| Property | Value |
|---|---|
| Background | `var(--surface)` |
| Border | `1px solid var(--border)` |
| Border radius | `var(--r)` (2px) |
| Box shadow | `0 4px 16px rgba(0,0,0,0.4)` |
| Min width | 160px |
| List padding | 4px |
| Item padding | `7px 12px` |
| Font | `inherit` (Martian Mono), 11px |
| Item color | `var(--text-2)` |
| Item hover | bg `var(--surface-2)`, color `var(--text)`, left border `2px solid var(--accent)` |
| Copied state | color `var(--accent)`, bg `var(--accent-soft)` |
| Entrance animation | Reuse existing `rise` keyframe (opacity + translateY, 0.15s) |

---

## 6. Files Changed

| File | Change |
|---|---|
| `src/lib/types.ts` | Add `ContextMenuItem` interface |
| `src/lib/components/ContextMenu.svelte` | New component |
| `src/routes/+page.svelte` | Add state, `showMenu` helper, render `ContextMenu`, pass prop to children |
| `src/lib/components/Sidebar.svelte` | Add `onShowMenu` prop, wire `oncontextmenu` on room items |
| `src/lib/components/ChatMain.svelte` | Add `onShowMenu` prop, wire `oncontextmenu` on `.msg` and `.msg-author` |
