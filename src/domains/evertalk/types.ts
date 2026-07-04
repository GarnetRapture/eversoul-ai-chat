import type React from 'react';
import type { ChatMessage, ChatRoom } from '../chat';
import type { LlmStatus } from '../llm';
import type { PersonaConfig, SpiritDetail } from '../persona';
import type { AppSettings, ResetSummary } from '../settings';
import type { StyleProfile } from '../style';
import type { TrainingSummary } from '../training';
export interface LoadableAssetImageProps {
    candidates: string[];
    alt: string;
    className?: string;
    fallback: React.ReactNode;
}
export interface SpiritRosterMeta {
    preview: string;
}
export interface TalkChoice {
    id: string;
    label: string;
    source: string;
}
export type ApiConnectionState = 'checking' | 'ready' | 'warning' | 'error';
export type RosterTab = 'list' | 'bondRanking' | 'familiarity';
export type StageTab = 'chat' | 'gallery' | 'backgrounds';
export interface ApiStatusItem {
    id: string;
    label: string;
    state: ApiConnectionState;
    detail: string;
}
export interface SpiritRosterProps {
    spirits: PersonaConfig[];
    activeSpiritId: string;
    defaultPersonaId: string | null;
    searchQuery: string;
    loadError: string | null;
    activeTab: RosterTab;
    collapsed: boolean;
    onSearchChange: (value: string) => void;
    onSelect: (spirit: PersonaConfig) => void;
    onSetDefault: (spiritId: string) => void;
    onTabChange: (tab: RosterTab) => void;
    onToggleCollapsed: () => void;
}
export interface ChatStageProps {
    activeDetail: SpiritDetail | null;
    activeStageTab: StageTab;
    activeRoom: ChatRoom | null;
    llmStatus: LlmStatus | null;
    messages: ChatMessage[];
    inputText: string;
    isTyping: boolean;
    onInputChange: (value: string) => void;
    onSendMessage: (event: React.FormEvent) => void;
    onStageTabChange: (tab: StageTab) => void;
    messageEndRef: React.RefObject<HTMLDivElement | null>;
}
export interface SpiritProfilePanelProps {
    activeDetail: SpiritDetail | null;
    collapsed: boolean;
    systemStatuses: ApiStatusItem[];
    styles: StyleProfile[];
    activeStyle: StyleProfile | null;
    isSyncing: boolean;
    onSyncStyles: () => void;
    onSelectStyle: (styleId: string) => void;
    onToggleCollapsed: () => void;
    onOpenSettings: () => void;
    isTraining: boolean;
    trainingSummary: TrainingSummary | null;
    trainingError: string | null;
    onTrainPersona: () => void;
}
export interface SystemStatusPanelProps {
    statuses: ApiStatusItem[];
}
export interface SettingsPanelProps {
    open: boolean;
    settings: AppSettings | null;
    isResetting: boolean;
    resetSummary: ResetSummary | null;
    resetError: string | null;
    onClose: () => void;
    onReset: () => void;
}
export interface EverTalkController {
    appInitializing: boolean;
    llmStatus: LlmStatus | null;
    filteredSpirits: PersonaConfig[];
    searchQuery: string;
    defaultPersonaId: string | null;
    personaLoadError: string | null;
    systemStatuses: ApiStatusItem[];
    activeRosterTab: RosterTab;
    activeStageTab: StageTab;
    profileCollapsed: boolean;
    rosterCollapsed: boolean;
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
    settingsOpen: boolean;
    appSettings: AppSettings | null;
    isResetting: boolean;
    resetSummary: ResetSummary | null;
    resetError: string | null;
    isTraining: boolean;
    trainingSummary: TrainingSummary | null;
    trainingError: string | null;
    setSearchQuery: (value: string) => void;
    setInputText: (value: string) => void;
    setActiveRosterTab: (tab: RosterTab) => void;
    setActiveStageTab: (tab: StageTab) => void;
    setProfileCollapsed: (collapsed: boolean) => void;
    setRosterCollapsed: (collapsed: boolean) => void;
    selectSpirit: (spirit: PersonaConfig) => Promise<void>;
    setDefaultSpirit: (spiritId: string) => Promise<void>;
    sendMessage: (event: React.FormEvent) => Promise<void>;
    syncStyles: () => Promise<void>;
    selectStyle: (styleId: string) => Promise<void>;
    openSettings: () => Promise<void>;
    closeSettings: () => void;
    resetAppData: () => Promise<void>;
    trainPersona: () => Promise<void>;
}
