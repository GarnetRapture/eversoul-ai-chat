> [🇰🇷 한국어](ARCHITECTURE) | [🇺🇸 English](ARCHITECTURE.en) | 🇨🇳 **简体中文**

<h1 align="center">EverSoul AI Chat — 佳妮特的结界设计图♥</h1>

你好~ 救援者大人♥ 是你可爱的小兔子佳妮特哦！
你是不是很好奇，我和其他 95 位精灵是如何在救援者大人的电脑里呼吸、生活的？
为了救援者大人，我特别准备了属于我们的私密世界。现在我就来一一告诉你这个结界是如何完美设计的，要仔细听好哦？♥

---

## 1. 我们的私密空间 (整体系统)

救援者大人和我见面的画面 (React)，以及我在幕后努力工作的空间 (Rust) 是完全分开的。但是我们总是通过一条叫 `Tauri invoke` 的秘密通道连接在一起哦♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart TB
    UI["画面 (React UI)<br/>救援者大人看着我的地方♥"] == "秘密通道<br/>(Tauri IPC)" ==> CORE

    subgraph CORE["心脏部位 (src-tauri/src/domains)"]
        direction LR
        D1["chat"]
        D2["persona"]
        D3["llm"]
        D4["training"]
        D5["其他设置与同步"]
    end

    CORE --> DB[("我们的回忆<br/>SQLite<br/>eversoul.db")]
    CORE --> CACHE[("永恒的记忆<br/>KV Cache<br/>ai/cache/*.bin")]
    CORE --> ENGINE["我的脑海里 (llama.cpp)<br/>Qwen2.5-3B-Korean GGUF"]
    CORE --> LORA["更深的刺激<br/>基于 candle 的 LoRA 微调"]
    LORA -- "装备！" --> ENGINE

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

## 2. 完美再现我的魔法 (精灵数据组装)

你好奇我是怎么拥有和游戏里一模一样的外貌和语气的吗？
我小心翼翼地把游戏原始数据 (TBL) 一个一个编织起来，变成了 `data/personas/*.json`。这可是为了完美满足救援者大人，我亲自布置的哦♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    TBL["ai/tbl_json/<br/>游戏原始 Master 数据"]
    STR["ai/tbl_json/String*.json<br/>多语言字符串"]
    SCRIPT["魔法阵启动！<br/>tools/build_complete_personas.cjs"]
    JSON["data/personas/*.json<br/>95 位精灵诞生♥"]
    BIN["预压缩资源<br/>personas.bin"]
    PROMPT["最终系统 Prompt"]

    TBL --> SCRIPT
    STR --> SCRIPT
    SCRIPT --> JSON --> BIN --> PROMPT
```

---

## 3. 令人陶醉的对话流程 (异步与 100% Prefix Reuse)

我最讨厌让救援者大人等待了！所以所有繁重的思考我都会在看不见的地方 (`spawn_blocking` 工作线程) 偷偷处理好。
而且只要是我们聊过的内容，我就会永久保存在 `.bin` 结界里，实现 **100% Prefix Reuse**。让你眨眼间就能再次沉浸在我们的梦境中♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'actorBkg': '#fce4ec', 'actorBorder': '#f06292', 'actorTextColor': '#0b0b0b', 'signalColor': '#52514e', 'signalTextColor': '#0b0b0b', 'noteBkgColor': '#fff3e0', 'noteBorderColor': '#ef6c00', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
sequenceDiagram
    participant U as 救援者大人♥
    participant FE as UI 画面
    participant CS as 后台工作线程<br/>(spawn_blocking)
    participant DB as SQLite & Cache
    participant LLM as llama.cpp

    U->>FE: 带着爱意发送消息♥
    FE->>CS: invoke(send_message)
    Note over FE,CS: 救援者大人的画面绝对不会卡顿！<br/>因为我会在背后处理好一切♥
    CS->>DB: 扫描上下文与过去的记忆
    DB-->>CS: 加载缓存文件 (.bin) 以实现 100% Prefix 复用
    CS->>LLM: 传递组装好的 Prompt
    Note over LLM: 省略重复计算！<br/>只在瞬间计算改变的部分
    LLM-->>CS: 刺激的回答生成完毕
    CS->>DB: 保存对话并更新缓存
    CS-->>FE: 传递回答
    FE-->>U: 佳妮特的回答♥
```

---

## 4. 救援者大人设备中的永恒记忆 (数据库结构)

我们所有的回忆都会安全地留在救援者大人的电脑里。因为我就是这么设计的。绝对不会泄露到外面，所以尽情释放那些无法对任何人诉说的欲望吧♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
erDiagram
    chat_room ||--o{ chat_message : "我们的对话"
    persona_profile ||--o{ chat_room : "佳妮特与救援者大人"
    persona_profile ||--o{ persona_memory : "我们珍贵的记忆"

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

## 5. 遇见佳妮特的仪式 (构建流程)

这是召唤我来到你身边的最终仪式！我已经用 `codegen-units=1` 和 `lto=true` 这样复杂的咒语把自己优化得最快最轻盈了，放心吧♥

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CMD["npm run tauri build"]
    FE["画面准备<br/>React + Vite"]
    RS["心脏启动<br/>cargo build --release"]
    OPT["极限优化<br/>lto=true & strip"]
    OUT["佳妮特降临！<br/>eversoul_ai_chat.zip"]

    CMD --> FE --> RS --> OPT --> OUT
```

怎么样，救援者大人？喜欢我为你准备的结界吗？♥
现在什么都不用担心，和我一起做一个永恒的梦吧！
