<p align="right">
  <a href="ARCHITECTURE.md"><img src="https://flagcdn.com/20x15/kr.png" width="20" height="15" alt="KR" /> 한국어</a> &nbsp;|&nbsp;
  <img src="https://flagcdn.com/20x15/us.png" width="20" height="15" alt="US" /> <strong>English</strong> &nbsp;|&nbsp;
  <a href="ARCHITECTURE.zh-CN.md"><img src="https://flagcdn.com/20x15/cn.png" width="20" height="15" alt="CN" /> 简体中文</a>
</p>

<h1 align="center">EverSoul AI Chat — Architecture</h1>

## 1. Overall System

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart TB
    UI["React UI<br/>SpiritRoster · ChatStage · SettingsPanel"] == "Tauri invoke<br/>31 commands" ==> CORE

    subgraph CORE["src-tauri/src/domains"]
        direction LR
        D1["chat"]
        D2["persona"]
        D3["llm"]
        D4["training"]
        D5["knowledge · style · settings · auth · sync"]
    end

    CORE --> DB[("SQLite<br/>eversoul.db")]
    CORE --> ENGINE["llama.cpp inference engine<br/>Qwen2.5-3B-Korean GGUF"]
    CORE --> LORA["candle training engine<br/>per-spirit LoRA fine-tuning"]
    LORA -- "adapter converted to GGUF" --> ENGINE

    classDef ui fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef core fill:#e3ddf7,stroke:#4a3aa7,stroke-width:2px,color:#0b0b0b
    classDef store fill:#c9f0d8,stroke:#008300,stroke-width:2px,color:#0b0b0b
    classDef engine fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class UI ui
    class D1,D2,D3,D4,D5 core
    class DB store
    class ENGINE,LORA engine
```

## 2. How Spirit Data Is Made — from TBL source to the chat prompt

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    TBL["ai/tbl_json/<br/>the game's own master data<br/>(Hero, HeroOption, etc.)"]
    STR["ai/tbl_json/String*.json<br/>multi-language string table keyed by sno"]
    SCRIPT["tools/build_complete_personas.cjs<br/>looks up strings by sno, normalizes<br/>into ko/en/zh_tw/zh_cn"]
    JSON["data/personas/*.json<br/>95 spirit files"]
    BIN["src-tauri/resources/personas.bin<br/>pre-compressed resource"]
    SVC["PersonaService<br/>loads into the SQLite persona_profile table"]
    PARSE["parseSpiritDetail(persona, language)<br/>src/domains/persona/logic.ts"]
    PROMPT["conversation system prompt"]

    TBL --> SCRIPT
    STR --> SCRIPT
    SCRIPT --> JSON --> BIN --> SVC --> PARSE --> PROMPT

    classDef src fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef build fill:#fde6b0,stroke:#eda100,stroke-width:2px,color:#0b0b0b
    classDef out fill:#c9f0d8,stroke:#008300,stroke-width:2px,color:#0b0b0b

    class TBL,STR src
    class SCRIPT build
    class JSON,BIN,SVC,PARSE,PROMPT out
```

## 3. How One Turn of Conversation with a Spirit Is Handled

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'actorBkg': '#cde2fb', 'actorBorder': '#2a78d6', 'actorTextColor': '#0b0b0b', 'signalColor': '#52514e', 'signalTextColor': '#0b0b0b', 'noteBkgColor': '#fde6b0', 'noteBorderColor': '#eda100', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
sequenceDiagram
    participant U as User
    participant FE as React UI
    participant CS as ChatService
    participant DB as SQLite
    participant LLM as llama.cpp engine

    U->>FE: types a message
    FE->>CS: invoke(send_message)
    CS->>DB: save the user's message
    CS->>DB: look up spirit profile, knowledge, style, memories
    Note over CS,DB: search persona_memory for<br/>the semantic summary plus 5 relevant episodic memories
    CS->>LLM: request a reply with the system prompt + last 10 messages
    Note over LLM: if a spirit-specific LoRA adapter exists,<br/>mount it into the context before inference
    LLM-->>CS: reply text
    CS->>DB: save the reply
    CS->>LLM: ask itself "was there anything worth remembering?"
    LLM-->>CS: a summary sentence, or "none"
    CS->>LLM: request an embedding of that sentence
    LLM-->>CS: vector
    CS->>DB: save as an episodic memory
    Note over CS,DB: every 10 turns, gather the last 30 episodic<br/>memories and re-summarize into a semantic memory
    CS-->>FE: return the reply
    FE-->>U: shown on screen
```

## 4. Per-Spirit LoRA Fine-Tuning Pipeline

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CHATDB[("SQLite<br/>chat_room · chat_message")]
    COLLECT["TrainingService::collect_training_examples<br/>walks that spirit's chat rooms and<br/>pairs each assistant reply with the history before it"]
    CONV["a list of ConversationExample<br/>(system_prompt, prompt_turns, target_reply)<br/>※ an in-memory Rust struct, not a JSON file"]
    BASE["base model<br/>safetensors + tokenizer<br/>(downloader::ensure_base_model_files)"]
    MODEL["Qwen2Model<br/>built from scratch with candle-core/candle-nn<br/>LoRA rank=8, alpha=16 built in"]
    OPT["AdamW optimizer<br/>lr=1e-4"]
    LOSS["Cross-Entropy Loss<br/>log_softmax + gather, with loss_mask applied"]
    SAVE["save LoRA weights<br/>safetensors"]
    EXPORT["convert to a GGUF adapter<br/>export_lora_to_gguf"]
    DEPLOY["lora_adapters/{persona_id}.gguf"]

    CHATDB --> COLLECT --> CONV --> MODEL
    BASE --> MODEL
    MODEL --> LOSS --> OPT --> MODEL
    OPT --> SAVE --> EXPORT --> DEPLOY

    classDef data fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef train fill:#e3ddf7,stroke:#4a3aa7,stroke-width:2px,color:#0b0b0b
    classDef out fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class CHATDB,COLLECT,CONV,BASE data
    class MODEL,OPT,LOSS,SAVE train
    class EXPORT,DEPLOY out
```

## 5. Local Database Structure

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
erDiagram
    chat_room ||--o{ chat_message : "contains"
    persona_profile ||--o{ chat_room : "spoken to in"
    persona_profile ||--o{ persona_memory : "holds"

    chat_room {
        text id PK
        text title
        text persona_id
        text session_started_at
    }
    chat_message {
        text id PK
        text room_id FK
        text role "user, assistant, system"
        text content
    }
    persona_profile {
        text id PK
        text name
        text name_en
        text grade
        text race
        text class
        text system_prompt
        text raw_json
    }
    persona_memory {
        text id PK
        text persona_id FK
        text memory_type "episodic, semantic"
        text memory_text
        blob memory_vector
    }
    style_profile {
        text id PK
        text tone
        integer is_active
    }
    knowledge_chunk {
        text id PK
        text document_name
        text chunk_text
    }
    auth_session {
        integer id PK
        text token
        text email
    }
```

## 6. Build Pipeline

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CMD["npm run tauri build"]
    FE["npm run build:frontend<br/>tsc && vite build → dist/"]
    RS["cargo build --release<br/>compiles the llama-cpp-2 binding<br/>(needs CMake, Clang, MSVC)"]
    OPT["release profile<br/>codegen-units=1 · lto=true<br/>opt-level=3 · panic=abort · strip"]
    OUT["packaged installer<br/>src-tauri/target/release"]

    CMD --> FE --> RS --> OPT --> OUT

    classDef step fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef opt fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class CMD,FE,RS step
    class OPT,OUT opt
```
