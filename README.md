<!-- Banner -->
<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=3b82f6&height=180&section=header&text=EverSoul%20AI%20Chat&fontSize=60&fontAlignY=45&animation=twinkling" alt="EverSoul AI Chat Banner" />
</p>

<!-- Badges -->
<p align="center">
  <img src="https://img.shields.io/badge/version-0.0.6-blue?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/license-Apache_2.0-green?style=flat-square" alt="License" />
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC107?style=flat-square&logo=tauri" alt="Tauri" />
  <img src="https://img.shields.io/badge/React-19.0-61DAFB?style=flat-square&logo=react" alt="React" />
  <img src="https://img.shields.io/badge/Rust-2021-000000?style=flat-square&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/SQLite-3.0-003B57?style=flat-square&logo=sqlite" alt="SQLite" />
</p>

---

## 🌟 개요

**EverSoul AI Chat**은 사용자의 PC 환경에서 완전히 독립적으로 동작하는 **CPU 기반 로컬 LLM 개인화 채팅 클라이언트**입니다. 

클라우드 서버나 외부 API 호출 없이 로컬 CPU 자원만으로 인공지능 응답을 추론하므로, 개인정보가 외부로 유출되지 않는 강력한 보안 환경을 제공합니다. 서버는 사용자 고유의 성격/말투 설정 및 데이터 동기화만을 담당하며, 모든 LLM 실시간 추론과 대화 관리는 사용자의 로컬 클라이언트 내부에서 독립적으로 처리됩니다.

---

## 🚀 주요 기능

- 💻 **100% 로컬 CPU 추론**: GPU 없이도 최적화된 속도로 동작하는 로컬 LLM 실행 구조 설계.
- 🎭 **캐릭터 개인화 패키지**: Persona Pack, Style Pack, Knowledge Pack 데이터를 로컬 컨텍스트로 조립하여 고유한 말투와 지식을 지닌 대화 상대 구현.
- 📂 **SQLite 기반 영속화**: 로컬 대화 세션 및 메시지 기록을 경량 데이터베이스를 통해 안전하게 보관.
- 🔄 **서버 프로필 동기화**: 웹 API 연동을 통해 다양한 디바이스 간 채팅 메타 정보 및 성격 팩 안전하게 백업.

---

## 🛠 기술 스택

### Frontend Stack
- **Framework**: `React 19` + `TypeScript 6` + `Vite 8`
- **State Management**: `TanStack React Query v5` (비동기 데이터), `Zustand v5` (전역 클라이언트 상태)
- **Styling**: `Vanilla CSS` + `clsx` (클래스 믹싱)
- **Icons**: `lucide-react`

### Desktop Runtime & Backend Stack
- **Core Runtime**: `Tauri v2` (Rust 2021 edition)
- **Local Database**: `SQLite3` (Rust `rusqlite` bundled)
- **HTTP Client**: `reqwest` (API 서버 연동 및 데이터 동기화)
- **AI Inference Engine**: `llama.cpp` (Rust `llama-cpp-2` C-bindings)
- **Serialization / Utilities**: `serde`, `serde_json`, `anyhow`, `thiserror`, `tracing`, `uuid`

---

## 📦 로컬 모델 설정

본 프로젝트는 고품질 한국어 성능을 보장하기 위해 단일 고정 모델을 채택하고 있으며, 대용량 파일이기 때문에 Git 추적에서 제외되어 있습니다. 어플리케이션을 구동하기 전 수동으로 아래 경로에 모델 파일을 배치해 주십시오.

### 모델 정보
- **이름**: `MyeongHo0621/Qwen2.5-3B-Korean Q4_K_M`
- **다운로드 주소**: [Hugging Face 다운로드 페이지](https://huggingface.co/MyeongHo0621/Qwen2.5-3B-Korean/resolve/main/gguf/qwen25-3b-korean-Q4_K_M.gguf)
- **위치**: 프로젝트 루트 하위 `ai/model/`
- **파일명**: `qwen25-3b-korean-Q4_K_M.gguf`

```bash
# 디렉터리가 없을 시 생성
mkdir -p ai/model

# 다운로드 후 이동 예시 (Windows PowerShell)
Invoke-WebRequest -Uri "https://huggingface.co/MyeongHo0621/Qwen2.5-3B-Korean/resolve/main/gguf/qwen25-3b-korean-Q4_K_M.gguf" -OutFile "ai/model/qwen25-3b-korean-Q4_K_M.gguf"
```

---

## 💻 실행 및 빌드 가이드

### 빌드 사전 요구사항
로컬 LLM 추론 바인딩(`llama-cpp-2`)을 빌드하기 위해 아래 도구들의 사전 설치가 필요합니다.
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) (C++ 컴파일러 포함)
- [CMake](https://cmake.org/download/) (버전 3.20 이상)
- [Clang](https://releases.llvm.org/download.html) (Bindgen용 C/C++ 파서)

### 의존성 설치
```bash
npm install
```

### 개발 모드 실행
```bash
npm run tauri dev
```

### 상용 배포 빌드
```bash
npm run tauri build
```

---

## 📄 라이선스

This project is licensed under the **Apache License 2.0**.
GGUF Model (`Qwen2.5-3B-Korean`) is created by `MyeongHo0621` and distributed under **Apache License 2.0**.
