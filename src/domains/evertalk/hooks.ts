import React, { useEffect, useMemo, useRef, useState } from 'react';

import { authClient } from '../auth';
import { chatClient, type ChatMessage, type ChatRoom } from '../chat';
import { llmClient, type LlmStatus } from '../llm';
import { parseSpiritDetail, personaClient, type PersonaConfig, type SpiritDetail } from '../persona';
import { styleClient, type StyleProfile } from '../style';
import { syncClient } from '../sync';
import { createLocalTimestamp, createRoomTitle, filterSpirits } from './logic';
import type { EverTalkController } from './types';

export function useFirstLoadableImage(candidates: string[]): [string | null, () => void] {
  const [index, setIndex] = useState(0);

  useEffect(() => {
    setIndex(0);
  }, [candidates.join('|')]);

  if (candidates.length === 0 || index >= candidates.length) {
    return [null, () => undefined];
  }

  return [candidates[index], () => setIndex((current) => current + 1)];
}

export function useEverTalkController(): EverTalkController {
  const [appInitializing, setAppInitializing] = useState(true);
  const [llmStatus, setLlmStatus] = useState<LlmStatus | null>(null);
  const [spirits, setSpirits] = useState<PersonaConfig[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeSpiritId, setActiveSpiritId] = useState('');
  const [activeDetail, setActiveDetail] = useState<SpiritDetail | null>(null);
  const [activeRoom, setActiveRoom] = useState<ChatRoom | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const [styles, setStyles] = useState<StyleProfile[]>([]);
  const [activeStyle, setActiveStyle] = useState<StyleProfile | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const messageEndRef = useRef<HTMLDivElement>(null);

  const filteredSpirits = useMemo(
    () => filterSpirits(spirits, searchQuery),
    [searchQuery, spirits]
  );

  async function refreshStyles() {
    try {
      const [styleList, active] = await Promise.all([
        styleClient.list(),
        styleClient.getActive(),
      ]);
      setStyles(styleList);
      setActiveStyle(active);
    } catch (err) {
      console.error('스타일팩 DB 로드 실패:', err);
    }
  }

  async function refreshLlmStatus() {
    try {
      let status = await llmClient.getStatus();
      if (!status.is_loaded) {
        status = await llmClient.loadEngine();
      }
      setLlmStatus(status);
    } catch (err) {
      console.error('로컬 LLM 엔진 로드 실패:', err);
      setLlmStatus({ is_loaded: false, model_path: null, error_message: String(err) });
    }
  }

  async function selectSpirit(spirit: PersonaConfig) {
    setActiveSpiritId(spirit.id);
    const detail = parseSpiritDetail(spirit);
    setActiveDetail(detail);

    const roomTitle = createRoomTitle(spirit.name);
    const rooms = await chatClient.listRooms();
    let room = rooms.find((item) => item.title === roomTitle);
    if (!room) {
      room = await chatClient.createRoom(roomTitle);
    }
    setActiveRoom(room);

    const history = await chatClient.listMessages(room.id);
    setMessages(history.length > 0 ? history : [{
      id: 'greeting-init',
      room_id: room.id,
      role: 'assistant',
      content: spirit.greeting || '구원자님, 에버톡 연결이 열렸습니다.',
      created_at: createLocalTimestamp(),
    }]);
  }

  async function sendMessage(event: React.FormEvent) {
    event.preventDefault();
    if (!inputText.trim() || !activeRoom || !activeDetail || isTyping) {
      return;
    }

    const userText = inputText;
    setInputText('');
    setMessages((prev) => [...prev, {
      id: `user-${Date.now()}`,
      room_id: activeRoom.id,
      role: 'user',
      content: userText,
      created_at: createLocalTimestamp(),
    }]);
    setIsTyping(true);

    try {
      const reply = await chatClient.sendMessage(activeRoom.id, userText, activeSpiritId);
      setMessages((prev) => [...prev, reply]);
    } catch (err) {
      console.error('채팅 응답 수집 실패:', err);
      setMessages((prev) => [...prev, {
        id: `err-${Date.now()}`,
        room_id: activeRoom.id,
        role: 'assistant',
        content: '로컬 AI 엔진에서 응답을 수집하지 못했습니다. 모델 파일과 엔진 상태를 확인하십시오.',
        created_at: createLocalTimestamp(),
      }]);
    } finally {
      setIsTyping(false);
    }
  }

  async function syncStyles() {
    setIsSyncing(true);
    try {
      await syncClient.runSync();
      await refreshStyles();
    } catch (err) {
      console.error('서버 동기화 실패:', err);
    } finally {
      setIsSyncing(false);
    }
  }

  async function selectStyle(styleId: string) {
    try {
      const updated = await styleClient.selectActive(styleId);
      setActiveStyle(updated);
      setStyles((prev) => prev.map((style) => ({ ...style, is_active: style.id === styleId })));
    } catch (err) {
      console.error('스타일 활성화 실패:', err);
    }
  }

  useEffect(() => {
    async function initApp() {
      try {
        try {
          const session = await authClient.getSession();
          if (!session) {
            await authClient.login('garnet@everlib.pro', 'ever_token_secret');
          }
        } catch (err) {
          console.error('원격 인증 세션 수립 실패:', err);
        }

        const dbList = await personaClient.list();
        const sortedList = dbList.sort((a, b) => a.name.localeCompare(b.name));
        setSpirits(sortedList);

        const defaultSpirit = sortedList.find((persona) => persona.id === 'garnetrapture') ?? sortedList[0];
        if (defaultSpirit) {
          await selectSpirit(defaultSpirit);
        }

        await refreshStyles();
        void refreshLlmStatus();
      } catch (err) {
        console.error('앱 초기화 중 오류 발생:', err);
      } finally {
        setAppInitializing(false);
      }
    }

    initApp();
  }, []);

  useEffect(() => {
    messageEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  return {
    appInitializing,
    llmStatus,
    filteredSpirits,
    searchQuery,
    activeSpiritId,
    activeDetail,
    activeRoom,
    messages,
    inputText,
    isTyping,
    styles,
    activeStyle,
    isSyncing,
    messageEndRef,
    setSearchQuery,
    setInputText,
    selectSpirit,
    sendMessage,
    syncStyles,
    selectStyle,
  };
}
