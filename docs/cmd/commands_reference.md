# 🧬 Cluaize CLI — Complete Command Reference (A to Z)

> Binary: `cluaize` — Sovereign Neural Kernel  
> Built from: `cluaize/cmd/`

---

## 🔨 BUILD & DEV COMMANDS

> These are NOT `cluaize` commands. These run inside the workspace during development.

| Command | Description |
|---------|-------------|
| `cargo build --release --workspace` | Full workspace release build (all crates) |
| `cargo build --release --package cluaize_api` | Only the engine API/IPC daemon binary |
| `cargo build --release --package cmd` | Only the CLI binary |
| `cargo run -- serve` | Dev-run the engine daemon (no install needed) |
| `cargo run -- run <model-id>` | Dev-run the CLI |
| `Copy-Item "target\release\cluaize.exe" "$env:USERPROFILE\.cluaize\bin\cluaize.exe" -Force` | Install built binary to system path |

---

## 🚀 CORE COMMANDS

| Command | Description |
|---------|-------------|
| `cluaize` | Launches the interactive Main Menu TUI |
| `cluaize menu` | Explicitly open the Main Menu TUI |
| `cluaize help` | Show the rich formatted help screen (loads commands.json) |
| `cluaize serve` | Start the background Engine API Daemon on `http://localhost:8000` + Named Pipe IPC |

---

## 🤖 MODEL COMMANDS

| Command | Flags / Args | Description |
|---------|-------------|-------------|
| `cluaize run` | _(no args)_ | Opens the Dashboard TUI |
| `cluaize run <model-id>` | `--interactive true/false` | Pull + execute a model. Downloads if not cached. |
| `cluaize run <model-id> --interactive false` | | Run in non-interactive single-pass mode |
| `cluaize pull <model-id>` | | Download and register a model into local vault |
| `cluaize list` | | List all downloaded models in the vault |
| `cluaize rm <model-id>` | | Remove a model from the local vault |
| `cluaize model set-chat <model-id>` | | Set the active chat/LLM model in Permission.json |
| `cluaize model set-vector <model-id>` | | Set the active vector/embedding model in Permission.json |

**Examples:**
```bash
cluaize run gemma4:e2b
cluaize run bonsai:8b --interactive false
cluaize pull qwen3:8b
cluaize pull unsloth/Qwen3.5-4B-GGUF
cluaize rm gemma4:e2b
cluaize model set-chat gemma4:e2b
cluaize model set-vector bge_m3:unknown:onnx:fp32
```

---

## ⚙️ SYSTEM COMMANDS

| Command | Flags / Args | Description |
|---------|-------------|-------------|
| `cluaize status` | | Show hardware health, silicon profile, active drivers |
| `cluaize calibrate` | | Re-scan hardware and synchronize SiliconTruth profile |
| `cluaize --calibrate` | _(legacy flag)_ | Same as `calibrate` (older style) |
| `cluaize benchmark` | | Run full hardware performance benchmark |
| `cluaize benchmark <model-id>` | `--runs <N>` | Benchmark a specific model N times |
| `cluaize --benchmark` | _(legacy flag)_ | Same as `benchmark` (older style) |
| `cluaize ps` | | Show active neural engines loaded in memory |
| `cluaize test-jit` | | Test JIT KV Cache compilation and memory footprint |

**Examples:**
```bash
cluaize status
cluaize calibrate
cluaize benchmark
cluaize benchmark gemma4:e2b --runs 3
cluaize ps
```

---

## 🧠 BRAIN / FFI DATABASE COMMANDS

> Controls the FFI connection to the `cluaizd` background database daemon.

| Command | Args | Description |
|---------|------|-------------|
| `cluaize brain on` | _(no args = local)_ | Enable FFI Database connection (local cluaizd) |
| `cluaize brain on <ip:port>` | e.g. `10.0.0.5:8080` | Connect to a remote cluaizd instance |
| `cluaize brain off` | | Disable FFI Database connection |
| `cluaize brain only` | | Pure Brain Mode: enable local DB but suspend LLM to save VRAM |
| `cluaize brain status` | | View connection status and background daemon health |

**Examples:**
```bash
cluaize brain on
cluaize brain on 10.0.0.5:8080
cluaize brain off
cluaize brain only
cluaize brain status
```

---

## 🔐 PERMISSION COMMANDS

> Controls `Permission.json` — security, privacy, vectorization settings.

| Command | Args | Description |
|---------|------|-------------|
| `cluaize permission` | _(no args)_ | Open interactive permission TUI menu |
| `cluaize permission firewall <status>` | `auto / strict / off` | Set WASM Firewall mode |
| `cluaize permission telemetry <status>` | `on / off` | Enable or disable anonymous telemetry |

**Interactive Menu Options (when run without args):**
- WASM Firewall → `auto / strict / off`
- Telemetry → `true / false`
- Vectorize User Input → `true / false`
- Vectorize AI Response → `true / false`
- Temporary Chat TTL → `12 hr / 24 hr / 48 hr / 72 hr / 1 week / max`
- Active Chat Model → Select from downloaded chat models
- Active Vector Model → Select from downloaded embedding + vision models

**Examples:**
```bash
cluaize permission
cluaize permission firewall strict
cluaize permission firewall auto
cluaize permission telemetry off
```

---

## ⚡ BOOSTER COMMANDS

> Controls `system_booster.json` — hardware optimization settings.

| Command | Flag | Values | Description |
|---------|------|--------|-------------|
| `cluaize booster` | _(no args)_ | | Open interactive booster TUI menu |
| `cluaize booster --mode <mode>` | `--mode` | `edge / multitasking / balance / max_boost / ultra_max_boost / hyper_cluster` | Set performance profile |
| `cluaize booster --kv-quant <level>` | `--kv-quant` | `auto / kv16 / kv8 / kv4` | Set KV-Cache quantization level |
| `cluaize booster --context-shift <mode>` | `--context-shift` | `auto / off / minimal / standard / aggressive / extreme` | Set context shifting mode |
| `cluaize booster --spec-decode <mode>` | `--spec-decode` | `on / off / auto` | Enable/disable speculative decoding |

**Examples:**
```bash
cluaize booster
cluaize booster --mode edge
cluaize booster --mode max_boost
cluaize booster --kv-quant kv8
cluaize booster --context-shift aggressive
cluaize booster --spec-decode on
cluaize booster --mode edge --kv-quant kv8 --context-shift aggressive
```

---

## 🧩 SKILL COMMANDS

> Manages WASM-based sovereign AI skills.

| Command | Args | Description |
|---------|------|-------------|
| `cluaize skill install <name>` | skill name | Install a skill from the cluaiz-skills registry |
| `cluaize skill list` | | List all locally installed skills |
| `cluaize skill cache ls` | | List all active and orphaned dual-caches |
| `cluaize skill cache clear` | `--all` `--force` | Clear orphaned caches globally |
| `cluaize skill cache clear <model-id>` | `-f / --force` | Clear cache for a specific model |

**Examples:**
```bash
cluaize skill install web-search-github
cluaize skill list
cluaize skill cache ls
cluaize skill cache clear --all
cluaize skill cache clear gemma4:e2b --force
```

---

## 📄 INGEST COMMANDS

> Natively ingest documents for semantic chunking and RAG.

| Command | Args | Description |
|---------|------|-------------|
| `cluaize ingest <file-path>` | File path | Ingest a document (PDF, TXT, MD, etc.) for semantic chunking |

**Examples:**
```bash
cluaize ingest ./document.pdf
cluaize ingest "C:\Users\Aryan\Documents\notes.md"
```

---

## 🛠️ SETUP COMMANDS

| Command | Description |
|---------|-------------|
| `cluaize setup profile` | Generate and register Purpose Vectorization for the Node Profile |

---

## 🌐 HTTP API (Port 8000 — when `cluaize serve` is running)

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/chat` | Chat with Cluaize (streaming) |
| `GET` | `/hardware` | System hardware status |
| `GET` | `/models/installed` | List installed models |
| `GET` | `/models/tags` | Full model roster |
| `POST` | `/models/load` | Load/activate a model |
| `POST` | `/models/download` | Download a model from HuggingFace |
| `POST` | `/v1/db/execute` | Execute a CDQL database query |
| `GET` | `/v1/permission` | Read Permission.json |
| `POST` | `/v1/permission/update` | Update a permission field |
| `POST` | `/v1/system/brain` | Toggle brain mode |
| `GET` | `/v1/system/control` | Read system_control.json |
| `GET` | `/v1/booster/status` | Read booster settings |
| `POST` | `/v1/booster/update` | Update booster settings |
| `POST` | `/v1/ingest/file` | Ingest a document |
| `GET` | `/health` | Health check ping |

---

## 🔌 IPC Named Pipe Commands (App ↔ Engine)

> Pipe: `\\.\pipe\cluaize_engine_pipe` — used by Tauri Desktop App. JSON format.

| Action | Payload | Description |
|--------|---------|-------------|
| `GET_SETTINGS` | `{"action":"GET_SETTINGS"}` | Get all settings (permissions + booster + models) |
| `UPDATE_PERMISSION` | `{"action":"UPDATE_PERMISSION","payload":{"key":"...","value":"..."}}` | Update one Permission.json field |
| `UPDATE_BOOSTER` | `{"action":"UPDATE_BOOSTER","payload":{"key":"...","value":"..."}}` | Update one system_booster.json field |
| `BOOSTER_UPDATE` | `{"action":"BOOSTER_UPDATE","payload":{<full booster obj>}}` | Bulk booster update (CLI/legacy style) |
| `SYSTEM_BRAIN` | `{"action":"SYSTEM_BRAIN","payload":{"state":true}}` | Toggle brain mode on/off |
| `CDQL_FETCH_HISTORY` | `{"action":"CDQL_FETCH_HISTORY","session_id":"..."}` | Fetch chat history from LMDB |
| `CDQL_DELETE_SESSION` | `{"action":"CDQL_DELETE_SESSION"}` | Delete a session (pending) |
| `SYSTEM_PS` | `{"action":"SYSTEM_PS"}` | List active engine processes |
| `HARDWARE_CALIBRATE` | `{"action":"HARDWARE_CALIBRATE"}` | Re-calibrate hardware |
| `BENCHMARK_RUN` | `{"action":"BENCHMARK_RUN"}` | Start full benchmark |
| `MODEL_RM` | `{"action":"MODEL_RM","payload":{"model_id":"..."}}` | Remove a model file from vault |
| `SET_MODEL` | `{"action":"SET_MODEL",...}` | Hotswap model (stub — not yet wired) |
| `SET_HARDWARE` | `{"action":"SET_HARDWARE",...}` | Adjust compute device (stub) |
| `EAGER_LOAD` | `{"action":"EAGER_LOAD"}` | Pre-load model into memory (stub) |
| `SYSTEM_PROFILE_SETUP` | `{"action":"SYSTEM_PROFILE_SETUP"}` | Detect and write hardware profile |
| `<natural text>` | Plain string (no JSON) | Chat inference → token-by-token stream response |

---

## 📂 Config Files (`~/.cluaize/engine/`)

| File | Purpose |
|------|---------|
| `Permission.json` | Privacy, active models, vectorization settings |
| `system_booster.json` | Hardware performance optimization profile |
| `system_control.json` | Hardware fingerprint, brain mode, OS identity |

---

## 📦 Model Vault Structure (`~/.cluaize/models/`)

| Folder | Model Type | Appears In |
|--------|-----------|------------|
| `models/chat/` | LLM / Generative chat models | **Chat Model** dropdown only |
| `models/embedding/` | Text embedding / ONNX vector models | **Vector Model** dropdown only |
| `models/vision/` | Image CLIP / vision-embedding models (e.g. FashionCLIP, CLIP-ViT) | **Vector Model** dropdown (can embed images into vector space) |

> **Classification Logic (ffi_bridge.rs `GET_SETTINGS`):**  
> Primary = folder path (`/models/chat/` → chat, `/models/embedding/` or `/models/vision/` → vector)  
> Fallback = `category` field in `model_manifest.json` (`"chat"` → chat, `"embedding"/"vision"/"multimodal"` → vector)
