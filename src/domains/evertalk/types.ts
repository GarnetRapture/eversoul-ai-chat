import type React from 'react';
import type { AppLanguage, PerformanceTier } from '../../shared/types';
import type { ChatMessage, ChatRoom } from '../chat';
import type { LlmModelValidation, LlmRequestStatus, LlmSessionStatus, LlmStatus } from '../llm';
import type { BondRankingEntry, FamiliarityEntry, PersonaConfig, SpiritDetail } from '../persona';
import type { AppSettings, HardwareProfile, ResetSummary, SetupProgress } from '../settings';
import type { StyleProfile } from '../style';
import type { LocalStatusSnapshot } from '../sync';
import type { TrainingSummary } from '../training';
import type { EverTalkLabels } from './i18n';
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
export type StageTab = 'chat' | 'gallery';
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
    bondRanking: BondRankingEntry[];
    bondRankingLoading: boolean;
    familiarityList: FamiliarityEntry[];
    familiarityLoading: boolean;
    labels: EverTalkLabels;
    appLanguage: AppLanguage;
    activeSessionIds: string[];
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
    messagesListRef: React.RefObject<HTMLDivElement | null>;
    labels: EverTalkLabels;
    onOpenProfileDetail: () => void;
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
    onOpenBackgroundGallery: () => void;
    isTraining: boolean;
    trainingSummary: TrainingSummary | null;
    trainingError: string | null;
    localStatus: LocalStatusSnapshot | null;
    labels: EverTalkLabels;
    onTrainPersona: () => void;
    onOpenProfileDetail: () => void;
}
export interface SystemStatusPanelProps {
    statuses: ApiStatusItem[];
    labels: EverTalkLabels;
}
export interface SettingsPanelProps {
    open: boolean;
    settings: AppSettings | null;
    modelValidation: LlmModelValidation | null;
    llmSessionStatuses: LlmSessionStatus[];
    llmRequestStatuses: LlmRequestStatus[];
    isResetting: boolean;
    resetSummary: ResetSummary | null;
    resetError: string | null;
    labels: EverTalkLabels;
    onClose: () => void;
    onReset: () => void;
    onSetLanguage: (language: AppLanguage) => Promise<void>;
}
export interface BackgroundGalleryPanelProps {
    open: boolean;
    labels: EverTalkLabels;
    onClose: () => void;
}
export interface LanguageGatePanelProps {
    open: boolean;
    language: AppLanguage;
    labels: EverTalkLabels;
    onSelectLanguage: (language: AppLanguage) => Promise<void>;
}
export interface PerformanceGatePanelProps {
    open: boolean;
    tier: PerformanceTier;
    hardwareProfile: HardwareProfile | null;
    labels: EverTalkLabels;
    onSelectTier: (tier: PerformanceTier) => Promise<void>;
}
export interface ProfileDetailPanelProps {
    open: boolean;
    activeDetail: SpiritDetail | null;
    labels: EverTalkLabels;
    onClose: () => void;
}
export interface SetupProgressPanelProps {
    open: boolean;
    progress: SetupProgress | null;
    labels: EverTalkLabels;
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
    messagesListRef: React.RefObject<HTMLDivElement | null>;
    settingsOpen: boolean;
    backgroundGalleryOpen: boolean;
    appSettings: AppSettings | null;
    modelValidation: LlmModelValidation | null;
    llmSessionStatuses: LlmSessionStatus[];
    llmRequestStatuses: LlmRequestStatus[];
    isResetting: boolean;
    resetSummary: ResetSummary | null;
    resetError: string | null;
    isTraining: boolean;
    trainingSummary: TrainingSummary | null;
    trainingError: string | null;
    bondRanking: BondRankingEntry[];
    bondRankingLoading: boolean;
    familiarityList: FamiliarityEntry[];
    familiarityLoading: boolean;
    appLanguage: AppLanguage;
    labels: EverTalkLabels;
    localStatus: LocalStatusSnapshot | null;
    languageGateOpen: boolean;
    profileDetailOpen: boolean;
    performanceGateOpen: boolean;
    hardwareProfile: HardwareProfile | null;
    activeSessionIds: string[];
    setupInProgress: boolean;
    setupProgress: SetupProgress | null;
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
    openBackgroundGallery: () => void;
    closeBackgroundGallery: () => void;
    resetAppData: () => Promise<void>;
    trainPersona: () => Promise<void>;
    setLanguage: (language: AppLanguage) => Promise<void>;
    closeLanguageGate: () => void;
    openProfileDetail: () => void;
    closeProfileDetail: () => void;
    setPerformanceTier: (tier: PerformanceTier) => Promise<void>;
}
