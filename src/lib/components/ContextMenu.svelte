<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
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
  let closeTimer: ReturnType<typeof setTimeout> | null = null;

  // Flip position if menu would overflow viewport
  onMount(() => {
    if (!menuEl) return;
    const rect = menuEl.getBoundingClientRect();
    if (rect.right > window.innerWidth)  menuEl.style.left = (x - rect.width)  + 'px';
    if (rect.bottom > window.innerHeight) menuEl.style.top = (y - rect.height) + 'px';
  });

  onDestroy(() => {
    if (closeTimer !== null) clearTimeout(closeTimer);
  });

  function handleItem(item: ContextMenuItem, index: number) {
    item.action();
    copiedIndex = index;
    closeTimer = setTimeout(onclose, 1200);
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
  onkeydown={(e) => e.stopPropagation()}
  oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); }}
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
    margin: 0;
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
