import React, { useEffect, useMemo, useRef, useState } from 'react';
import { authClient } from '../auth';
import { chatClient, type ChatMessage, type ChatRoom } from '../chat';
import { llmClient, type LlmStatus } from '../llm';
import { parseSpiritDetail, personaClient, type BondRankingEntry, type PersonaConfig, type SpiritDetail } from '../persona';
import { settingsClient, type AppSettings, type ResetSummary } from '../settings';
import { styleClient, type StyleProfile } from '../style';
import { syncClient } from '../sync';
import { trainingClient, type TrainingSummary } from '../training';
import { createApiStatus, createRoomTitle, filterSpirits, formatUnknownError, } from './logic';
import type { ApiStatusItem, EverTalkController, RosterTab, StageTab } from './types';
export function useFirstLoadableImage(candidates: string[]): [
    string | null,
    () => void
] {
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
    const [defaultPersonaId, setDefaultPersonaId] = useState<string | null>(null);
    const [personaLoadError, setPersonaLoadError] = useState<string | null>(null);
    const [systemStatuses, setSystemStatuses] = useState<ApiStatusItem[]>([
        createApiStatus('auth', '웹 인증 세션', 'checking', '확인 중'),
        createApiStatus('persona-archive', '페르소나 팩', 'checking', '확인 중'),
        createApiStatus('persona-db', '정령 DB', 'checking', '확인 중'),
        createApiStatus('chat-db', '채팅 DB', 'checking', '확인 중'),
        createApiStatus('style-db', '스타일 DB', 'checking', '확인 중'),
        createApiStatus('llm', '로컬 GGUF 모델', 'checking', '확인 중'),
        createApiStatus('sync', '데이터팩 동기화', 'warning', '수동 동기화 대기'),
    ]);
    const [searchQuery, setSearchQuery] = useState('');
    const [activeRosterTab, setActiveRosterTab] = useState<RosterTab>('list');
    const [activeStageTab, setActiveStageTab] = useState<StageTab>('chat');
    const [profileCollapsed, setProfileCollapsed] = useState(false);
    const [rosterCollapsed, setRosterCollapsed] = useState(false);
    const [activeSpiritId, setActiveSpiritId] = useState('');
    const [activeDetail, setActiveDetail] = useState<SpiritDetail | null>(null);
    const [activeRoom, setActiveRoom] = useState<ChatRoom | null>(null);
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [inputText, setInputText] = useState('');
    const [isTyping, setIsTyping] = useState(false);
    const [styles, setStyles] = useState<StyleProfile[]>([]);
    const [activeStyle, setActiveStyle] = useState<StyleProfile | null>(null);
    const [isSyncing, setIsSyncing] = useState(false);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [backgroundGalleryOpen, setBackgroundGalleryOpen] = useState(false);
    const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
    const [isResetting, setIsResetting] = useState(false);
    const [resetSummary, setResetSummary] = useState<ResetSummary | null>(null);
    const [resetError, setResetError] = useState<string | null>(null);
    const [isTraining, setIsTraining] = useState(false);
    const [trainingSummary, setTrainingSummary] = useState<TrainingSummary | null>(null);
    const [trainingError, setTrainingError] = useState<string | null>(null);
    const [bondRanking, setBondRanking] = useState<BondRankingEntry[]>([]);
    const [bondRankingLoading, setBondRankingLoading] = useState(false);
    const messagesListRef = useRef<HTMLDivElement>(null);
    const appInitStartedRef = useRef(false);
    const filteredSpirits = useMemo(() => filterSpirits(spirits, searchQuery), [searchQuery, spirits]);
    function setSystemStatus(status: ApiStatusItem) {
        setSystemStatuses((prev) => prev.map((item) => (item.id === status.id ? status : item)));
    }
    async function refreshStyles() {
        try {
            const [styleList, active] = await Promise.all([
                styleClient.list(),
                styleClient.getActive(),
            ]);
            setStyles(styleList);
            setActiveStyle(active);
            setSystemStatus(createApiStatus('style-db', '스타일 DB', 'ready', `${styleList.length}개 로드`));
        }
        catch (err) {
            console.error('스타일팩 DB 로드 실패:', err);
            setSystemStatus(createApiStatus('style-db', '스타일 DB', 'error', formatUnknownError(err)));
        }
    }
    async function refreshLlmStatus() {
        try {
            let status = await llmClient.getStatus();
            if (!status.is_loaded) {
                status = await llmClient.loadEngine();
            }
            setLlmStatus(status);
            setSystemStatus(createApiStatus('llm', '로컬 GGUF 모델', status.is_loaded ? 'ready' : 'warning', status.is_loaded ? 'ai/model GGUF 로드됨' : '모델 미로드'));
        }
        catch (err) {
            console.error('로컬 LLM 엔진 로드 실패:', err);
            const message = formatUnknownError(err);
            setLlmStatus({ is_loaded: false, model_path: null, error_message: message });
            setSystemStatus(createApiStatus('llm', '로컬 GGUF 모델', 'error', message));
        }
    }
    async function selectSpirit(spirit: PersonaConfig) {
        setActiveSpiritId(spirit.id);
        const detail = parseSpiritDetail(spirit);
        setActiveDetail(detail);
        const roomTitle = createRoomTitle(spirit.name);
        let room = await chatClient.getLatestSessionRoom(spirit.id);
        if (!room) {
            room = await chatClient.createSessionRoom(roomTitle, spirit.id);
        }
        setActiveRoom(room);
        const history = await chatClient.listMessages(room.id);
        setMessages(history);
    }
    async function setDefaultSpirit(spiritId: string) {
        try {
            const updated = await personaClient.setDefault(spiritId);
            setDefaultPersonaId(updated);
            setSystemStatus(createApiStatus('persona-db', '정령 DB', 'ready', `기본 프로필 ${updated}`));
        }
        catch (err) {
            setSystemStatus(createApiStatus('persona-db', '정령 DB', 'error', formatUnknownError(err)));
        }
    }
    async function sendMessage(event: React.FormEvent) {
        event.preventDefault();
        if (!inputText.trim() || !activeRoom || !activeDetail || isTyping) {
            return;
        }
        const userText = inputText;
        const room = activeRoom;
        setInputText('');
        setIsTyping(true);
        const optimisticMessage: ChatMessage = {
            id: `pending-${room.id}-${messages.length}`,
            room_id: room.id,
            role: 'user',
            content: userText,
            created_at: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, optimisticMessage]);
        try {
            await chatClient.sendMessage(room.id, userText, activeSpiritId);
            const savedMessages = await chatClient.listMessages(room.id);
            setMessages(savedMessages);
        }
        catch (err) {
            console.error('채팅 응답 수집 실패:', err);
            setSystemStatus(createApiStatus('llm', '로컬 GGUF 모델', 'error', formatUnknownError(err)));
        }
        finally {
            setIsTyping(false);
        }
    }
    async function syncStyles() {
        setIsSyncing(true);
        try {
            await syncClient.runSync();
            await refreshStyles();
        }
        catch (err) {
            console.error('서버 동기화 실패:', err);
            setSystemStatus(createApiStatus('sync', '데이터팩 동기화', 'error', formatUnknownError(err)));
        }
        finally {
            setIsSyncing(false);
        }
    }
    async function selectStyle(styleId: string) {
        try {
            const updated = await styleClient.selectActive(styleId);
            setActiveStyle(updated);
            setStyles((prev) => prev.map((style) => ({ ...style, is_active: style.id === styleId })));
        }
        catch (err) {
            console.error('스타일 활성화 실패:', err);
        }
    }
    async function openSettings() {
        setSettingsOpen(true);
        setResetSummary(null);
        setResetError(null);
        try {
            const current = await settingsClient.get();
            setAppSettings(current);
        }
        catch (err) {
            console.error('설정 조회 실패:', err);
            setResetError(formatUnknownError(err));
        }
    }
    function closeSettings() {
        setSettingsOpen(false);
    }
    function openBackgroundGallery() {
        setBackgroundGalleryOpen(true);
    }
    function closeBackgroundGallery() {
        setBackgroundGalleryOpen(false);
    }
    async function resetAppData() {
        setIsResetting(true);
        setResetError(null);
        try {
            const summary = await settingsClient.reset();
            setResetSummary(summary);
            setAppSettings({ default_persona_id: null, active_style_id: null });
            setActiveSpiritId('');
            setActiveDetail(null);
            setActiveRoom(null);
            setMessages([]);
            setDefaultPersonaId(null);
            setActiveStyle(null);
            setStyles([]);
            const dbList = await personaClient.list();
            const sortedList = dbList.sort((a, b) => a.name.localeCompare(b.name));
            setSpirits(sortedList);
            setSystemStatus(createApiStatus('persona-db', '정령 DB', sortedList.length > 0 ? 'ready' : 'warning', sortedList.length > 0 ? `${sortedList.length}개 로드` : 'DB 목록 없음'));
            const defaultSpirit = sortedList.find((persona) => persona.id === 'garnetrapture') ?? sortedList[0];
            if (defaultSpirit) {
                await selectSpirit(defaultSpirit);
            }
            const rooms = await chatClient.listRooms();
            setSystemStatus(createApiStatus('chat-db', '채팅 DB', 'ready', `${rooms.length}개 대화방`));
            setSystemStatus(createApiStatus('style-db', '스타일 DB', 'warning', '0개 로드'));
        }
        catch (err) {
            console.error('설정 초기화 실패:', err);
            setResetError(formatUnknownError(err));
        }
        finally {
            setIsResetting(false);
        }
    }
    async function trainPersona() {
        if (!activeSpiritId || isTraining) {
            return;
        }
        setIsTraining(true);
        setTrainingError(null);
        try {
            const summary = await trainingClient.run(activeSpiritId);
            setTrainingSummary(summary);
        }
        catch (err) {
            console.error('정령 LoRA 학습 실패:', err);
            setTrainingError(formatUnknownError(err));
        }
        finally {
            setIsTraining(false);
        }
    }
    useEffect(() => {
        if (appInitStartedRef.current) {
            return;
        }
        appInitStartedRef.current = true;
        async function initApp() {
            try {
                try {
                    const session = await authClient.getSession();
                    setSystemStatus(createApiStatus('auth', '웹 인증 세션', session ? 'ready' : 'warning', session ? '세션 확인됨' : '로컬 세션 없음'));
                }
                catch (err) {
                    console.error('원격 인증 세션 수립 실패:', err);
                    setSystemStatus(createApiStatus('auth', '웹 인증 세션', 'error', formatUnknownError(err)));
                }
                try {
                    const archiveList = await personaClient.listArchive();
                    setSystemStatus(createApiStatus('persona-archive', '페르소나 팩', 'ready', `${archiveList.length}개 확인`));
                }
                catch (err) {
                    setSystemStatus(createApiStatus('persona-archive', '페르소나 팩', 'error', formatUnknownError(err)));
                }
                let dbList: PersonaConfig[] = [];
                let savedDefaultId: string | null = null;
                try {
                    dbList = await personaClient.list();
                    savedDefaultId = await personaClient.getDefault();
                    setDefaultPersonaId(savedDefaultId);
                    setPersonaLoadError(null);
                    setSystemStatus(createApiStatus('persona-db', '정령 DB', dbList.length > 0 ? 'ready' : 'warning', dbList.length > 0 ? `${dbList.length}개 로드` : 'DB 목록 없음'));
                }
                catch (err) {
                    const message = formatUnknownError(err);
                    setPersonaLoadError(message);
                    setSystemStatus(createApiStatus('persona-db', '정령 DB', 'error', message));
                }
                try {
                    const rooms = await chatClient.listRooms();
                    setSystemStatus(createApiStatus('chat-db', '채팅 DB', 'ready', `${rooms.length}개 대화방`));
                }
                catch (err) {
                    setSystemStatus(createApiStatus('chat-db', '채팅 DB', 'error', formatUnknownError(err)));
                }
                const sortedList = dbList.sort((a, b) => a.name.localeCompare(b.name));
                setSpirits(sortedList);
                const savedDefault = savedDefaultId
                    ? sortedList.find((persona) => persona.id === savedDefaultId)
                    : null;
                const defaultSpirit = savedDefault
                    ?? sortedList.find((persona) => persona.id === 'garnetrapture')
                    ?? sortedList[0];
                if (defaultSpirit) {
                    await selectSpirit(defaultSpirit);
                }
                await refreshStyles();
                void refreshLlmStatus();
            }
            catch (err) {
                console.error('앱 초기화 중 오류 발생:', err);
                setPersonaLoadError(formatUnknownError(err));
            }
            finally {
                setAppInitializing(false);
            }
        }
        initApp();
    }, []);
    useEffect(() => {
        const listEl = messagesListRef.current;
        if (!listEl) {
            return;
        }
        listEl.scrollTop = listEl.scrollHeight;
    }, [messages]);
    useEffect(() => {
        if (activeRosterTab !== 'bondRanking') {
            return;
        }
        let cancelled = false;
        setBondRankingLoading(true);
        personaClient
            .getBondRanking()
            .then((ranking) => {
            if (!cancelled) {
                setBondRanking(ranking);
            }
        })
            .catch((err) => {
            console.error('인연도 랭킹 조회 실패:', err);
        })
            .finally(() => {
            if (!cancelled) {
                setBondRankingLoading(false);
            }
        });
        return () => {
            cancelled = true;
        };
    }, [activeRosterTab]);
    return {
        appInitializing,
        llmStatus,
        filteredSpirits,
        searchQuery,
        defaultPersonaId,
        personaLoadError,
        systemStatuses,
        activeRosterTab,
        activeStageTab,
        profileCollapsed,
        rosterCollapsed,
        activeSpiritId,
        activeDetail,
        activeRoom,
        messages,
        inputText,
        isTyping,
        styles,
        activeStyle,
        isSyncing,
        messagesListRef,
        settingsOpen,
        backgroundGalleryOpen,
        appSettings,
        isResetting,
        resetSummary,
        resetError,
        isTraining,
        trainingSummary,
        trainingError,
        bondRanking,
        bondRankingLoading,
        setSearchQuery,
        setInputText,
        setActiveRosterTab,
        setActiveStageTab,
        setProfileCollapsed,
        setRosterCollapsed,
        selectSpirit,
        setDefaultSpirit,
        sendMessage,
        syncStyles,
        selectStyle,
        openSettings,
        closeSettings,
        openBackgroundGallery,
        closeBackgroundGallery,
        resetAppData,
        trainPersona,
    };
}
