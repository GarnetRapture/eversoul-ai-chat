import React, { useState, useEffect, useRef } from "react";
import { authClient } from "./domains/auth";
import { personaClient, PersonaConfig } from "./domains/persona";
import { chatClient, ChatRoom, ChatMessage } from "./domains/chat";
import { llmClient, LlmStatus } from "./domains/llm";

// DB persona_profile 에 적재되어 있는 raw_json 문자열을 파싱한 실제 메타데이터 규격
interface SpiritDetail {
  id: string;
  name: string;
  name_en: string;
  grade: string;
  race: string;
  class: string;
  sub_class: string;
  stat: string;
  profile: {
    nick_name: string | null;
    constellation: string | null;
    union: string | null;
    birthday: string | null;
    height: number | null;
    weight: number | null;
    cv_ko: string | null;
    cv_jp: string | null;
    like: string[];
    dislike: string[];
    hobby: string[];
    speciality: string[];
  };
  personality: {
    description: string | null;
    greeting: string | null;
  };
  speech_patterns: string[];
  comments: Array<{
    writer: string;
    comment: string;
  }>;
}

// 실제 정령 종족에 맞추어 아바타 그라디언트 배경색을 동적 결정하는 헬퍼 함수
function getRaceGradient(race: string): string {
  switch (race) {
    case "인간형": return "from-red-900 to-pink-950";
    case "요정형": return "from-emerald-950 to-teal-900";
    case "야수형": return "from-amber-950 to-orange-900";
    case "불사형": return "from-purple-900 to-indigo-950";
    case "천사형": return "from-amber-700 to-yellow-950";
    case "악마형": return "from-violet-950 to-fuchsia-950";
    default: return "from-slate-700 to-zinc-900";
  }
}

export default function App() {
  // 로딩 및 앱 전체 상태
  const [appInitializing, setAppInitializing] = useState(true);
  const [llmStatus, setLlmStatus] = useState<LlmStatus | null>(null);
  
  // 100% DB 기반의 정령 설정 목록 보관 상태
  const [spirits, setSpirits] = useState<PersonaConfig[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeSpiritId, setActiveSpiritId] = useState<string>("");
  
  // 현재 선택된 정령의 DB 세부 데이터팩 파싱 객체
  const [activeDetail, setActiveDetail] = useState<SpiritDetail | null>(null);
  const [isLoadingDetail, setIsLoadingDetail] = useState(false);

  // 채팅 상태
  const [activeRoom, setActiveRoom] = useState<ChatRoom | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState("");
  const [isTyping, setIsTyping] = useState(false);

  // 채팅 자동 스크롤 레퍼런스
  const messageEndRef = useRef<HTMLDivElement>(null);

  // 1. 앱 마운트 시 초기화 (자동 로그인, LLM 엔진 탑재 및 100% SQLite3 DB 데이터 로드)
  useEffect(() => {
    async function initApp() {
      try {
        // 백엔드 FFI 세션 수립
        const session = await authClient.getSession();
        if (!session) {
          await authClient.login("garnet@everlib.pro", "ever_token_secret");
        }

        // 로컬 LLM 로드
        let status = await llmClient.getStatus();
        if (!status.is_loaded) {
          status = await llmClient.loadEngine();
        }
        setLlmStatus(status);

        // [100% DB 연동 핵심]: SQLite3 persona_profile 테이블 전체 데이터를 일괄 조회 (SELECT * FROM persona_profile)
        const dbList = await personaClient.list();
        
        // 이름 기준 가나다 정렬 적재
        const sortedList = dbList.sort((a, b) => a.name.localeCompare(b.name));
        setSpirits(sortedList);

        // 기본 활성 정령으로 garnet(가넷) 지정
        const garnetRecord = sortedList.find(p => p.id === "garnet");
        if (garnetRecord) {
          handleSelectSpirit(garnetRecord.id, garnetRecord);
        } else if (sortedList.length > 0) {
          handleSelectSpirit(sortedList[0].id, sortedList[0]);
        }
      } catch (err) {
        console.error("앱 초기화 중 오류 발생:", err);
      } finally {
        setAppInitializing(false);
      }
    }

    initApp();
  }, []);

  // 대화 추가 시 하단 스크롤 자동 이동
  useEffect(() => {
    messageEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, isTyping]);

  // 2. 정령 프리셋 선택 (100% 로컬 SQLite3 데이터 쿼리 연계)
  async function handleSelectSpirit(spiritId: string, preloadedRecord?: PersonaConfig) {
    setIsLoadingDetail(true);
    setActiveSpiritId(spiritId);
    try {
      // (1) 상태 리스트 또는 DB 레코드로부터 데이터 확보
      const record = preloadedRecord || spirits.find(p => p.id === spiritId);
      if (!record) return;

      // (2) DB에 저장된 raw_json 문자열을 파싱하여 세부 프로필 및 스펙 획득 (완벽한 100% DB 매핑)
      const detail: SpiritDetail = JSON.parse(record.raw_json);
      setActiveDetail(detail);

      // (3) 정령 전용 대화방 개설/조회
      const roomTitle = `${record.name}의 인연스토리`;
      const rooms = await chatClient.listRooms();
      let matchedRoom = rooms.find(r => r.title === roomTitle);

      if (!matchedRoom) {
        matchedRoom = await chatClient.createRoom(roomTitle);
      }
      setActiveRoom(matchedRoom);

      // (4) SQLite3 DB 메시지 테이블에서 대화 이력 조회
      const history = await chatClient.listMessages(matchedRoom.id);
      
      if (history.length === 0) {
        setMessages([
          {
            id: "greeting-init",
            room_id: matchedRoom.id,
            role: "assistant",
            content: record.greeting || "안녕, 구원자님! 기다리고 있었어.",
            created_at: new Date().toLocaleTimeString()
          }
        ]);
      } else {
        setMessages(history);
      }
    } catch (err) {
      console.error("정령 DB 로드 및 채팅 연동 실패:", err);
    } finally {
      setIsLoadingDetail(false);
    }
  }

  // 3. 메시지 전송 및 LLM 인연채팅 응답 처리
  async function handleSendMessage(e: React.FormEvent) {
    e.preventDefault();
    if (!inputText.trim() || !activeRoom || isTyping || !activeDetail) return;

    const userText = inputText;
    setInputText("");

    const userMsg: ChatMessage = {
      id: `user-${Date.now()}`,
      room_id: activeRoom.id,
      role: "user",
      content: userText,
      created_at: new Date().toLocaleTimeString()
    };
    setMessages(prev => [...prev, userMsg]);
    setIsTyping(true);

    try {
      // 백엔드 FFI: 인연채팅 전송 (SQLite3의 정령 프로필 기반으로 추론 수행)
      const reply = await chatClient.sendMessage(activeRoom.id, userText, activeSpiritId);
      setMessages(prev => [...prev, reply]);
    } catch (err) {
      console.error("채팅 응답 수집 실패:", err);
      setMessages(prev => [...prev, {
        id: `err-${Date.now()}`,
        room_id: activeRoom.id,
        role: "assistant",
        content: "[연결 오류] 로컬 AI 엔진에서 응답을 수집하지 못했습니다. 설정과 모델 파일을 확인하십시오.",
        created_at: new Date().toLocaleTimeString()
      }]);
    } finally {
      setIsTyping(false);
    }
  }

  // SQLite3 DB 목록 대상 검색 필터
  const filteredList = spirits.filter(s => {
    return s.name.includes(searchQuery) ||
           s.name_en.toLowerCase().includes(searchQuery.toLowerCase());
  });

  if (appInitializing) {
    return (
      <div className="absolute inset-0 bg-bg-main flex flex-col items-center justify-center z-50">
        <div className="w-12 h-12 border-4 border-pink-neon/10 border-t-pink-neon rounded-full animate-spin mb-4"></div>
        <div className="font-title text-sm text-text-secondary tracking-widest animate-pulse">
          에버톡 SQLite3 로컬 데이터베이스 커넥션 연결 중...
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      {/* 1. 좌측 패널 (정령 리스트) */}
      <aside className="sidebar">
        <div className="sidebar-header">
          <div className="sidebar-title-row">
            <h1 className="sidebar-title">EVER TALK</h1>
            <span className={`engine-status-badge ${llmStatus?.is_loaded ? "loaded" : "loading"}`}>
              {llmStatus?.is_loaded ? "CPU LLM ON" : "LOADING ENGINE"}
            </span>
          </div>
          <div className="search-wrapper">
            <input
              type="text"
              placeholder="정령 이름 검색..."
              className="search-input"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        <div className="spirit-list">
          {filteredList.map((spirit) => {
            const isActive = activeSpiritId === spirit.id;
            const firstChar = spirit.name.charAt(0);
            const raceGrad = getRaceGradient(spirit.race);

            return (
              <div
                key={spirit.id}
                className={`spirit-item ${isActive ? "active" : ""}`}
                onClick={() => handleSelectSpirit(spirit.id, spirit)}
              >
                <div className={`avatar-container bg-linear-to-tr ${raceGrad} text-white`}>
                  {firstChar}
                </div>
                <div className="spirit-info">
                  <div className="spirit-name-row">
                    <span className="spirit-name">{spirit.name}</span>
                    <span className={`spirit-badge ${spirit.grade === "에픽" ? "epic" : spirit.grade === "레어" ? "rare" : "common"}`}>
                      {spirit.grade}
                    </span>
                  </div>
                  <div className="spirit-subtext">{spirit.race} • {spirit.class}</div>
                </div>
              </div>
            );
          })}
        </div>
      </aside>

      {/* 2. 중앙 패널 (에버톡 채팅 인터페이스) */}
      <main className="chat-area">
        {activeDetail ? (
          <>
            {/* 채팅창 헤더 */}
            <div className="chat-header">
              <div className="chat-header-info">
                <h2 className="chat-header-name">
                  {activeDetail.name}
                </h2>
                <p className="chat-header-status text-xs truncate">
                  {activeDetail.personality?.greeting || "구원자님, 반갑습니다."}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-pink-neon text-xs font-semibold px-2.5 py-1 bg-pink-neon/10 rounded-full border border-pink-neon/20">
                  인연레벨 99
                </span>
              </div>
            </div>

            {/* 메시지 스트림 */}
            <div className="chat-messages-scroll">
              {messages.map((msg) => {
                const isUser = msg.role === "user";
                const raceGrad = getRaceGradient(activeDetail.race);
                return (
                  <div key={msg.id} className={`message-row ${isUser ? "user" : "assistant"}`}>
                    {!isUser && (
                      <div className={`w-8 h-8 rounded-full flex items-center justify-center font-bold text-xs bg-linear-to-tr ${raceGrad} text-white shrink-0 mt-2`}>
                        {activeDetail.name.charAt(0)}
                      </div>
                    )}
                    <div className="message-bubble-wrapper">
                      <span className="message-sender">
                        {isUser ? "구원자" : activeDetail.name}
                      </span>
                      <div className="message-bubble">
                        {msg.content}
                      </div>
                    </div>
                  </div>
                );
              })}

              {/* 답장 타이핑 인디케이터 */}
              {isTyping && (
                <div className="message-row assistant">
                  <div className={`w-8 h-8 rounded-full flex items-center justify-center font-bold text-xs bg-linear-to-tr ${getRaceGradient(activeDetail.race)} text-white shrink-0 mt-2`}>
                    {activeDetail.name.charAt(0)}
                  </div>
                  <div className="message-bubble-wrapper">
                    <span className="message-sender">
                      {activeDetail.name}
                    </span>
                    <div className="message-bubble">
                      <div className="typing-dots">
                        <span></span>
                        <span></span>
                        <span></span>
                      </div>
                    </div>
                  </div>
                </div>
              )}
              <div ref={messageEndRef} />
            </div>

            {/* 입력창 */}
            <div className="chat-input-area">
              <form className="chat-input-form" onSubmit={handleSendMessage}>
                <input
                  type="text"
                  placeholder={`${activeDetail.name}에게 톡 보내기...`}
                  className="chat-input"
                  value={inputText}
                  onChange={(e) => setInputText(e.target.value)}
                  disabled={isTyping}
                />
                <button
                  type="submit"
                  className="chat-send-btn"
                  disabled={isTyping || !inputText.trim()}
                >
                  보내기
                </button>
              </form>
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-text-muted">
            <span className="text-5xl mb-4">💬</span>
            <p className="font-title text-sm tracking-wide">대화할 정령을 선택하여 메신저를 시작하십시오.</p>
          </div>
        )}
      </main>

      {/* 3. 우측 패널 (정령 도감 상세 프로필) */}
      <aside className="profile-panel">
        {isLoadingDetail ? (
          <div className="flex-1 flex flex-col items-center justify-center text-text-muted p-6 text-center">
            <div className="w-8 h-8 border-2 border-pink-neon/20 border-t-pink-neon rounded-full animate-spin mb-3"></div>
            <p className="text-xs">SQLite3 DB 프로필 로딩 중...</p>
          </div>
        ) : activeDetail ? (
          <>
            <div className="profile-art-card">
              <div className="profile-art-overlay">
                {activeDetail.name_en}
              </div>
              <div className={`profile-art-avatar bg-linear-to-tr ${getRaceGradient(activeDetail.race)}`}>
                {activeDetail.name.charAt(0)}
              </div>
            </div>

            <div className="profile-body">
              <h3 className="profile-section-title">스펙 & 프로필</h3>
              <div className="profile-details-grid">
                <div className="profile-detail-card">
                  <div className="detail-label">소속</div>
                  <div className="detail-value text-pink-neon">{activeDetail.profile?.union || "-"}</div>
                </div>
                <div className="profile-detail-card">
                  <div className="detail-label">성우 (KO / JP)</div>
                  <div className="detail-value text-xs">{activeDetail.profile?.cv_ko || "-"} / {activeDetail.profile?.cv_jp || "-"}</div>
                </div>
                <div className="profile-detail-card">
                  <div className="detail-label">생일 / 별자리</div>
                  <div className="detail-value">{activeDetail.profile?.birthday || "-"} / {activeDetail.profile?.constellation || "-"}</div>
                </div>
                <div className="profile-detail-card">
                  <div className="detail-label">신체 스펙</div>
                  <div className="detail-value">{activeDetail.profile?.height ? `${activeDetail.profile.height}cm` : "-"} / {activeDetail.profile?.weight ? `${activeDetail.profile.weight}kg` : "-"}</div>
                </div>
              </div>

              <h3 className="profile-section-title">선호 & 호불호</h3>
              <div className="tag-list">
                {activeDetail.profile?.like?.map((item: string, idx: number) => (
                  <span key={`like-${idx}`} className="tag-badge like">👍 {item}</span>
                ))}
                {activeDetail.profile?.dislike?.map((item: string, idx: number) => (
                  <span key={`dislike-${idx}`} className="tag-badge dislike">👎 {item}</span>
                ))}
              </div>

              <h3 className="profile-section-title">정령 소개</h3>
              <div className="profile-desc-box">
                {activeDetail.personality?.description || "소개 정보가 등록되지 않은 정령입니다."}
              </div>

              {activeDetail.comments && activeDetail.comments.length > 0 && (
                <>
                  <h3 className="profile-section-title">영지 정령 평판</h3>
                  <div className="flex flex-col gap-2">
                    {activeDetail.comments.slice(0, 3).map((cmt: any, idx: number) => (
                      <div key={`cmt-${idx}`} className="comment-item">
                        <div className="comment-writer-row">
                          <span className="comment-writer">@{cmt.writer}</span>
                          <span className="comment-badge">친밀</span>
                        </div>
                        <div className="comment-content">"{cmt.comment}"</div>
                      </div>
                    ))}
                  </div>
                </>
              )}
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-text-muted p-6 text-center">
            <p className="text-xs">정령을 선택하면 상세 프로필 카드가 여기에 노출됩니다.</p>
          </div>
        )}
      </aside>
    </div>
  );
}
