<p align="right">
  <img src="https://flagcdn.com/20x15/kr.png" width="20" height="15" alt="KR" /> <strong>한국어</strong> &nbsp;|&nbsp;
  <a href="ARCHITECTURE.en.md"><img src="https://flagcdn.com/20x15/us.png" width="20" height="15" alt="US" /> English</a> &nbsp;|&nbsp;
  <a href="ARCHITECTURE.zh-CN.md"><img src="https://flagcdn.com/20x15/cn.png" width="20" height="15" alt="CN" /> 简体中文</a>
</p>

<h1 align="center">EverSoul AI Chat — 아키텍처</h1>

## 1. 전체 시스템

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart TB
    UI["React UI<br/>SpiritRoster · ChatStage · SettingsPanel"] == "Tauri invoke<br/>31개 커맨드" ==> CORE

    subgraph CORE["src-tauri/src/domains"]
        direction LR
        D1["chat"]
        D2["persona"]
        D3["llm"]
        D4["training"]
        D5["knowledge · style · settings · auth · sync"]
    end

    CORE --> DB[("SQLite<br/>eversoul.db")]
    CORE --> ENGINE["llama.cpp 추론 엔진<br/>Qwen2.5-3B-Korean GGUF"]
    CORE --> LORA["candle 학습 엔진<br/>정령별 LoRA 파인튜닝"]
    LORA -- "GGUF로 변환된 어댑터" --> ENGINE

    classDef ui fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef core fill:#e3ddf7,stroke:#4a3aa7,stroke-width:2px,color:#0b0b0b
    classDef store fill:#c9f0d8,stroke:#008300,stroke-width:2px,color:#0b0b0b
    classDef engine fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class UI ui
    class D1,D2,D3,D4,D5 core
    class DB store
    class ENGINE,LORA engine
```

## 2. 정령 데이터가 만들어지는 과정 — TBL 원본 → 대화 프롬프트

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    TBL["ai/tbl_json/<br/>게임 원본 마스터데이터<br/>(Hero, HeroOption 등)"]
    STR["ai/tbl_json/String*.json<br/>sno 기준 다국어 문자열 테이블"]
    SCRIPT["tools/build_complete_personas.cjs<br/>sno로 문자열 조회 후<br/>ko/en/zh_tw/zh_cn 정규화"]
    JSON["data/personas/*.json<br/>정령 95개 파일"]
    BIN["src-tauri/resources/personas.bin<br/>사전 압축 리소스"]
    SVC["PersonaService<br/>SQLite persona_profile 테이블로 적재"]
    PARSE["parseSpiritDetail(persona, language)<br/>src/domains/persona/logic.ts"]
    PROMPT["대화 시스템 프롬프트"]

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

## 3. 정령과 나눈 대화 한 턴이 처리되는 순서

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'actorBkg': '#cde2fb', 'actorBorder': '#2a78d6', 'actorTextColor': '#0b0b0b', 'signalColor': '#52514e', 'signalTextColor': '#0b0b0b', 'noteBkgColor': '#fde6b0', 'noteBorderColor': '#eda100', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
sequenceDiagram
    participant U as 사용자
    participant FE as React UI
    participant CS as ChatService
    participant DB as SQLite
    participant LLM as llama.cpp 엔진

    U->>FE: 메시지 입력
    FE->>CS: invoke(send_message)
    CS->>DB: 사용자 메시지 저장
    CS->>DB: 정령 프로필 · 지식 · 스타일 · 기억 조회
    Note over CS,DB: persona_memory에서<br/>semantic 요약 + 관련 episodic 기억 5건 검색
    CS->>LLM: 시스템 프롬프트 + 최근 대화 10개로 추론 요청
    Note over LLM: 정령 전용 LoRA 어댑터가 있으면<br/>컨텍스트에 장착 후 추론
    LLM-->>CS: 응답 텍스트
    CS->>DB: 응답 메시지 저장
    CS->>LLM: "기억할 게 있었는지" 자문 프롬프트
    LLM-->>CS: 요약 문장 또는 "없음"
    CS->>LLM: 문장 임베딩 요청
    LLM-->>CS: 벡터
    CS->>DB: episodic 기억 저장
    Note over CS,DB: 10턴마다 episodic 30건을 모아<br/>semantic 기억으로 재요약
    CS-->>FE: 응답 반환
    FE-->>U: 화면에 표시
```

## 4. 정령별 LoRA 파인튜닝 파이프라인

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CHATDB[("SQLite<br/>chat_room · chat_message")]
    COLLECT["TrainingService::collect_training_examples<br/>정령별 대화방을 순회하며<br/>assistant 메시지마다 그 이전 대화 이력을 묶음"]
    CONV["ConversationExample 목록<br/>(system_prompt, prompt_turns, target_reply)<br/>※ JSON 파일이 아니라 메모리상의 Rust 구조체"]
    BASE["베이스 모델<br/>safetensors + tokenizer<br/>(downloader::ensure_base_model_files)"]
    MODEL["Qwen2Model<br/>candle-core/candle-nn로 직접 구현<br/>LoRA rank=8, alpha=16 내장"]
    OPT["AdamW 옵티마이저<br/>lr=1e-4"]
    LOSS["Cross-Entropy Loss<br/>log_softmax + gather, loss_mask 적용"]
    SAVE["LoRA 가중치 저장<br/>safetensors"]
    EXPORT["GGUF 어댑터로 변환<br/>export_lora_to_gguf"]
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

## 5. 로컬 데이터베이스 구조

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
erDiagram
    chat_room ||--o{ chat_message : "포함"
    persona_profile ||--o{ chat_room : "대화 상대"
    persona_profile ||--o{ persona_memory : "기억 보유"

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

## 6. 빌드 파이프라인

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart LR
    CMD["npm run tauri build"]
    FE["npm run build:frontend<br/>tsc && vite build → dist/"]
    RS["cargo build --release<br/>llama-cpp-2 바인딩 컴파일<br/>(CMake · Clang · MSVC 필요)"]
    OPT["release 프로파일<br/>codegen-units=1 · lto=true<br/>opt-level=3 · panic=abort · strip"]
    OUT["설치 파일 패키징<br/>src-tauri/target/release"]

    CMD --> FE --> RS --> OPT --> OUT

    classDef step fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef opt fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b

    class CMD,FE,RS step
    class OPT,OUT opt
```
