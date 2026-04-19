<script lang="ts">
  interface Props {
    authMode: 'signin' | 'signup';
    err: string;
    fEmail: string;
    fPass: string;
    fUser: string;
    onSignin: () => void;
    onSignup: () => void;
    onToggleMode: () => void;
  }

  let {
    authMode,
    err,
    fEmail = $bindable(),
    fPass  = $bindable(),
    fUser  = $bindable(),
    onSignin,
    onSignup,
    onToggleMode,
  }: Props = $props();
</script>

<div class="auth-wrap">
  <div class="auth-card">
    <h1 class="auth-brand">OXYDE</h1>
    <p class="auth-tagline">realtime · native · focused</p>

    {#if err}
      <div class="err-banner">{err}</div>
    {/if}

    {#if authMode === 'signin'}
      <div class="field-stack">
        <input class="field" type="email" placeholder="email" bind:value={fEmail} autocomplete="email" />
        <input class="field" type="password" placeholder="password" bind:value={fPass}
          onkeydown={(e) => e.key === 'Enter' && onSignin()} autocomplete="current-password" />
        <button class="btn-primary" onclick={onSignin}>sign in</button>
      </div>
      <button class="btn-ghost" onclick={onToggleMode}>
        no account? create one →
      </button>
    {:else}
      <div class="field-stack">
        <input class="field" type="text"     placeholder="username" bind:value={fUser} autocomplete="username" />
        <input class="field" type="email"    placeholder="email"    bind:value={fEmail} autocomplete="email" />
        <input class="field" type="password" placeholder="password" bind:value={fPass}
          onkeydown={(e) => e.key === 'Enter' && onSignup()} autocomplete="new-password" />
        <button class="btn-primary" onclick={onSignup}>create account</button>
      </div>
      <button class="btn-ghost" onclick={onToggleMode}>
        ← back to sign in
      </button>
    {/if}
  </div>
</div>

<style>
  .auth-wrap {
    display: flex; align-items: center; justify-content: center;
    height: 100vh; background: var(--bg);
    animation: rise 0.28s ease;
  }
  @keyframes rise {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .auth-card {
    width: 360px; padding: 52px 44px;
    background: var(--sidebar-bg);
    border: 1px solid var(--border);
    border-radius: var(--r);
  }
  .auth-brand {
    font-family: 'Cormorant Garamond', Georgia, serif;
    font-size: 52px; font-weight: 700;
    color: var(--accent); letter-spacing: 0.22em;
    text-align: center;
  }
  .auth-tagline {
    text-align: center; color: var(--muted);
    font-size: 9.5px; letter-spacing: 0.15em;
    margin-top: 8px; margin-bottom: 36px;
  }
  .err-banner {
    padding: 10px 14px; margin-bottom: 18px;
    background: rgba(184, 48, 48, 0.10);
    border: 1px solid rgba(184, 48, 48, 0.28);
    border-radius: var(--r);
    color: #d98080; font-size: 11px; line-height: 1.5;
  }
  .field-stack { display: flex; flex-direction: column; gap: 10px; margin-bottom: 14px; }
  .field {
    width: 100%; padding: 10px 14px;
    background: var(--bg);
    border: 1px solid var(--border); border-radius: var(--r);
    color: var(--text); font-family: inherit; font-size: 12px;
    outline: none; transition: border-color 0.12s;
  }
  .field:focus { border-color: var(--accent); }
  .field::placeholder { color: var(--muted); }
  .btn-primary {
    width: 100%; padding: 11px;
    background: var(--accent); border: none; border-radius: var(--r);
    color: #fff; font-family: inherit; font-size: 12px;
    font-weight: 500; letter-spacing: 0.07em;
    cursor: pointer; transition: opacity 0.12s, transform 0.08s;
  }
  .btn-primary:hover  { opacity: 0.85; }
  .btn-primary:active { transform: scale(0.98); }
  .btn-ghost {
    display: block; width: 100%; text-align: center;
    padding: 9px; background: none; border: none;
    color: var(--muted); font-family: inherit; font-size: 11px;
    cursor: pointer; transition: color 0.12s;
  }
  .btn-ghost:hover { color: var(--text-2); }
</style>
