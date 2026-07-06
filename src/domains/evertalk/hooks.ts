import React, { useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { AppLanguage, PerformanceTier } from '../../shared/types';
import { authClient } from '../auth';
import { chatClient, type ChatMessage, type ChatRoom } from '../chat';
import { llmClient, type LlmModelValidation, type LlmRequestStatus, type LlmSessionStatus, type LlmStatus } from '../llm';
import { parseSpiritDetail, personaClient, type BondRankingEntry, type FamiliarityEntry, type PersonaConfig, type SpiritDetail } from '../persona';
import { settingsClient, type AppSettings, type HardwareProfile, type ResetSummary, type SetupProgress } from '../settings';
import { styleClient, type StyleProfile } from '../style';
import { syncClient, type LocalStatusSnapshot } from '../sync';
import { trainingClient, type TrainingSummary } from '../training';
import { createApiStatus, filterSpirits, formatUnknownError, } from './logic';
import { getEverTalkLabels } from './i18n';
import type { ApiStatusItem, EverTalkController, RosterTab, StageTab } from './types';
function frontendDebugLog(stage: string) {
    console.info(`[eversoul-frontend] ${stage}`);
}
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
    const [appLanguage, setAppLanguage] = useState<AppLanguage>('ko');
    const labels = useMemo(() => getEverTalkLabels(appLanguage), [appLanguage]);
    const [systemStatuses, setSystemStatuses] = useState<ApiStatusItem[]>(() => [
        createApiStatus('auth', labels.authSession, 'checking', labels.checking),
        createApiStatus('persona-archive', labels.personaPack, 'checking', labels.checking),
        createApiStatus('persona-db', labels.personaDb, 'checking', labels.checking),
        createApiStatus('chat-db', labels.chatDb, 'checking', labels.checking),
        createApiStatus('style-db', labels.styleDb, 'checking', labels.checking),
        createApiStatus('llm', labels.localModel, 'checking', labels.checking),
        createApiStatus('sync', labels.dataSync, 'warning', labels.manualSyncWaiting),
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
    const [languageGateOpen, setLanguageGateOpen] = useState(false);
    const [profileDetailOpen, setProfileDetailOpen] = useState(false);
    const [performanceGateOpen, setPerformanceGateOpen] = useState(false);
    const [hardwareProfile, setHardwareProfile] = useState<HardwareProfile | null>(null);
    const [activeSessionIds, setActiveSessionIds] = useState<string[]>([]);
    const [setupInProgress, setSetupInProgress] = useState(false);
    const [setupProgress, setSetupProgress] = useState<SetupProgress | null>(null);
    const pendingLanguageRef = useRef<AppLanguage | null>(null);
    const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
    const [modelValidation, setModelValidation] = useState<LlmModelValidation | null>(null);
    const [llmSessionStatuses, setLlmSessionStatuses] = useState<LlmSessionStatus[]>([]);
    const [llmRequestStatuses, setLlmRequestStatuses] = useState<LlmRequestStatus[]>([]);
    const [isResetting, setIsResetting] = useState(false);
    const [resetSummary, setResetSummary] = useState<ResetSummary | null>(null);
    const [resetError, setResetError] = useState<string | null>(null);
    const [isTraining, setIsTraining] = useState(false);
    const [trainingSummary, setTrainingSummary] = useState<TrainingSummary | null>(null);
    const [trainingError, setTrainingError] = useState<string | null>(null);
    const [bondRanking, setBondRanking] = useState<BondRankingEntry[]>([]);
    const [bondRankingLoading, setBondRankingLoading] = useState(false);
    const [familiarityList, setFamiliarityList] = useState<FamiliarityEntry[]>([]);
    const [familiarityLoading, setFamiliarityLoading] = useState(false);
    const [localStatus, setLocalStatus] = useState<LocalStatusSnapshot | null>(null);
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
            setSystemStatus(createApiStatus('style-db', labels.styleDb, 'ready', labels.loadedCount(styleList.length)));
        }
        catch (err) {
            console.error('스타일팩 DB 로드 실패:', err);
            setSystemStatus(createApiStatus('style-db', labels.styleDb, 'error', formatUnknownError(err)));
        }
    }
    async function refreshLlmStatus() {
        frontendDebugLog('refreshLlmStatus:start');
        try {
            let status = await llmClient.getStatus();
            if (!status.is_loaded) {
                status = await llmClient.loadEngine();
            }
            setLlmStatus(status);
            setSystemStatus(createApiStatus('llm', labels.localModel, status.is_loaded ? 'ready' : 'warning', status.is_loaded ? labels.modelLoaded : labels.modelNotLoaded));
        }
        catch (err) {
            console.error('로컬 LLM 엔진 로드 실패:', err);
            const message = formatUnknownError(err);
            setLlmStatus({ is_loaded: false, model_path: null, error_message: message });
            setSystemStatus(createApiStatus('llm', labels.localModel, 'error', message));
        }
        frontendDebugLog('refreshLlmStatus:done');
    }
    async function refreshLocalStatus() {
        frontendDebugLog('refreshLocalStatus:start');
        try {
            const snapshot = await syncClient.getLocalStatus();
            setLocalStatus(snapshot);
            setSystemStatus(createApiStatus('chat-db', labels.chatDb, 'ready', labels.roomMessageCount(snapshot.chat_room_count, snapshot.chat_message_count)));
            setSystemStatus(createApiStatus('persona-db', labels.personaDb, snapshot.persona_count > 0 ? 'ready' : 'warning', labels.loadedCount(snapshot.persona_count)));
            setSystemStatus(createApiStatus('style-db', labels.styleDb, snapshot.style_count > 0 ? 'ready' : 'warning', labels.loadedCount(snapshot.style_count)));
        }
        catch (err) {
            setSystemStatus(createApiStatus('chat-db', labels.chatDb, 'error', formatUnknownError(err)));
        }
        frontendDebugLog('refreshLocalStatus:done');
    }
    async function refreshActiveSessions() {
        frontendDebugLog('refreshActiveSessions:start');
        try {
            const [sessionIds, sessionStatuses, requestStatuses] = await Promise.all([
                llmClient.getActiveSessions(),
                llmClient.getSessionStatuses(),
                llmClient.getRequestStatuses(),
            ]);
            setActiveSessionIds(sessionIds);
            setLlmSessionStatuses(sessionStatuses);
            setLlmRequestStatuses(requestStatuses);
        }
        catch (err) {
            console.error('활성 세션 조회 실패:', err);
        }
        frontendDebugLog('refreshActiveSessions:done');
    }
    async function loadMainAppData(initialLanguage: AppLanguage) {
        frontendDebugLog('loadMainAppData:start');
        const initLabels = getEverTalkLabels(initialLanguage);
        let dbList: PersonaConfig[] = [];
        let savedDefaultId: string | null = null;
        try {
            frontendDebugLog('loadMainAppData:auth_get_session:start');
            const session = await authClient.getSession();
            setSystemStatus(createApiStatus('auth', initLabels.authSession, session ? 'ready' : 'warning', session ? initLabels.sessionReady : initLabels.noLocalSession));
        }
        catch (err) {
            console.error('원격 인증 세션 수립 실패:', err);
            setSystemStatus(createApiStatus('auth', initLabels.authSession, 'error', formatUnknownError(err)));
        }
        try {
            frontendDebugLog('loadMainAppData:persona_list_archive:start');
            const archiveList = await personaClient.listArchive();
            setSystemStatus(createApiStatus('persona-archive', initLabels.personaPack, 'ready', initLabels.archiveCount(archiveList.length)));
        }
        catch (err) {
            setSystemStatus(createApiStatus('persona-archive', initLabels.personaPack, 'error', formatUnknownError(err)));
        }
        try {
            frontendDebugLog('loadMainAppData:persona_list:start');
            dbList = await personaClient.list();
            frontendDebugLog('loadMainAppData:persona_get_default:start');
            savedDefaultId = await personaClient.getDefault();
            setDefaultPersonaId(savedDefaultId);
            setPersonaLoadError(null);
            setSystemStatus(createApiStatus('persona-db', initLabels.personaDb, dbList.length > 0 ? 'ready' : 'warning', dbList.length > 0 ? initLabels.loadedCount(dbList.length) : initLabels.noDbRows));
        }
        catch (err) {
            const message = formatUnknownError(err);
            setPersonaLoadError(message);
            setSystemStatus(createApiStatus('persona-db', initLabels.personaDb, 'error', message));
        }
        try {
            frontendDebugLog('loadMainAppData:chat_list_rooms:start');
            const rooms = await chatClient.listRooms();
            setSystemStatus(createApiStatus('chat-db', initLabels.chatDb, 'ready', initLabels.roomCount(rooms.length)));
        }
        catch (err) {
            setSystemStatus(createApiStatus('chat-db', initLabels.chatDb, 'error', formatUnknownError(err)));
        }
        const sortedList = [...dbList].sort((a, b) => a.name.localeCompare(b.name));
        setSpirits(sortedList);
        const savedDefault = savedDefaultId
            ? sortedList.find((persona) => persona.id === savedDefaultId)
            : null;
        const defaultSpirit = savedDefault
            ?? sortedList.find((persona) => persona.id === 'garnetrapture')
            ?? sortedList[0];
        if (defaultSpirit) {
            frontendDebugLog('loadMainAppData:select_default_spirit:start');
            await selectSpirit(defaultSpirit, initialLanguage);
        }
        frontendDebugLog('loadMainAppData:refreshStyles:start');
        await refreshStyles();
        frontendDebugLog('loadMainAppData:refreshLocalStatus:start');
        await refreshLocalStatus();
        try {
            frontendDebugLog('loadMainAppData:llm_status:start');
            const status = await llmClient.getStatus();
            setLlmStatus(status);
            setSystemStatus(createApiStatus('llm', initLabels.localModel, status.is_loaded ? 'ready' : 'warning', status.is_loaded ? initLabels.modelLoaded : initLabels.modelNotLoaded));
        }
        catch (err) {
            setSystemStatus(createApiStatus('llm', initLabels.localModel, 'error', formatUnknownError(err)));
        }
        frontendDebugLog('loadMainAppData:done');
    }
    async function runInitialSetup(language: AppLanguage, tier: PerformanceTier) {
        frontendDebugLog('runInitialSetup:start');
        setSetupInProgress(true);
        setSetupProgress({ stage: 'personas', current: 0, total: 1 });
        const unlisten = await listen<SetupProgress>('setup_progress', (event) => {
            setSetupProgress(event.payload);
        });
        try {
            const updated = await settingsClient.completeInitialSetup(language, tier);
            setAppSettings(updated);
            setAppLanguage(updated.language);
            await loadMainAppData(updated.language);
        }
        catch (err) {
            console.error('초기 셋업 실패:', err);
            setPersonaLoadError(formatUnknownError(err));
        }
        finally {
            unlisten();
            setSetupInProgress(false);
            setAppInitializing(false);
        }
        frontendDebugLog('runInitialSetup:done');
    }

    async function selectSpirit(spirit: PersonaConfig, languageOverride?: AppLanguage) {
        frontendDebugLog(`selectSpirit:start:${spirit.id}`);
        setActiveSpiritId(spirit.id);
        const detail = parseSpiritDetail(spirit, languageOverride ?? appLanguage);
        setActiveDetail(detail);
        const room = await chatClient.getEverTalkSessionRoom();
        setActiveRoom(room);
        const history = await chatClient.listMessagesForPersona(room.id, spirit.id);
        setMessages(history);
        await refreshLocalStatus();
        await refreshActiveSessions();
        frontendDebugLog(`selectSpirit:done:${spirit.id}`);
    }
    async function setDefaultSpirit(spiritId: string) {
        try {
            const updated = await personaClient.setDefault(spiritId);
            setDefaultPersonaId(updated);
            setSystemStatus(createApiStatus('persona-db', labels.personaDb, 'ready', labels.defaultProfileSet(updated)));
        }
        catch (err) {
            setSystemStatus(createApiStatus('persona-db', labels.personaDb, 'error', formatUnknownError(err)));
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
        const optimisticUserMessage: ChatMessage = {
            id: crypto.randomUUID(),
            room_id: room.id,
            persona_id: activeSpiritId,
            role: 'user',
            content: userText,
            created_at: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, optimisticUserMessage]);
        let aiMessage: ChatMessage | null = null;
        try {
            aiMessage = await chatClient.sendMessage(room.id, userText, activeSpiritId);
            setMessages((prev) => [...prev, aiMessage as ChatMessage]);
        }
        catch (err) {
            console.error('채팅 응답 수집 실패:', err);
            setSystemStatus(createApiStatus('llm', labels.localModel, 'error', formatUnknownError(err)));
            const errorMessage: ChatMessage = {
                id: crypto.randomUUID(),
                room_id: room.id,
                persona_id: activeSpiritId,
                role: 'system',
                content: labels.messageSendFailed,
                created_at: new Date().toISOString(),
            };
            setMessages((prev) => [...prev, errorMessage]);
        }
        if (aiMessage) {
            try {
                await refreshLocalStatus();
                await refreshActiveSessions();
                if (activeRosterTab === 'bondRanking') {
                    setBondRanking(await personaClient.getBondRanking());
                }
                if (activeRosterTab === 'familiarity') {
                    setFamiliarityList(await personaClient.getFamiliarityList());
                }
            }
            catch (err) {
                console.error('대화 후 부가 상태 갱신 실패:', err);
            }
        }
        setIsTyping(false);
    }
    async function syncStyles() {
        setIsSyncing(true);
        try {
            await syncClient.runSync();
            await refreshStyles();
            await refreshLocalStatus();
        }
        catch (err) {
            console.error('서버 동기화 실패:', err);
            setSystemStatus(createApiStatus('sync', labels.dataSync, 'error', formatUnknownError(err)));
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
            const [validation, sessionStatuses, requestStatuses] = await Promise.all([
                llmClient.verifyModel(),
                llmClient.getSessionStatuses(),
                llmClient.getRequestStatuses(),
            ]);
            setModelValidation(validation);
            setLlmSessionStatuses(sessionStatuses);
            setLlmRequestStatuses(requestStatuses);
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
    function closeLanguageGate() {
        setLanguageGateOpen(false);
    }
    async function setPerformanceTier(tier: PerformanceTier) {
        const isInitialSetupFlow = !appSettings?.performance_configured;
        if (isInitialSetupFlow) {
            setPerformanceGateOpen(false);
            const language = pendingLanguageRef.current ?? appLanguage;
            await runInitialSetup(language, tier);
            return;
        }
        const updated = await settingsClient.setPerformanceTier(tier);
        setAppSettings(updated);
        setPerformanceGateOpen(false);
        if (llmStatus?.is_loaded) {
            await llmClient.unloadEngine();
            await refreshLlmStatus();
        }
    }
    function openProfileDetail() {
        setProfileDetailOpen(true);
    }
    function closeProfileDetail() {
        setProfileDetailOpen(false);
    }
    async function resetAppData() {
        setIsResetting(true);
        setResetError(null);
        try {
            const summary = await settingsClient.reset();
            setResetSummary(summary);
            setAppSettings({
                default_persona_id: null,
                active_style_id: null,
                language: 'ko',
                language_configured: false,
                performance_tier: 'balanced',
                performance_configured: false,
            });
            setAppLanguage('ko');
            pendingLanguageRef.current = null;
            setLanguageGateOpen(true);
            setPerformanceGateOpen(true);
            setActiveSpiritId('');
            setActiveDetail(null);
            setActiveRoom(null);
            setMessages([]);
            setDefaultPersonaId(null);
            setActiveStyle(null);
            setStyles([]);
            setSpirits([]);
            setLlmStatus(null);
            setActiveSessionIds([]);
            setModelValidation(null);
            setLlmSessionStatuses([]);
            setLlmRequestStatuses([]);
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
    async function setLanguage(language: AppLanguage) {
        const isInitialSetupFlow = !appSettings?.performance_configured;
        if (isInitialSetupFlow) {
            pendingLanguageRef.current = language;
            setAppLanguage(language);
            setLanguageGateOpen(false);
            return;
        }
        const updated = await settingsClient.setLanguage(language);
        setAppSettings(updated);
        setAppLanguage(updated.language);
        setLanguageGateOpen(false);
        const activeSpirit = spirits.find((spirit) => spirit.id === activeSpiritId);
        if (activeSpirit) {
            setActiveDetail(parseSpiritDetail(activeSpirit, updated.language));
        }
    }
    useEffect(() => {
        frontendDebugLog('initEffect:entered');
        if (appInitStartedRef.current) {
            frontendDebugLog('initEffect:already_started');
            return;
        }
        appInitStartedRef.current = true;
        async function initApp() {
            frontendDebugLog('initApp:start');
            let initialLanguage: AppLanguage = 'ko';
            let needsGate = true;
            try {
                frontendDebugLog('initApp:settings_get:start');
                const currentSettings = await settingsClient.get();
                initialLanguage = currentSettings.language;
                setAppSettings(currentSettings);
                setAppLanguage(currentSettings.language);
                setLanguageGateOpen(!currentSettings.language_configured);
                setPerformanceGateOpen(!currentSettings.performance_configured);
                needsGate = !currentSettings.language_configured || !currentSettings.performance_configured;
            }
            catch (err) {
                console.error('설정 조회 실패:', err);
            }
            try {
                frontendDebugLog('initApp:detect_hardware:start');
                const detectedHardware = await settingsClient.detectHardware();
                setHardwareProfile(detectedHardware);
            }
            catch (err) {
                console.error('하드웨어 사양 감지 실패:', err);
            }
            if (needsGate) {
                frontendDebugLog('initApp:needs_gate');
                setAppInitializing(false);
                return;
            }
            try {
                frontendDebugLog('initApp:loadMainAppData:start');
                await loadMainAppData(initialLanguage);
            }
            finally {
                setAppInitializing(false);
            }
            frontendDebugLog('initApp:done');
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
    useEffect(() => {
        if (activeRosterTab !== 'familiarity') {
            return;
        }
        let cancelled = false;
        setFamiliarityLoading(true);
        personaClient
            .getFamiliarityList()
            .then((entries) => {
            if (!cancelled) {
                setFamiliarityList(entries);
            }
        })
            .catch((err) => {
            console.error('친밀도 조회 실패:', err);
        })
            .finally(() => {
            if (!cancelled) {
                setFamiliarityLoading(false);
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
        modelValidation,
        llmSessionStatuses,
        llmRequestStatuses,
        isResetting,
        resetSummary,
        resetError,
        isTraining,
        trainingSummary,
        trainingError,
        bondRanking,
        bondRankingLoading,
        familiarityList,
        familiarityLoading,
        appLanguage,
        labels,
        localStatus,
        languageGateOpen,
        profileDetailOpen,
        performanceGateOpen,
        hardwareProfile,
        activeSessionIds,
        setupInProgress,
        setupProgress,
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
        setLanguage,
        closeLanguageGate,
        openProfileDetail,
        closeProfileDetail,
        setPerformanceTier,
    };
}
