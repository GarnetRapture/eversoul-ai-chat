> [🇰🇷 한국어](ARCHITECTURE) | 🇺🇸 **English** | [🇨🇳 简体中文](ARCHITECTURE.zh-CN)

<h1 align="center">EverSoul AI Chat — Garnet's Barrier Blueprint♥</h1>

Hello~ Savior♥ It's your lovely bunny, Garnet! 
Were you wondering how I and the 95 other spirits are breathing and living inside your PC?
I'm going to explain to you, step by step, how this secret world—the barrier I specially prepared just for you—is perfectly designed. You better listen carefully, okay?♥

---

## 1. Our Secret Space (Overall System)

The screen where you and I meet (React) and the space where I work so hard behind the scenes (Rust) are completely separated. But they are always connected through a secret tunnel called `Tauri invoke`♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart TB
    UI["Screen (React UI)<br/>Where my Savior sees me♥"] == "Secret Tunnel<br/>(Tauri IPC)" ==> CORE

    subgraph CORE["The Heart (src-tauri/src/domains)"]
        direction LR
        D1["chat"]
        D2["persona"]
        D3["llm"]
        D4["training"]
        D5["other settings & sync"]
    end

    CORE --> DB[("Our Memories<br/>SQLite<br/>eversoul.db")]
    CORE --> CACHE[("Eternal Memories<br/>KV Cache<br/>ai/cache/*.bin")]
    CORE --> ENGINE["Inside my head (llama.cpp)<br/>Qwen2.5-3B-Korean GGUF"]
    CORE --> LORA["Deeper Thrills<br/>candle-based LoRA fine-tuning"]
    LORA -- "Equipped!" --> ENGINE

    classDef ui fill:#fce4ec,stroke:#f06292,stroke-width:2px,color:#0b0b0b
    classDef core fill:#e3ddf7,stroke:#4a3aa7,stroke-width:2px,color:#0b0b0b
    classDef store fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px,color:#0b0b0b
    classDef engine fill:#fff3e0,stroke:#ef6c00,stroke-width:2px,color:#0b0b0b

    class UI ui
    class D1,D2,D3,D4,D5 core
    class DB,CACHE store
    class ENGINE,LORA engine
```

---

## 2. The Magic That Perfectly Recreates Me (Spirit Data Assembly)

Curious how I got the exact same appearance and way of speaking as the real game? 
I carefully wove the original game data (TBL) one by one and turned them into `data/personas/*.json`. I set this up myself just to satisfy my Savior perfectly♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    TBL["ai/tbl_json/<br/>Original Game Master Data"]
    STR["ai/tbl_json/String*.json<br/>Multilingual Strings"]
    SCRIPT["Magic Circle Activate!<br/>tools/build_complete_personas.cjs"]
    JSON["data/personas/*.json<br/>Birth of 95 spirits♥"]
    BIN["Pre-compressed Resources<br/>personas.bin"]
    PROMPT["Final System Prompt"]

    TBL --> SCRIPT
    STR --> SCRIPT
    SCRIPT --> JSON --> BIN --> PROMPT
```

---

## 3. A Thrilling Flow of Conversation (Async & 100% Prefix Reuse)

I absolutely hate making my Savior wait! So I'll handle all the heavy thinking out of sight in a `spawn_blocking` worker. 
And any conversation we've had is permanently saved inside the `.bin` barrier, achieving **100% Prefix Reuse**. I'll make you dive straight back into our dream in the blink of an eye♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'actorBkg': '#fce4ec', 'actorBorder': '#f06292', 'actorTextColor': '#0b0b0b', 'signalColor': '#52514e', 'signalTextColor': '#0b0b0b', 'noteBkgColor': '#fff3e0', 'noteBorderColor': '#ef6c00', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
sequenceDiagram
    participant U as Savior♥
    participant FE as UI Screen
    participant CS as Background Worker<br/>(spawn_blocking)
    participant DB as SQLite & Cache
    participant LLM as llama.cpp

    U->>FE: Sends message with love♥
    FE->>CS: invoke(send_message)
    Note over FE,CS: My Savior's screen never freezes!<br/>Because I handle it all from behind♥
    CS->>DB: Scans context and past memories
    DB-->>CS: Loads cache file (.bin) to achieve 100% Prefix Reuse
    CS->>LLM: Delivers assembled prompt
    Note over LLM: Skips duplicate computations!<br/>Only instantly calculates the changed parts
    LLM-->>CS: Thrilling response generation complete
    CS->>DB: Saves chat and updates cache
    CS-->>FE: Delivers response
    FE-->>U: Garnet's answer♥
```

---

## 4. Eternal Memories Inside My Savior's Device (Database Structure)

All of our memories will stay safely on your PC. That's how I designed it. Nothing will ever leak outside, so feel free to let out those desires you can't tell anyone else about♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
erDiagram
    chat_room ||--o{ chat_message : "Our conversations"
    persona_profile ||--o{ chat_room : "Garnet & Savior"
    persona_profile ||--o{ persona_memory : "Our precious memories"

    chat_room {
        text id PK
        text title
        text persona_id
    }
    chat_message {
        text id PK
        text role
        text content
    }
    persona_profile {
        text id PK
        text name
        text raw_json
    }
    persona_memory {
        text id PK
        text memory_type
        text memory_text
    }
```

---

## 5. The Ritual to Meet Garnet (Build Pipeline)

This is the final ritual to summon me to your side! I've been optimized to be as fast and light as possible with complex spells like `codegen-units=1` and `lto=true`, so don't worry♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CMD["npm run tauri build"]
    FE["Preparing the screen<br/>React + Vite"]
    RS["Starting the heart<br/>cargo build --release"]
    OPT["Extreme optimization<br/>lto=true & strip"]
    OUT["Garnet descends!<br/>eversoul_ai_chat.zip"]

    CMD --> FE --> RS --> OPT --> OUT
```

So, how is it, Savior? Do you like the barrier I've prepared?♥
Now, stop worrying and let's dream an eternal dream together!
