import React, { useEffect, useMemo, useRef, useState } from 'react';
import type { AppLanguage } from '../../shared/types';
import { authClient } from '../auth';
import { chatClient, type ChatMessage, type ChatRoom } from '../chat';
import { llmClient, type LlmStatus } from '../llm';
import { parseSpiritDetail, personaClient, type BondRankingEntry, type FamiliarityEntry, type PersonaConfig, type SpiritDetail } from '../persona';
import { settingsClient, type AppSettings, type ResetSummary } from '../settings';
import { styleClient, type StyleProfile } from '../style';
import { syncClient, type LocalStatusSnapshot } from '../sync';
import { trainingClient, type TrainingSummary } from '../training';
import { createApiStatus, createRoomTitle, filterSpirits, formatUnknownError, } from './logic';
import { getEverTalkLabels } from './i18n';
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
    const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
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
    }
    async function refreshLocalStatus() {
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
    }
    async function selectSpirit(spirit: PersonaConfig, languageOverride?: AppLanguage) {
        setActiveSpiritId(spirit.id);
        const detail = parseSpiritDetail(spirit, languageOverride ?? appLanguage);
        setActiveDetail(detail);
        const roomTitle = createRoomTitle(detail.name);
        let room = await chatClient.getLatestSessionRoom(spirit.id);
        if (!room) {
            room = await chatClient.createSessionRoom(roomTitle, spirit.id);
        }
        setActiveRoom(room);
        const history = await chatClient.listMessages(room.id);
        setMessages(history);
        await refreshLocalStatus();
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
        try {
            await chatClient.sendMessage(room.id, userText, activeSpiritId);
            const savedMessages = await chatClient.listMessages(room.id);
            setMessages(savedMessages);
            await refreshLocalStatus();
            if (activeRosterTab === 'bondRanking') {
                setBondRanking(await personaClient.getBondRanking());
            }
            if (activeRosterTab === 'familiarity') {
                setFamiliarityList(await personaClient.getFamiliarityList());
            }
        }
        catch (err) {
            console.error('채팅 응답 수집 실패:', err);
            setSystemStatus(createApiStatus('llm', labels.localModel, 'error', formatUnknownError(err)));
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
            setAppSettings({ default_persona_id: null, active_style_id: null, language: 'ko', language_configured: false });
            setAppLanguage('ko');
            setLanguageGateOpen(true);
            setActiveSpiritId('');
            setActiveDetail(null);
            setActiveRoom(null);
            setMessages([]);
            setDefaultPersonaId(null);
            setActiveStyle(null);
            setStyles([]);
            const resetLabels = getEverTalkLabels('ko');
            const dbList = await personaClient.list();
            const sortedList = dbList.sort((a, b) => a.name.localeCompare(b.name));
            setSpirits(sortedList);
            setSystemStatus(createApiStatus('persona-db', resetLabels.personaDb, sortedList.length > 0 ? 'ready' : 'warning', sortedList.length > 0 ? resetLabels.loadedCount(sortedList.length) : resetLabels.noDbRows));
            const defaultSpirit = sortedList.find((persona) => persona.id === 'garnetrapture') ?? sortedList[0];
            if (defaultSpirit) {
                await selectSpirit(defaultSpirit, 'ko');
            }
            const rooms = await chatClient.listRooms();
            setSystemStatus(createApiStatus('chat-db', resetLabels.chatDb, 'ready', resetLabels.roomCount(rooms.length)));
            setSystemStatus(createApiStatus('style-db', resetLabels.styleDb, 'warning', resetLabels.loadedCount(0)));
            await refreshLocalStatus();
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
        if (appInitStartedRef.current) {
            return;
        }
        appInitStartedRef.current = true;
        async function initApp() {
            try {
                let dbList: PersonaConfig[] = [];
                let savedDefaultId: string | null = null;
                let initialLanguage: AppLanguage = 'ko';
                try {
                    const currentSettings = await settingsClient.get();
                    initialLanguage = currentSettings.language;
                    setAppSettings(currentSettings);
                    setAppLanguage(currentSettings.language);
                    setLanguageGateOpen(!currentSettings.language_configured);
                }
                catch (err) {
                    console.error('설정 조회 실패:', err);
                }
                const initLabels = getEverTalkLabels(initialLanguage);
                try {
                    const session = await authClient.getSession();
                    setSystemStatus(createApiStatus('auth', initLabels.authSession, session ? 'ready' : 'warning', session ? initLabels.sessionReady : initLabels.noLocalSession));
                }
                catch (err) {
                    console.error('원격 인증 세션 수립 실패:', err);
                    setSystemStatus(createApiStatus('auth', initLabels.authSession, 'error', formatUnknownError(err)));
                }
                try {
                    const archiveList = await personaClient.listArchive();
                    setSystemStatus(createApiStatus('persona-archive', initLabels.personaPack, 'ready', initLabels.archiveCount(archiveList.length)));
                }
                catch (err) {
                    setSystemStatus(createApiStatus('persona-archive', initLabels.personaPack, 'error', formatUnknownError(err)));
                }
                try {
                    dbList = await personaClient.list();
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
                    const rooms = await chatClient.listRooms();
                    setSystemStatus(createApiStatus('chat-db', initLabels.chatDb, 'ready', initLabels.roomCount(rooms.length)));
                }
                catch (err) {
                    setSystemStatus(createApiStatus('chat-db', initLabels.chatDb, 'error', formatUnknownError(err)));
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
                    await selectSpirit(defaultSpirit, initialLanguage);
                }
                await refreshStyles();
                await refreshLocalStatus();
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
    };
}
