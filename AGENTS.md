# AGENTS.md

## 프로젝트 목적

EverSoul AI Chat은 사용자의 PC에서 직접 실행되는 CPU 기반 로컬 LLM 채팅 클라이언트이다.

서비스 UI는 React, TypeScript, Tauri 기반으로 제공한다. 사용자는 설치형 EXE를 실행한 뒤 웹 API를 통해 로그인하고, 서버에서 내려받은 개인화 데이터를 로컬 클라이언트에 적용해 채팅 서비스를 이용한다.

핵심 목표는 서버에서 AI 추론을 수행하지 않고 사용자의 로컬 환경에서 경량 LLM을 실행하는 것이다. 서버는 로그인, 권한 확인, 사용자별 말투, 성격, 채팅 설정 데이터, 지식팩, 업데이트 정보를 제공한다. 로컬 클라이언트는 해당 데이터를 받아 AI 응답의 맥락으로 조립한다.

이 프로젝트에서 학습 데이터는 실제 모델 재학습 데이터를 뜻하지 않는다. 서버에서 내려받은 성격 설정, 말투 예시, 지식 데이터, 대화 규칙을 로컬 컨텍스트로 구성해 응답 품질과 캐릭터성을 제어하는 데이터로 취급한다.

1차 목표는 설치형 EXE 하나로 실행되는 개인화 로컬 AI 채팅 서비스를 만드는 것이다. 장기 목표는 모델 선택, 말투 프로필, 캐릭터 성격, 지식팩, 대화 기록, 업데이트 배포를 웹 서비스와 연동해 관리하는 구조를 완성하는 것이다.

## 응답 및 작업 원칙

- 모든 사용자 응답은 한국어로 작성한다.
- 실제 파일을 확인한 뒤 작업하고, 파일명만으로 구조나 역할을 단정하지 않는다.
- 임시 구현, 더미 데이터, TODO, placeholder, mock, stub 코드를 작성하지 않는다.
- 사용자가 명시하지 않은 기능 삭제, 구조 축소, 우회 구현을 하지 않는다.
- 기존 동작과 설정을 유지하면서 요청된 범위만 수정한다.

## 프로젝트 구조

- 프런트엔드는 `src/`의 React, TypeScript, Vite 구조를 사용한다.
- 데스크톱 런타임은 `src-tauri/`의 Tauri 2 및 Rust 구조를 사용한다.
- 로컬 영속 데이터는 SQLite 저장소를 기준으로 설계한다.
- 로컬 LLM 실행 엔진은 서버 추론이 아니라 사용자 PC의 CPU 실행을 전제로 설계한다.
- 로컬 LLM 모델은 `ai/model/qwen25-3b-korean-Q4_K_M.gguf`로 고정한다 (MyeongHo0621/Qwen2.5-3B-Korean Q4_K_M, Apache 2.0, 한국어 20만 건 파인튜닝).
- `ai/` 디렉터리는 대용량 GGUF 파일을 포함하며 git 추적에서 제외한다. 모델 파일은 사용자가 별도로 배치해야 한다.
- 웹 API는 로그인, 권한 확인, 사용자별 설정, persona pack, style pack, knowledge pack, 업데이트 정보 동기화를 담당한다.
- 정적 자산은 `public/` 또는 `src/assets/`에 둔다.
- VS Code 프로젝트 사전 설정은 `.vscode/extensions.json`, `.vscode/settings.json`, `.vscode/tasks.json`을 추적 대상으로 유지한다.

## 명령어 정책

- 패키지 매니저는 `npm`만 사용한다.
- 빌드 에러 확인 목적의 `npm run dev`, `npm run build`, `npm run tauri` 실행은 사용자 승인 없이 수행하지 않는다.
- 정적 검증이 필요하면 먼저 관련 설정과 소스 파일을 읽고, 허용된 범위에서 타입 확인 명령만 사용한다.

## 구현 기준

- 공유 타입, 유틸리티, 상태 로직은 컴포넌트 내부에 중복 정의하지 않고 적절한 모듈로 분리한다.
- UI 변경 시 기존 컴포넌트의 기능, 필드, 표시 정보, 이벤트 흐름을 제거하지 않는다.
- Tauri 명령, 파일 접근, 외부 호출은 프런트엔드 호출부와 Rust 구현부를 함께 확인한다.
- 서버 동기화 데이터와 로컬 LLM 컨텍스트 조립 로직은 역할을 분리한다.
- persona pack, style pack, knowledge pack은 실제 모델 재학습으로 표현하지 않고 로컬 컨텍스트 구성 데이터로 표현한다.
- 환경 변수나 로컬 비밀값은 `.env.example`에 이름과 용도만 남기고 실제 값은 저장소에 포함하지 않는다.

## 도메인 기반 코드 배치

- `src/domains/<domain>/` 아래에 프런트엔드 도메인 코드를 배치한다.
- 타입은 `types.ts`, 순수 로직은 `logic.ts`, React 훅은 `hooks.ts`, 컴포넌트는 `components/`에 둔다.
- 도메인 외부 공개 API는 `index.ts`에서만 노출한다.
- 다른 도메인의 내부 파일을 직접 가져오지 않고, 해당 도메인의 `index.ts`를 통해 가져온다.
- 공통 UI와 공통 유틸리티는 도메인에 억지로 넣지 않고 `src/shared/`에 둔다.
- Tauri 프런트엔드 호출 코드는 도메인별 client 모듈에 두고, Rust 명령 이름과 요청/응답 타입을 명시적으로 맞춘다.

## Tauri 및 Rust 코드 배치

- `src-tauri/src/domains/<domain>/` 아래에 Rust 도메인 코드를 배치한다.
- Rust 도메인 내부는 `mod.rs`, `types.rs`, `commands.rs`, `services.rs`, `repositories.rs` 역할을 분리한다.
- Tauri command는 `commands.rs`에 두고, 비즈니스 로직은 `services.rs`에 둔다.
- SQLite 접근은 `repositories.rs`에 두며 UI 응답 조립 로직과 섞지 않는다.
- 로컬 LLM 실행, 컨텍스트 조립, 웹 API 동기화는 서로 다른 도메인 또는 서비스 계층으로 분리한다.
- 프런트엔드 타입과 Rust 직렬화 타입은 필드 의미와 단위를 일치시킨다.

## Git 및 AI 도구 파일

- 루트 `.gitignore`가 저장소 전체 무시 규칙의 기준이다.
- `src-tauri/.gitignore` 같은 하위 중복 규칙은 만들지 않는다.
- Antigravity 공유 지침은 `AGENTS.md` 또는 `.agents/**/*.md`에 둔다.
- Antigravity, Gemini, Claude Code, Cursor, Continue, Windsurf, Aider의 로컬 캐시와 개인 설정은 저장소에 포함하지 않는다.

## 작업 워크플로

- 일관성 있는 구현 작업은 `.agents/workflows/domain-implementation.md`를 따른다.
- TypeScript와 React 구조화 작업은 `.agents/skills/typescript-domain-structure/SKILL.md`를 따른다.
- Tauri와 Rust 구조화 작업은 `.agents/skills/tauri-domain-structure/SKILL.md`를 따른다.
