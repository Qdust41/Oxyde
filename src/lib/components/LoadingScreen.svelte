<div class="loading">
  <span class="brand-mark">OXYDE</span>
  <div class="spinner"></div>
</div>

<style>
  .loading {
    display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    height: 100vh; gap: 20px;
    background: var(--bg);
  }
  .brand-mark {
    font-family: 'Cormorant Garamond', Georgia, serif;
    font-size: 32px; font-weight: 700;
    color: var(--accent);
    letter-spacing: 0.22em;
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.45; } }
  .spinner {
    width: 20px; height: 20px;
    border: 1.5px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
