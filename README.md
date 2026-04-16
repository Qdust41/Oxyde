# Oxyde

Tauri 2 Desktop Application built with SvelteKit 5 + TypeScript + SurrealDB

## Project Overview
This is a native desktop application featuring:
- ✅ Modern SvelteKit 5 frontend with TypeScript
- ✅ Tauri 2 runtime for native desktop performance
- ✅ Embedded SurrealDB database for local storage
- ✅ Authentication system
- ✅ Chat interface
- ✅ Native system capabilities through Tauri plugins

---

## Development Setup

### Prerequisites
First install required dependencies:

| Tool | Required Version |
|------|------------------|
| Rust | 1.75+ |
| Node.js | 20+ |
| pnpm | 9+ |
| System Dependencies | See Tauri requirements for your OS |

#### System Specific Setup:

**Linux (Debian/Ubuntu):**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**Windows:**
- Install Visual Studio Build Tools with "Desktop development with C++" workload
- Install WebView2 Runtime (included in Windows 11+)

**macOS:**
```bash
xcode-select --install
brew install gtk+3
```

---

### Installation

1. **Clone and install dependencies:**
```bash
git clone <repository-url>
cd oxyde
pnpm install
```

2. **Verify Rust setup:**
```bash
rustc --version
cargo --version
```

3. **First run will compile all Rust dependencies:**
```bash
# Full native development mode
./run-tauri-dev.sh
```

---

### Available Scripts

| Command | Description |
|---------|-------------|
| `pnpm dev` | Run web-only dev server (browser, no Tauri) |
| `pnpm tauri dev` | Run full native Tauri application |
| `./run-tauri-dev.sh` | Run Tauri dev with Linux GPU fix |
| `pnpm build` | Build production web assets |
| `pnpm tauri build` | Create native installers/bundles |
| `pnpm check` | Run TypeScript + Svelte type checking |
| `pnpm check:watch` | Watch mode for type checking |

---

### Project Structure

```
oxyde/
├── src/                     # SvelteKit Frontend
│   ├── lib/
│   │   ├── components/      # Reusable Svelte components
│   │   ├── helpers.ts       # Utility functions
│   │   └── types.ts         # TypeScript type definitions
│   ├── routes/              # Application routes
│   └── app.html
├── src-tauri/               # Rust Backend
│   ├── src/
│   │   ├── commands/        # Tauri command handlers (chat, user)
│   │   ├── db.rs            # SurrealDB integration
│   │   ├── error.rs         # Error handling
│   │   ├── models.rs        # Data models
│   │   ├── lib.rs
│   │   └── main.rs          # App entry point
│   ├── Cargo.toml
│   └── tauri.conf.json
├── surreal/                 # Database schemas
├── static/                  # Static assets
└── docs/                    # Project documentation
```

---

### Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) / VSCodium with extensions:
- [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)

---

### Troubleshooting

**Linux GPU Rendering Issues:**
Use the provided `./run-tauri-dev.sh` script which disables DMA-BUF renderer.

**Slow first build:**
First run will compile all Rust crates, this is normal. Subsequent builds will be incremental and much faster.

**Rust dependency issues:**
```bash
cd src-tauri && cargo clean
```

---

## Tech Stack
| Layer | Technology |
|-------|------------|
| Frontend | SvelteKit 5, TypeScript, Vite |
| Runtime | Tauri 2 |
| Backend | Rust |
| Database | SurrealDB 3 |
| Package Manager | pnpm |