import React, { useEffect, useMemo, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { AppLanguage, PerformanceTier } from '../../shared/types';
import { authClient } from '../auth';
import { chatClient, type ChatMessage, type ChatRoom } from '../chat';
import { llmClient, type LlmModelValidation, type LlmRequestStatus, type LlmSessionStatus, type LlmStatus, type ModelDownloadProgress, type AvailableLocalModel } from '../llm';
import { modulesClient, type ImportedModule, type ModuleControl } from '../modules';
import { parseSpiritDetail, personaClient, type BondRankingEntry, type FamiliarityEntry, type PersonaConfig, type SpiritDetail } from '../persona';
import { settingsClient, type AppSettings, type ExternalApiConfigRequest, type ExternalApiTestResult, type HardwareProfile, type ResetSummary, type SetupProgress, type SetupPhase } from '../settings';
import { styleClient, type StyleProfile } from '../style';
import { syncClient, type LocalStatusSnapshot } from '../sync';
import { trainingClient, type TrainingSummary } from '../training';
import type { ApiProvider, InferenceMode } from '../../shared/types';
// [TTS 연동 보류] 합성 음성 품질 미흡으로 TTS 연동 보류 (2026-07-07). 재개 시 주석 해제.
// import { voiceClient } from '../voice';
import { createApiStatus, filterSpirits, formatUnknownError, } from './logic';
import { getEverTalkLabels } from './i18n';
import type { ApiStatusItem, EverTalkController, RosterTab, StageTab, WarmupState, WarmProgress } from './types';
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
    const [setupInferenceMode, setSetupInferenceMode] = useState<InferenceMode>('local');
    const [setupApiProvider, setSetupApiProvider] = useState<ApiProvider | null>('openai');
    const [setupApiKey, setSetupApiKey] = useState<string | null>('');
    const [availableModels, setAvailableModels] = useState<AvailableLocalModel[]>([]);
    const [selectedLocalModel, setSelectedLocalModel] = useState<string | null>(null);
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
    const [warmupState, setWarmupState] = useState<WarmupState>({
        isActive: false,
        currentIndex: 0,
        totalPersonas: 0,
        currentPersonaName: '',
        progress: null,
    });
    const [styles, setStyles] = useState<StyleProfile[]>([]);
    const [activeStyle, setActiveStyle] = useState<StyleProfile | null>(null);
    const [isSyncing, setIsSyncing] = useState(false);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [moduleManagementOpen, setModuleManagementOpen] = useState(false);
    const [backgroundGalleryOpen, setBackgroundGalleryOpen] = useState(false);
    const [languageGateOpen, setLanguageGateOpen] = useState(false);
    const [profileDetailOpen, setProfileDetailOpen] = useState(false);
    const [performanceGateOpen, setPerformanceGateOpen] = useState(false);
    const [hardwareProfile, setHardwareProfile] = useState<HardwareProfile | null>(null);
    const [activeSessionIds, setActiveSessionIds] = useState<string[]>([]);
    const [setupInProgress, setSetupInProgress] = useState(false);
    const [setupProgress, setSetupProgress] = useState<SetupProgress | null>(null);
    const [downloadProgress, setDownloadProgress] = useState<ModelDownloadProgress | null>(null);
    const [downloadError, setDownloadError] = useState<string | null>(null);
    const [isDownloading, setIsDownloading] = useState(false);
    const pendingLanguageRef = useRef<AppLanguage | null>(null);
    const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
    const [modelValidation, setModelValidation] = useState<LlmModelValidation | null>(null);
    const [llmSessionStatuses, setLlmSessionStatuses] = useState<LlmSessionStatus[]>([]);
    const [llmRequestStatuses, setLlmRequestStatuses] = useState<LlmRequestStatus[]>([]);
    const [isResetting, setIsResetting] = useState(false);
    const [resetSummary, setResetSummary] = useState<ResetSummary | null>(null);
    const [resetError, setResetError] = useState<string | null>(null);
    const [importedModules, setImportedModules] = useState<ImportedModule[]>([]);
    const [moduleImportError, setModuleImportError] = useState<string | null>(null);
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
            const currentSettings = await settingsClient.get();
            setAppSettings(currentSettings);
            if (currentSettings.external_api.enabled) {
                setLlmStatus({ is_loaded: false, model_path: null, error_message: null });
                setSystemStatus(createApiStatus('llm', labels.localModel, 'ready', `External API: ${currentSettings.external_api.model}`));
                return;
            }
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
    async function ensureLlmReadyForPersonaCache(): Promise<boolean> {
        try {
            const currentSettings = await settingsClient.get();
            setAppSettings(currentSettings);
            if (currentSettings.external_api.enabled) {
                setSystemStatus(createApiStatus('llm', labels.localModel, 'ready', `External API: ${currentSettings.external_api.model}`));
                return false;
            }
            let status = await llmClient.getStatus();
            if (!status.is_loaded) {
                status = await llmClient.loadEngine();
            }
            setLlmStatus(status);
            setSystemStatus(createApiStatus('llm', labels.localModel, status.is_loaded ? 'ready' : 'warning', status.is_loaded ? labels.modelLoaded : labels.modelNotLoaded));
            return status.is_loaded;
        }
        catch (err) {
            console.error('정령 사전 캐시용 로컬 LLM 로드 실패:', err);
            setSystemStatus(createApiStatus('llm', labels.localModel, 'error', formatUnknownError(err)));
            return false;
        }
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

        const loadPromises = [
            authClient.getSession().then(session => {
                setSystemStatus(createApiStatus('auth', initLabels.authSession, session ? 'ready' : 'warning', session ? initLabels.sessionReady : initLabels.noLocalSession));
            }).catch(err => {
                setSystemStatus(createApiStatus('auth', initLabels.authSession, 'error', formatUnknownError(err)));
            }),

            personaClient.listArchive().then(list => {
                setSystemStatus(createApiStatus('persona-archive', initLabels.personaPack, 'ready', initLabels.archiveCount(list.length)));
            }).catch(err => {
                setSystemStatus(createApiStatus('persona-archive', initLabels.personaPack, 'error', formatUnknownError(err)));
            }),

            Promise.all([personaClient.list(), personaClient.getDefault()]).then(([list, defId]) => {
                dbList = list;
                savedDefaultId = defId;
                setDefaultPersonaId(defId);
                setPersonaLoadError(null);
                setSystemStatus(createApiStatus('persona-db', initLabels.personaDb, list.length > 0 ? 'ready' : 'warning', list.length > 0 ? initLabels.loadedCount(list.length) : initLabels.noDbRows));
            }).catch(err => {
                const message = formatUnknownError(err);
                setPersonaLoadError(message);
                setSystemStatus(createApiStatus('persona-db', initLabels.personaDb, 'error', message));
            }),

            chatClient.listRooms().then(rooms => {
                setSystemStatus(createApiStatus('chat-db', initLabels.chatDb, 'ready', initLabels.roomCount(rooms.length)));
            }).catch(err => {
                setSystemStatus(createApiStatus('chat-db', initLabels.chatDb, 'error', formatUnknownError(err)));
            }),

            llmClient.getStatus().then(status => {
                setLlmStatus(status);
                setSystemStatus(createApiStatus('llm', initLabels.localModel, status.is_loaded ? 'ready' : 'warning', status.is_loaded ? initLabels.modelLoaded : initLabels.modelNotLoaded));
            }).catch(err => {
                setSystemStatus(createApiStatus('llm', initLabels.localModel, 'error', formatUnknownError(err)));
            }),
            
            refreshStyles(),
            refreshLocalStatus()
        ];

        await Promise.all(loadPromises);

        const sortedList = [...dbList].sort((a, b) => a.name.localeCompare(b.name));
        setSpirits(sortedList);
        
        // 전체 정령 대상 초기 구축 스케줄러 (백그라운드)
        ensureLlmReadyForPersonaCache().then(async llmReady => {
            if (llmReady && sortedList.length > 0) {
                const unlistenProgress = await listen<WarmProgress>('warm-progress', (event) => {
                    setWarmupState(prev => ({ ...prev, progress: event.payload }));
                });

                setWarmupState(prev => ({
                    ...prev,
                    isActive: true,
                    totalPersonas: sortedList.length,
                }));

                for (let i = 0; i < sortedList.length; i++) {
                    const spirit = sortedList[i];
                    
                    setWarmupState(prev => ({
                        ...prev,
                        currentIndex: i,
                        currentPersonaName: spirit.name,
                        progress: null
                    }));

                    setSystemStatus(createApiStatus('llm', initLabels.localModel, 'warning', initLabels.warmupStatusBuilding(i + 1, sortedList.length)));
                    try {
                        await chatClient.preparePersonaCache(spirit.id);
                    } catch (err) {
                        console.error(`정령 사전 캐시 준비 실패 (${spirit.id}):`, err);
                    }
                }
                
                setWarmupState(prev => ({ ...prev, isActive: false }));
                unlistenProgress();
                
                setSystemStatus(createApiStatus('llm', initLabels.localModel, 'ready', initLabels.warmupStatusDone));
                await refreshActiveSessions();
            }
        });

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
            const currentSettings = await settingsClient.get();
            setAppSettings(currentSettings);
            if (currentSettings.external_api.enabled) {
                setLlmStatus({ is_loaded: false, model_path: null, error_message: null });
                setSystemStatus(createApiStatus('llm', initLabels.localModel, 'ready', `External API: ${currentSettings.external_api.model}`));
                frontendDebugLog('loadMainAppData:done');
                return;
            }
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
            const staged = await settingsClient.completeInitialSetup(language, setupInferenceMode, setupApiProvider, setupApiKey, tier);
            setAppSettings(staged);
            setAppLanguage(staged.language);
            await loadMainAppData(staged.language);
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

        async function selectRoom(room: ChatRoom) {
            if (room) {
                setActiveRoom(room);
                const currentSpiritId = spirit.id;
                if (currentSpiritId) {
                    const history = await chatClient.listMessagesForPersona(room.id, currentSpiritId);
                    setMessages(history);
                    
                    ensureLlmReadyForPersonaCache().then(async llmReady => {
                        if (llmReady) {
                            try {
                                await chatClient.preparePersonaCache(currentSpiritId);
                                await refreshActiveSessions();
                            } catch (err) {
                                console.error('채팅방 전환 중 사전 캐시 준비 실패:', err);
                            }
                        }
                    });
                }
                await refreshActiveSessions();
            }
        }

        const room = await chatClient.getEverTalkSessionRoom();
        await selectRoom(room);
        
        // 프론트엔드 UI 블로킹(프리징) 방지: 프롬프트 캐시 준비는 백그라운드에서 비동기로 실행
        ensureLlmReadyForPersonaCache().then(async llmReady => {
            if (llmReady) {
                try {
                    await chatClient.preparePersonaCache(spirit.id);
                } catch (err) {
                    console.error('정령 사전 캐시 준비 실패:', err);
                    setSystemStatus(createApiStatus('llm', labels.localModel, 'warning', formatUnknownError(err)));
                }
            }
        });

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
        setModuleImportError(null);
        try {
            const current = await settingsClient.get();
            setAppSettings(current);
            const [validation, sessionStatuses, requestStatuses, modules] = await Promise.all([
                llmClient.verifyModel(),
                llmClient.getSessionStatuses(),
                llmClient.getRequestStatuses(),
                modulesClient.list(),
            ]);
            setModelValidation(validation);
            setLlmSessionStatuses(sessionStatuses);
            setLlmRequestStatuses(requestStatuses);
            setImportedModules(modules);
        }
        catch (err) {
            console.error('설정 조회 실패:', err);
            setResetError(formatUnknownError(err));
        }
    }
    function closeSettings() {
        setSettingsOpen(false);
    }
    async function openModuleManagement() {
        setModuleManagementOpen(true);
        setModuleImportError(null);
        try {
            setImportedModules(await modulesClient.list());
        }
        catch (err) {
            setModuleImportError(formatUnknownError(err));
        }
    }
    function closeModuleManagement() {
        setModuleManagementOpen(false);
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
        const isInitialSetupFlow = (appSettings?.setup_stage ?? 'language') !== 'done';
        if (isInitialSetupFlow) {
            const language = appSettings?.language ?? appLanguage;
            await runInitialSetup(language, tier);
            return;
        }
        const updated = await settingsClient.setPerformanceTier(tier);
        setAppSettings(updated);
        setPerformanceGateOpen(false);
        if (appSettings?.inference_mode === 'local' && llmStatus?.is_loaded) {
            await llmClient.unloadEngine();
            await refreshLlmStatus();
        }
    }

    async function completeSetup() {
        if (appSettings?.setup_stage !== 'done') {
            const language = appSettings?.language ?? appLanguage;
            const defaultTier = hardwareProfile?.recommended_tier ?? 'balanced';
            await runInitialSetup(language, defaultTier);
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
                inference_mode: 'local',
                api_provider: null,
                api_key: null,
                setup_stage: 'language',
                show_reasoning: true,
                external_api: {
                    enabled: false,
                    base_url: 'https://api.openai.com/v1',
                    api_key_configured: false,
                    model: 'gpt-4o-mini',
                },
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
            setImportedModules([]);
            setModuleImportError(null);
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
        if (!activeSpiritId || isTraining) return;
        
        setIsTraining(true);
        setTrainingError(null);
        setTrainingSummary(null);
        try {
            const summary = await trainingClient.run(activeSpiritId);
            setTrainingSummary(summary);
        } catch (err) {
            console.error('정령 LoRA 학습 실패:', err);
            setTrainingError(formatUnknownError(err));
        } finally {
            setIsTraining(false);
        }
    }
    async function setLanguage(language: AppLanguage) {
        const isInitialSetupFlow = (appSettings?.setup_stage ?? 'language') !== 'done';
        if (isInitialSetupFlow) {
            setAppLanguage(language);
            await settingsClient.setLanguage(language);
            const staged = await settingsClient.setSetupStage('download');
            setAppSettings(staged);
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
    
    async function setShowReasoning(show: boolean) {
        const updated = await settingsClient.setShowReasoning(show);
        setAppSettings(updated);
    }
    
    async function setInferenceMode(mode: InferenceMode) {
        const updated = await settingsClient.setInferenceMode(mode);
        setAppSettings(updated);
    }

    async function setApiProvider(provider: ApiProvider | null) {
        const updated = await settingsClient.setApiProvider(provider);
        setAppSettings(updated);
    }

    async function setApiKey(key: string | null) {
        const updated = await settingsClient.setApiKey(key);
        setAppSettings(updated);
    }

    async function changeLocalModel(modelId: string) {
        setSelectedLocalModel(modelId);
        try {
            const updated = await settingsClient.setActiveLocalModel(modelId);
            setAppSettings(updated);
        } catch (err) {
            console.error('로컬 모델 변경 실패:', err);
        }
    }

    async function setExternalApiConfig(request: ExternalApiConfigRequest) {
        const updated = await settingsClient.setExternalApiConfig(request);
        setAppSettings(updated);
        if (updated.external_api.enabled) {
            setLlmStatus({ is_loaded: false, model_path: null, error_message: null });
            setSystemStatus(createApiStatus('llm', labels.localModel, 'ready', `External API: ${updated.external_api.model}`));
        }
        else {
            await refreshLlmStatus();
        }
    }

    async function testExternalApi(): Promise<ExternalApiTestResult> {
        return settingsClient.testExternalApi();
    }

    async function importModule(path: string) {
        setModuleImportError(null);
        try {
            await modulesClient.importFromPath(path);
            setImportedModules(await modulesClient.list());
        }
        catch (err) {
            const message = formatUnknownError(err);
            setModuleImportError(message);
            throw err;
        }
    }

    async function setModuleEnabled(id: string, enabled: boolean) {
        setModuleImportError(null);
        try {
            setImportedModules(await modulesClient.setEnabled(id, enabled));
        }
        catch (err) {
            const message = formatUnknownError(err);
            setModuleImportError(message);
            throw err;
        }
    }

    async function deleteModule(id: string) {
        setModuleImportError(null);
        try {
            setImportedModules(await modulesClient.delete(id));
        }
        catch (err) {
            const message = formatUnknownError(err);
            setModuleImportError(message);
            throw err;
        }
    }

    async function updateModuleControls(id: string, controls: ModuleControl[]) {
        setModuleImportError(null);
        try {
            setImportedModules(await modulesClient.updateControls(id, controls));
        }
        catch (err) {
            const message = formatUnknownError(err);
            setModuleImportError(message);
            throw err;
        }
    }

    async function startModelDownload() {
        if (isDownloading) {
            return;
        }
        setDownloadError(null);
        setIsDownloading(true);
        setDownloadProgress({ downloaded_bytes: 0, total_bytes: 0, ratio: 0, done: false });
        const unlisten = await listen<ModelDownloadProgress>('model_download_progress', (event) => {
            setDownloadProgress(event.payload);
        });
        try {
            await llmClient.downloadModel();
            
            const validation = await llmClient.verifyModel();
            setDownloadProgress({ 
                downloaded_bytes: validation.size_bytes, 
                total_bytes: validation.size_bytes, 
                ratio: 1.0, 
                done: true 
            });
        }
        catch (err) {
            console.error('모델 다운로드 실패:', err);
            setDownloadError(formatUnknownError(err));
        }
        finally {
            unlisten();
            setIsDownloading(false);
        }
    }
    
    async function nextSetupStage() {
        if (!appSettings) return;
        
        let nextStage: SetupPhase = 'done';
        if (appSettings.setup_stage === 'language') {
            nextStage = 'mode';
        } else if (appSettings.setup_stage === 'mode') {
            if (setupInferenceMode === 'api') {
                nextStage = 'performance';
            } else {
                nextStage = 'model_select';
            }
        } else if (appSettings.setup_stage === 'model_select') {
            const selected = availableModels.find(m => m.id === selectedLocalModel);
            if (selected && selected.is_downloaded) {
                nextStage = 'performance';
            } else {
                nextStage = 'download';
            }
        } else if (appSettings.setup_stage === 'download') {
            nextStage = 'performance';
        }
        
        try {
            const staged = await settingsClient.setSetupStage(nextStage);
            setAppSettings(staged);
        } catch (e) {
            console.error(e);
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
                needsGate = currentSettings.setup_stage !== 'done';
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
            try {
                const models = await llmClient.checkAvailableModels();
                setAvailableModels(models);
                if (models.length > 0) {
                    setSelectedLocalModel(models.find(m => m.is_downloaded)?.id || models[0].id);
                }
            } catch (err) {
                console.error('로컬 모델 상태 확인 실패:', err);
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
        moduleManagementOpen,
        backgroundGalleryOpen,
        appSettings,
        modelValidation,
        llmSessionStatuses,
        llmRequestStatuses,
        isResetting,
        resetSummary,
        resetError,
        importedModules,
        moduleImportError,
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
        warmupState,
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
        openModuleManagement,
        closeModuleManagement,
        openBackgroundGallery,
        closeBackgroundGallery,
        resetAppData,
        trainPersona,
        setLanguage,
        setShowReasoning,
        setExternalApiConfig,
        testExternalApi,
        importModule,
        setModuleEnabled,
        updateModuleControls,
        deleteModule,
        closeLanguageGate,
        openProfileDetail,
        closeProfileDetail,
        setPerformanceTier,
        setInferenceMode,
        setApiProvider,
        setApiKey,
        changeLocalModel,
        setupStage: appSettings?.setup_stage ?? 'language',
        inferenceMode: setupInferenceMode,
        apiProvider: setupApiProvider,
        apiKey: setupApiKey,
        availableModels,
        selectedLocalModel,
        setSelectedLocalModel,
        setSetupInferenceMode,
        setSetupApiProvider,
        setSetupApiKey,
        downloadProgress,
        downloadError,
        isDownloading,
        startModelDownload,
        nextSetupStage,
        completeSetup,
    };
}
