<p align="right">
  <a href="ARCHITECTURE.md"><img src="https://flagcdn.com/20x15/kr.png" width="20" height="15" alt="KR" /> 한국어</a> &nbsp;|&nbsp;
  <a href="ARCHITECTURE.en.md"><img src="https://flagcdn.com/20x15/us.png" width="20" height="15" alt="US" /> English</a> &nbsp;|&nbsp;
  <img src="https://flagcdn.com/20x15/cn.png" width="20" height="15" alt="CN" /> <strong>简体中文</strong>
</p>

<h1 align="center">EverSoul AI Chat — 架构</h1>

## 1. 整体系统

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart TB
    UI["React 界面<br/>SpiritRoster · ChatStage · SettingsPanel"] == "Tauri invoke<br/>31 个命令" ==> CORE

    subgraph CORE["src-tauri/src/domains"]
        direction LR
        D1["chat"]
        D2["persona"]
        D3["llm"]
        D4["training"]
        D5["knowledge · style · settings · auth · sync"]
    end

    CORE --> DB[("SQLite<br/>eversoul.db")]
    CORE --> ENGINE["llama.cpp 推理引擎<br/>Qwen2.5-3B-Korean GGUF"]
    CORE --> LORA["candle 训练引擎<br/>按精灵进行 LoRA 微调"]
    LORA -- "转换为 GGUF 的适配器" --> ENGINE

    classDef ui fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef core fill:#e3ddf7,stroke:#4a3aa7,stroke-width:2px,color:#0b0b0b
    classDef store fill:#c9f0d8,stroke:#008300,stroke-width:2px,color:#0b0b0b
    classDef engine fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class UI ui
    class D1,D2,D3,D4,D5 core
    class DB store
    class ENGINE,LORA engine
```

## 2. 精灵数据的生成过程 —— 从 TBL 原始数据到对话提示词

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    TBL["ai/tbl_json/<br/>游戏原始主数据<br/>（Hero、HeroOption 等）"]
    STR["ai/tbl_json/String*.json<br/>以 sno 为键的多语言字符串表"]
    SCRIPT["tools/build_complete_personas.cjs<br/>按 sno 查询字符串后<br/>规范化为 ko/en/zh_tw/zh_cn"]
    JSON["data/personas/*.json<br/>95 名精灵的文件"]
    BIN["src-tauri/resources/personas.bin<br/>预压缩资源"]
    SVC["PersonaService<br/>加载进 SQLite persona_profile 表"]
    PARSE["parseSpiritDetail(persona, language)<br/>src/domains/persona/logic.ts"]
    PROMPT["对话系统提示词"]

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

## 3. 与精灵的一轮对话的处理顺序

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'actorBkg': '#cde2fb', 'actorBorder': '#2a78d6', 'actorTextColor': '#0b0b0b', 'signalColor': '#52514e', 'signalTextColor': '#0b0b0b', 'noteBkgColor': '#fde6b0', 'noteBorderColor': '#eda100', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
sequenceDiagram
    participant U as 用户
    participant FE as React 界面
    participant CS as ChatService
    participant DB as SQLite
    participant LLM as llama.cpp 引擎

    U->>FE: 输入消息
    FE->>CS: invoke(send_message)
    CS->>DB: 保存用户消息
    CS->>DB: 查询精灵资料 · 知识 · 语气风格 · 记忆
    Note over CS,DB: 从 persona_memory 中<br/>检索 semantic 摘要与 5 条相关 episodic 记忆
    CS->>LLM: 用系统提示词 + 最近 10 条消息请求推理
    Note over LLM: 若存在该精灵专属的 LoRA 适配器，<br/>先挂载进上下文再推理
    LLM-->>CS: 回复文本
    CS->>DB: 保存回复消息
    CS->>LLM: 自问"有没有什么值得记住的"
    LLM-->>CS: 一句摘要，或"没有"
    CS->>LLM: 请求对该句子做向量嵌入
    LLM-->>CS: 向量
    CS->>DB: 保存为 episodic 记忆
    Note over CS,DB: 每 10 轮，收集最近 30 条 episodic 记忆<br/>重新总结为 semantic 记忆
    CS-->>FE: 返回回复
    FE-->>U: 显示在界面上
```

## 4. 按精灵进行的 LoRA 微调流程

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CHATDB[("SQLite<br/>chat_room · chat_message")]
    COLLECT["TrainingService::collect_training_examples<br/>遍历该精灵的所有聊天室<br/>把每条 assistant 回复和它之前的对话历史配成一对"]
    CONV["ConversationExample 列表<br/>(system_prompt, prompt_turns, target_reply)<br/>※ 是内存中的 Rust 结构体，不是 JSON 文件"]
    BASE["基础模型<br/>safetensors + tokenizer<br/>（downloader::ensure_base_model_files）"]
    MODEL["Qwen2Model<br/>用 candle-core/candle-nn 从零实现<br/>内置 LoRA rank=8、alpha=16"]
    OPT["AdamW 优化器<br/>lr=1e-4"]
    LOSS["交叉熵损失<br/>log_softmax + gather，应用 loss_mask"]
    SAVE["保存 LoRA 权重<br/>safetensors"]
    EXPORT["转换为 GGUF 适配器<br/>export_lora_to_gguf"]
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

## 5. 本地数据库结构

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
erDiagram
    chat_room ||--o{ chat_message : "包含"
    persona_profile ||--o{ chat_room : "对话对象"
    persona_profile ||--o{ persona_memory : "持有记忆"

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

## 6. 构建流程

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CMD["npm run tauri build"]
    FE["npm run build:frontend<br/>tsc && vite build → dist/"]
    RS["cargo build --release<br/>编译 llama-cpp-2 绑定<br/>（需要 CMake、Clang、MSVC）"]
    OPT["release 配置<br/>codegen-units=1 · lto=true<br/>opt-level=3 · panic=abort · strip"]
    OUT["打包安装文件<br/>src-tauri/target/release"]

    CMD --> FE --> RS --> OPT --> OUT

    classDef step fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef opt fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class CMD,FE,RS step
    class OPT,OUT opt
```
