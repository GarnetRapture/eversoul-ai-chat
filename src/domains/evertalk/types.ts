import type React from 'react';

import type { ChatMessage, ChatRoom } from '../chat';
import type { LlmStatus } from '../llm';
import type { PersonaConfig, SpiritDetail } from '../persona';
import type { StyleProfile } from '../style';

export interface LoadableAssetImageProps {
  candidates: string[];
  alt: string;
  className?: string;
  fallback: React.ReactNode;
}

export interface SpiritRosterMeta {
  preview: string;
  unreadCount: number;
  isNew: boolean;
}

export interface TalkChoice {
  id: string;
  label: string;
  detail: string;
  reward: string;
  rarity: 'normal' | 'rare';
}

export interface BondProgress {
  level: number;
  current: number;
  max: number;
  percent: number;
  meterClass: string;
}

export interface SpiritRosterProps {
  spirits: PersonaConfig[];
  activeSpiritId: string;
  searchQuery: string;
  onSearchChange: (value: string) => void;
  onSelect: (spirit: PersonaConfig) => void;
}

export interface ChatStageProps {
  activeDetail: SpiritDetail | null;
  activeRoom: ChatRoom | null;
  llmStatus: LlmStatus | null;
  messages: ChatMessage[];
  inputText: string;
  isTyping: boolean;
  onInputChange: (value: string) => void;
  onSendMessage: (event: React.FormEvent) => void;
  messageEndRef: React.RefObject<HTMLDivElement | null>;
}

export interface SpiritProfilePanelProps {
  activeDetail: SpiritDetail | null;
  styles: StyleProfile[];
  activeStyle: StyleProfile | null;
  isSyncing: boolean;
  onSyncStyles: () => void;
  onSelectStyle: (styleId: string) => void;
}

export interface EverTalkController {
  appInitializing: boolean;
  llmStatus: LlmStatus | null;
  filteredSpirits: PersonaConfig[];
  searchQuery: string;
  activeSpiritId: string;
  activeDetail: SpiritDetail | null;
  activeRoom: ChatRoom | null;
  messages: ChatMessage[];
  inputText: string;
  isTyping: boolean;
  styles: StyleProfile[];
  activeStyle: StyleProfile | null;
  isSyncing: boolean;
  messageEndRef: React.RefObject<HTMLDivElement | null>;
  setSearchQuery: (value: string) => void;
  setInputText: (value: string) => void;
  selectSpirit: (spirit: PersonaConfig) => Promise<void>;
  sendMessage: (event: React.FormEvent) => Promise<void>;
  syncStyles: () => Promise<void>;
  selectStyle: (styleId: string) => Promise<void>;
}
