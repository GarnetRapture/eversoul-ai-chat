import type { AppLanguage } from '../../shared/types';

export interface EverTalkLabels {
    languageGateTitle: string;
    languageGateDescription: string;
    languageKo: string;
    languageEn: string;
    languageZhCn: string;
    continue: string;
    rosterTitle: string;
    rosterSubtitle: (count: number) => string;
    searchPlaceholder: string;
    list: string;
    bondRanking: string;
    familiarity: string;
    loadingFamiliarity: string;
    noFamiliarity: string;
    familiarityDescription: string;
    messages: string;
    memories: string;
    score: string;
    profileDetail: string;
    close: string;
    bondStatus: string;
    profile: string;
    grade: string;
    race: string;
    className: string;
    subClass: string;
    stat: string;
    union: string;
    constellation: string;
    birthday: string;
    height: string;
    weight: string;
    cvKo: string;
    cvJp: string;
    like: string;
    dislike: string;
    hobby: string;
    speciality: string;
    personality: string;
    dialogueExamples: string;
    localStatus: string;
    personaCount: string;
    chatRooms: string;
    chatMessages: string;
    styles: string;
    knowledge: string;
    localMemories: string;
    imageGallery: string;
    backgroundGallery: string;
    zoomImage: string;
    settings: string;
    currentSettings: string;
    defaultSpirit: string;
    activeStyle: string;
    language: string;
    displayResponseLanguage: string;
    showReasoning: string;
    resetData: string;
    resetDescription: string;
    resetComplete: string;
    resetFailed: string;
    notConfigured: string;
    resetChatRooms: (count: number) => string;
    resetMessages: (count: number) => string;
    resetPersonas: (count: number) => string;
    resetStyles: (count: number) => string;
    resetKnowledgeChunks: (count: number) => string;
    resetLocalMemories: (count: number) => string;
    resetting: string;
    resetConfirm: string;
    resetAllData: string;
    previousPage: string;
    nextPage: string;
    page: string;
    selectSpirit: string;
    modelReady: string;
    modelWaiting: string;
    spiritReaction: string;
    bondChannel: string;
    noSavedMessages: string;
    firstMessageHint: string;
    messagePlaceholder: (name: string) => string;
    modelRequiredPlaceholder: string;
    send: string;
    chat: string;
    gallery: string;
    dataLoadFailed: string;
    databasePending: string;
    personaPackChecking: string;
    loadingBondRanking: string;
    noBondData: string;
    bondDescription: string;
    setDefaultProfile: (name: string) => string;
    personaDbLoading: string;
    settingsOpen: string;
    collapseRight: string;
    expandRight: string;
    collapseLeft: string;
    expandLeft: string;
    conversationKeywords: string;
    noPersonality: string;
    assetConnection: string;
    folder: string;
    background: string;
    disconnected: string;
    loraTraining: string;
    loraTrainingDescription: string;
    trainingComplete: string;
    trainingFailed: string;
    examples: string;
    steps: string;
    finalLoss: string;
    trainingRunning: string;
    startTraining: string;
    speakingStyle: string;
    syncing: string;
    syncServerStyle: string;
    emptyProfilePanel: string;
    systemStatus: string;
    authSession: string;
    personaPack: string;
    personaDb: string;
    chatDb: string;
    styleDb: string;
    localModel: string;
    dataSync: string;
    checking: string;
    archiveCount: (count: number) => string;
    noDbRows: string;
    noLocalSession: string;
    sessionReady: string;
    loadedCount: (count: number) => string;
    roomCount: (count: number) => string;
    roomMessageCount: (rooms: number, messages: number) => string;
    manualSyncWaiting: string;
    modelLoaded: string;
    modelNotLoaded: string;
    defaultProfileSet: (id: string) => string;
    loraTrainingNote: string;
    appLoading: string;
    performanceGateTitle: string;
    performanceGateDescription: string;
    performanceTierLight: string;
    performanceTierBalanced: string;
    performanceTierPerformance: string;
    performanceTierLightDescription: string;
    performanceTierBalancedDescription: string;
    performanceTierPerformanceDescription: string;
    hardwareDetected: (cores: number, memoryGb: number) => string;
    recommendedTier: string;
    activeSessionBadge: string;
    messageSendFailed: string;
    setupProgressTitle: string;
    setupStagePersonas: string;
    setupStageCaching: string;
    setupStageModel: string;
    setupStageDone: string;
    setupProgressCount: (current: number, total: number) => string;
    setupStepIndicator: (current: number, total: number) => string;
    setupLanguageStep: string;
    setupDownloadStep: string;
    setupPerformanceStep: string;
    downloadGateTitle: string;
    downloadGateDescription: string;
    downloadStart: string;
    downloadPreparing: string;
    downloadProgressDetail: (downloadedMb: number, totalMb: number) => string;
    downloadComplete: string;
    downloadFailed: string;
    appInfoTitle: string;
    appInfoDeveloper: string;
    appInfoContact: string;
    appInfoWebsite: string;
    downloadError: string;
}

export type EverTalkLabelBag = {
    [Key in keyof EverTalkLabels]: Record<AppLanguage, EverTalkLabels[Key]>;
};

export const EVERTALK_LABELS: Record<AppLanguage, EverTalkLabels> = {
    ko: {
        languageGateTitle: '언어 선택',
        languageGateDescription: '에버톡 UI와 로컬 모델 응답에 사용할 언어를 선택하세요.',
        languageKo: '한국어',
        languageEn: 'English',
        languageZhCn: '简体中文',
        continue: '시작',
        rosterTitle: '에버톡',
        rosterSubtitle: (count) => `정령 메시지 (${count})`,
        searchPlaceholder: '정령 이름 또는 영문명',
        list: '목록',
        bondRanking: '인연도 랭킹',
        familiarity: '친밀도',
        loadingFamiliarity: '친밀도 집계 중',
        noFamiliarity: '누적된 친밀도 데이터가 없습니다',
        familiarityDescription: '정령과 대화하면 SQLite 메시지와 기억 누적량으로 친밀도가 산출됩니다.',
        messages: '메시지',
        memories: '기억',
        score: '점수',
        profileDetail: '프로필 상세',
        close: '닫기',
        bondStatus: 'BOND STATUS',
        profile: '프로필',
        grade: '등급',
        race: '종족',
        className: '클래스',
        subClass: '서브 클래스',
        stat: '능력치',
        union: '소속',
        constellation: '별자리',
        birthday: '생일',
        height: '키',
        weight: '몸무게',
        cvKo: '한국 성우',
        cvJp: '일본 성우',
        like: '좋아함',
        dislike: '싫어함',
        hobby: '취미',
        speciality: '특기',
        personality: '개성 데이터',
        dialogueExamples: '대화 예시',
        localStatus: '로컬 상태',
        personaCount: '정령',
        chatRooms: '채팅방',
        chatMessages: '메시지',
        styles: '스타일',
        knowledge: '지식',
        localMemories: '누적 기억',
        imageGallery: '이미지 갤러리',
        backgroundGallery: '배경 갤러리',
        zoomImage: '확대 보기',
        settings: '설정',
        currentSettings: '현재 설정값 (settings.ini)',
        defaultSpirit: '기본 정령',
        activeStyle: '활성 스타일',
        language: '언어',
        displayResponseLanguage: '표시 및 응답 언어',
        showReasoning: 'AI의 추론 과정 표시 (<think>)',
        resetData: '데이터 초기화',
        resetDescription: '대화 기록, 정령/스타일/지식팩 데이터, 정령별 누적 기억과 설정값을 모두 삭제해 앱을 초기 상태로 되돌립니다.',
        resetComplete: '초기화 완료',
        resetFailed: '초기화 실패',
        notConfigured: '미지정',
        resetChatRooms: (count) => `대화방 ${count}개`,
        resetMessages: (count) => `메시지 ${count}개`,
        resetPersonas: (count) => `정령 프로필 ${count}개`,
        resetStyles: (count) => `스타일 ${count}개`,
        resetKnowledgeChunks: (count) => `지식 청크 ${count}개`,
        resetLocalMemories: (count) => `누적 기억 ${count}개`,
        resetting: '초기화 중...',
        resetConfirm: '정말 초기화하시겠습니까? 다시 클릭 시 실행',
        resetAllData: '모든 데이터 초기화',
        previousPage: '이전 페이지',
        nextPage: '다음 페이지',
        page: '페이지',
        selectSpirit: '정령 선택',
        modelReady: 'CPU LLM 연결됨',
        modelWaiting: '로컬 모델 대기',
        spiritReaction: '정령 반응 보기',
        bondChannel: '인연 채널',
        noSavedMessages: '저장된 대화가 없습니다',
        firstMessageHint: '첫 메시지를 보내면 SQLite 세션에 대화가 누적됩니다.',
        messagePlaceholder: (name) => `${name}에게 메시지를 입력하세요...`,
        modelRequiredPlaceholder: '정령과 로컬 모델 연결을 확인하세요',
        send: '전송',
        chat: '대화',
        gallery: '갤러리',
        dataLoadFailed: '정령 데이터 로드 실패',
        databasePending: '정령 DB 수립 대기',
        personaPackChecking: 'persona pack과 SQLite 연결 상태를 확인 중입니다.',
        loadingBondRanking: '인연도 랭킹 조회 중',
        noBondData: '누적된 대화가 없습니다',
        bondDescription: '정령과 대화를 나누면 실제 메시지/기억 누적량을 기준으로 랭킹이 산출됩니다.',
        setDefaultProfile: (name) => `${name} 기본 프로필 지정`,
        personaDbLoading: '정령 DB 로드 대기',
        settingsOpen: '설정 열기',
        collapseRight: '우측 패널 접기',
        expandRight: '우측 패널 펼치기',
        collapseLeft: '좌측 패널 접기',
        expandLeft: '좌측 패널 펼치기',
        conversationKeywords: '대화 키워드',
        noPersonality: '등록된 소개가 없습니다.',
        assetConnection: '자산 연결',
        folder: '폴더',
        background: '배경',
        disconnected: '미연결',
        loraTraining: '정령 LoRA 학습',
        loraTrainingDescription: '이 정령이 지금까지 나눈 모든 대화로 실제 역전파 기반 LoRA 학습을 실행합니다.',
        trainingComplete: '학습 완료',
        trainingFailed: '학습 실패',
        examples: '예시',
        steps: '스텝',
        finalLoss: '최종 손실',
        trainingRunning: '학습 진행 중...',
        startTraining: '이 정령 학습 시작',
        speakingStyle: '말투 스타일',
        syncing: '동기화 중',
        syncServerStyle: '서버 스타일 동기화',
        emptyProfilePanel: '정령을 선택하면 TBL 기반 프로필과 원본 자산 연결 상태가 표시됩니다.',
        systemStatus: '로컬 실행 상태',
        authSession: '웹 인증 세션',
        personaPack: '페르소나 팩',
        personaDb: '정령 DB',
        chatDb: '채팅 DB',
        styleDb: '스타일 DB',
        localModel: '로컬 GGUF 모델',
        dataSync: '데이터팩 동기화',
        checking: '확인 중',
        archiveCount: (count) => `${count}개 확인`,
        noDbRows: 'DB 목록 없음',
        noLocalSession: '로컬 세션 없음',
        sessionReady: '세션 확인됨',
        loadedCount: (count) => `${count}개 로드`,
        roomCount: (count) => `${count}개 대화방`,
        roomMessageCount: (rooms, messages) => `${rooms}개 대화방 · ${messages}개 메시지`,
        manualSyncWaiting: '수동 동기화 대기',
        modelLoaded: 'ai/model GGUF 로드됨',
        modelNotLoaded: '모델 미로드',
        defaultProfileSet: (id) => `기본 프로필 ${id}`,
        loraTrainingNote: 'CPU에서 수 분~수 시간이 걸릴 수 있으며, 최초 실행 시 기반 모델을 내려받습니다.',
        appLoading: 'Connecting to EverTalk Local Database...',
        performanceGateTitle: '성능 등급 선택',
        performanceGateDescription: '이 PC의 사양을 감지했습니다. 응답 속도와 정확도의 균형을 선택하세요. 언제든 설정에서 변경할 수 있습니다.',
        performanceTierLight: '가벼움',
        performanceTierBalanced: '균형',
        performanceTierPerformance: '고성능',
        performanceTierLightDescription: '스레드/컨텍스트를 최소로 사용해 저사양 PC에서도 안정적으로 작동합니다.',
        performanceTierBalancedDescription: '대부분의 PC에 적합한 기본 균형 설정입니다.',
        performanceTierPerformanceDescription: '코어와 컨텍스트를 최대로 사용해 더 빠르고 긴 대화를 처리합니다.',
        hardwareDetected: (cores, memoryGb) => `감지된 사양: 물리 코어 ${cores}개 · 메모리 ${memoryGb}GB`,
        recommendedTier: '이 PC에 추천',
        activeSessionBadge: '세션 활성',
        setupProgressTitle: '에버톡 초기 구성 중',
        setupStagePersonas: '정령 데이터 준비 중',
        setupStageCaching: '선택 언어로 대화 데이터 캐싱 중',
        setupStageModel: '로컬 모델 로딩 중',
        setupStageDone: '구성 완료',
        setupProgressCount: (current, total) => `${current} / ${total}`,
        setupStepIndicator: (current, total) => `단계 ${current} / ${total}`,
        setupLanguageStep: '언어',
        setupDownloadStep: '모델 다운로드',
        setupPerformanceStep: '성능',
        downloadGateTitle: '로컬 모델 다운로드',
        downloadGateDescription: '오프라인 대화를 위한 로컬 언어 모델을 내려받습니다. 최초 1회만 필요합니다.',
        downloadStart: '다운로드 시작',
        downloadPreparing: '다운로드 준비 중',
        downloadProgressDetail: (downloadedMb, totalMb) => `${downloadedMb} / ${totalMb} MB`,
        downloadComplete: '다운로드 완료',
        downloadFailed: '다운로드에 실패했습니다. 다시 시도해 주세요.',
        appInfoTitle: '프로그램 정보',
        appInfoDeveloper: '개발자',
        appInfoContact: '문의',
        appInfoWebsite: '웹사이트',
        messageSendFailed: '응답 생성에 실패했습니다. 다시 시도해 주세요.',
        downloadError: '모델 파일 다운로드를 실패했거나 찾을 수 없습니다.',
    },
    en: {
        languageGateTitle: 'Choose Language',
        languageGateDescription: 'Select the language for EverTalk UI and local model responses.',
        languageKo: '한국어',
        languageEn: 'English',
        languageZhCn: '简体中文',
        continue: 'Start',
        rosterTitle: 'EverTalk',
        rosterSubtitle: (count) => `Soul messages (${count})`,
        searchPlaceholder: 'Soul name or English name',
        list: 'List',
        bondRanking: 'Bond Ranking',
        familiarity: 'Familiarity',
        loadingFamiliarity: 'Loading familiarity',
        noFamiliarity: 'No familiarity data yet',
        familiarityDescription: 'SQLite messages and saved memories are used to calculate familiarity.',
        messages: 'Messages',
        memories: 'Memories',
        score: 'Score',
        profileDetail: 'Profile Detail',
        close: 'Close',
        bondStatus: 'BOND STATUS',
        profile: 'Profile',
        grade: 'Grade',
        race: 'Race',
        className: 'Class',
        subClass: 'Sub Class',
        stat: 'Stat',
        union: 'Union',
        constellation: 'Constellation',
        birthday: 'Birthday',
        height: 'Height',
        weight: 'Weight',
        cvKo: 'Korean VA',
        cvJp: 'Japanese VA',
        like: 'Likes',
        dislike: 'Dislikes',
        hobby: 'Hobbies',
        speciality: 'Specialities',
        personality: 'Personality',
        dialogueExamples: 'Dialogue Examples',
        localStatus: 'Local Status',
        personaCount: 'Souls',
        chatRooms: 'Rooms',
        chatMessages: 'Messages',
        styles: 'Styles',
        knowledge: 'Knowledge',
        localMemories: 'Memories',
        imageGallery: 'Image Gallery',
        backgroundGallery: 'Background Gallery',
        zoomImage: 'Zoom',
        settings: 'Settings',
        currentSettings: 'Current settings (settings.ini)',
        defaultSpirit: 'Default Soul',
        activeStyle: 'Active Style',
        language: 'Language',
        displayResponseLanguage: 'Display and response language',
        showReasoning: 'Show AI Reasoning Process (<think>)',
        resetData: 'Reset Data',
        resetDescription: 'Deletes chat history, soul/style/knowledge data, saved memories, and settings.',
        resetComplete: 'Reset complete',
        resetFailed: 'Reset failed',
        notConfigured: 'Not set',
        resetChatRooms: (count) => `${count} rooms`,
        resetMessages: (count) => `${count} messages`,
        resetPersonas: (count) => `${count} soul profiles`,
        resetStyles: (count) => `${count} styles`,
        resetKnowledgeChunks: (count) => `${count} knowledge chunks`,
        resetLocalMemories: (count) => `${count} saved memories`,
        resetting: 'Resetting...',
        resetConfirm: 'Click again to confirm reset',
        resetAllData: 'Reset all data',
        previousPage: 'Previous page',
        nextPage: 'Next page',
        page: 'Page',
        selectSpirit: 'Select Soul',
        modelReady: 'CPU LLM connected',
        modelWaiting: 'Local model waiting',
        spiritReaction: 'Show soul reaction',
        bondChannel: 'Bond Channel',
        noSavedMessages: 'No saved messages',
        firstMessageHint: 'Send the first message to store the conversation in the SQLite session.',
        messagePlaceholder: (name) => `Message ${name}...`,
        modelRequiredPlaceholder: 'Check the selected soul and local model connection',
        send: 'Send',
        chat: 'Chat',
        gallery: 'Gallery',
        dataLoadFailed: 'Failed to load soul data',
        databasePending: 'Soul DB pending',
        personaPackChecking: 'Checking persona pack and SQLite connection.',
        loadingBondRanking: 'Loading bond ranking',
        noBondData: 'No accumulated conversations',
        bondDescription: 'Ranking is calculated from actual messages and saved memories.',
        setDefaultProfile: (name) => `Set ${name} as default profile`,
        personaDbLoading: 'Waiting for soul DB',
        settingsOpen: 'Open settings',
        collapseRight: 'Collapse right panel',
        expandRight: 'Expand right panel',
        collapseLeft: 'Collapse left panel',
        expandLeft: 'Expand left panel',
        conversationKeywords: 'Conversation Keywords',
        noPersonality: 'No description registered.',
        assetConnection: 'Asset Connection',
        folder: 'Folder',
        background: 'Background',
        disconnected: 'Disconnected',
        loraTraining: 'Soul LoRA Training',
        loraTrainingDescription: 'Run real backpropagation LoRA training from this soul’s accumulated conversations.',
        trainingComplete: 'Training complete',
        trainingFailed: 'Training failed',
        examples: 'Examples',
        steps: 'Steps',
        finalLoss: 'Final loss',
        trainingRunning: 'Training...',
        startTraining: 'Start training',
        speakingStyle: 'Speaking Style',
        syncing: 'Syncing',
        syncServerStyle: 'Sync server styles',
        emptyProfilePanel: 'Select a soul to view TBL profile and asset connection status.',
        systemStatus: 'Local Runtime Status',
        authSession: 'Web Auth Session',
        personaPack: 'Persona Pack',
        personaDb: 'Soul DB',
        chatDb: 'Chat DB',
        styleDb: 'Style DB',
        localModel: 'Local GGUF Model',
        dataSync: 'Data Pack Sync',
        checking: 'Checking',
        archiveCount: (count) => `${count} checked`,
        noDbRows: 'No DB rows',
        noLocalSession: 'No local session',
        sessionReady: 'Session ready',
        loadedCount: (count) => `${count} loaded`,
        roomCount: (count) => `${count} rooms`,
        roomMessageCount: (rooms, messages) => `${rooms} rooms · ${messages} messages`,
        manualSyncWaiting: 'Manual sync waiting',
        modelLoaded: 'ai/model GGUF loaded',
        modelNotLoaded: 'Model not loaded',
        defaultProfileSet: (id) => `Default profile ${id}`,
        loraTrainingNote: 'This can take minutes to hours on CPU and downloads the base model on first run.',
        appLoading: 'Connecting EverTalk local database',
        performanceGateTitle: 'Choose Performance Tier',
        performanceGateDescription: 'We detected this PC\'s specs. Choose the balance between response speed and accuracy. You can change this later in Settings.',
        performanceTierLight: 'Light',
        performanceTierBalanced: 'Balanced',
        performanceTierPerformance: 'Performance',
        performanceTierLightDescription: 'Uses minimal threads/context so it stays stable on lower-spec PCs.',
        performanceTierBalancedDescription: 'Default balanced setting suited to most PCs.',
        performanceTierPerformanceDescription: 'Uses maximum cores and context for faster, longer conversations.',
        hardwareDetected: (cores, memoryGb) => `Detected: ${cores} physical cores · ${memoryGb}GB memory`,
        recommendedTier: 'Recommended for this PC',
        activeSessionBadge: 'Session active',
        setupProgressTitle: 'Setting up EverTalk',
        setupStagePersonas: 'Preparing soul data',
        setupStageCaching: 'Caching conversation data for the selected language',
        setupStageModel: 'Loading local model',
        setupStageDone: 'Setup complete',
        setupProgressCount: (current, total) => `${current} / ${total}`,
        setupStepIndicator: (current, total) => `Step ${current} / ${total}`,
        setupLanguageStep: 'Language',
        setupDownloadStep: 'Model Download',
        setupPerformanceStep: 'Performance',
        downloadGateTitle: 'Download Local Model',
        downloadGateDescription: 'Download the local language model for offline conversations. Required only once.',
        downloadStart: 'Start Download',
        downloadPreparing: 'Preparing download',
        downloadProgressDetail: (downloadedMb, totalMb) => `${downloadedMb} / ${totalMb} MB`,
        downloadComplete: 'Download complete',
        downloadFailed: 'Download failed. Please try again.',
        appInfoTitle: 'About',
        appInfoDeveloper: 'Developer',
        appInfoContact: 'Contact',
        appInfoWebsite: 'Website',
        messageSendFailed: 'Failed to generate a response. Please try again.',
        downloadError: 'Model file download failed or file is missing.',
    },
    zh_cn: {
        languageGateTitle: '选择语言',
        languageGateDescription: '请选择 EverTalk 界面与本地模型回复使用的语言。',
        languageKo: '한국어',
        languageEn: 'English',
        languageZhCn: '简体中文',
        continue: '开始',
        rosterTitle: 'EverTalk',
        rosterSubtitle: (count) => `精灵消息 (${count})`,
        searchPlaceholder: '精灵名称或英文名',
        list: '列表',
        bondRanking: '羁绊排行',
        familiarity: '亲密度',
        loadingFamiliarity: '正在统计亲密度',
        noFamiliarity: '暂无亲密度数据',
        familiarityDescription: '将根据 SQLite 消息与记忆累积量计算亲密度。',
        messages: '消息',
        memories: '记忆',
        score: '分数',
        profileDetail: '详细资料',
        close: '关闭',
        bondStatus: 'BOND STATUS',
        profile: '资料',
        grade: '等级',
        race: '种族',
        className: '职业',
        subClass: '副职业',
        stat: '属性',
        union: '所属',
        constellation: '星座',
        birthday: '生日',
        height: '身高',
        weight: '体重',
        cvKo: '韩语声优',
        cvJp: '日语声优',
        like: '喜欢',
        dislike: '讨厌',
        hobby: '兴趣',
        speciality: '特技',
        personality: '个性数据',
        dialogueExamples: '对话示例',
        localStatus: '本地状态',
        personaCount: '精灵',
        chatRooms: '聊天室',
        chatMessages: '消息',
        styles: '风格',
        knowledge: '知识',
        localMemories: '记忆',
        imageGallery: '图片图库',
        backgroundGallery: '背景图库',
        zoomImage: '放大',
        settings: '设置',
        currentSettings: '当前设置 (settings.ini)',
        defaultSpirit: '默认精灵',
        activeStyle: '启用风格',
        language: '语言',
        displayResponseLanguage: '显示与回复语言',
        showReasoning: '显示 AI 推理过程 (<think>)',
        resetData: '重置数据',
        resetDescription: '删除聊天记录、精灵/风格/知识数据、累积记忆与设置。',
        resetComplete: '重置完成',
        resetFailed: '重置失败',
        notConfigured: '未设置',
        resetChatRooms: (count) => `${count} 个聊天室`,
        resetMessages: (count) => `${count} 条消息`,
        resetPersonas: (count) => `${count} 个精灵资料`,
        resetStyles: (count) => `${count} 个风格`,
        resetKnowledgeChunks: (count) => `${count} 个知识片段`,
        resetLocalMemories: (count) => `${count} 个累积记忆`,
        resetting: '正在重置...',
        resetConfirm: '再次点击确认重置',
        resetAllData: '重置全部数据',
        previousPage: '上一页',
        nextPage: '下一页',
        page: '页',
        selectSpirit: '选择精灵',
        modelReady: 'CPU LLM 已连接',
        modelWaiting: '等待本地模型',
        spiritReaction: '查看精灵反应',
        bondChannel: '羁绊频道',
        noSavedMessages: '暂无保存的对话',
        firstMessageHint: '发送第一条消息后会累积到 SQLite 会话。',
        messagePlaceholder: (name) => `向 ${name} 发送消息...`,
        modelRequiredPlaceholder: '请确认精灵与本地模型连接',
        send: '发送',
        chat: '聊天',
        gallery: '图库',
        dataLoadFailed: '精灵数据加载失败',
        databasePending: '等待精灵数据库',
        personaPackChecking: '正在确认 persona pack 与 SQLite 连接。',
        loadingBondRanking: '正在加载羁绊排行',
        noBondData: '暂无累积对话',
        bondDescription: '排行基于实际消息与保存记忆计算。',
        setDefaultProfile: (name) => `将 ${name} 设为默认资料`,
        personaDbLoading: '等待精灵数据库',
        settingsOpen: '打开设置',
        collapseRight: '收起右侧面板',
        expandRight: '展开右侧面板',
        collapseLeft: '收起左侧面板',
        expandLeft: '展开左侧面板',
        conversationKeywords: '对话关键词',
        noPersonality: '没有已登记介绍。',
        assetConnection: '资源连接',
        folder: '文件夹',
        background: '背景',
        disconnected: '未连接',
        loraTraining: '精灵 LoRA 训练',
        loraTrainingDescription: '使用该精灵累积的全部对话执行真实反向传播 LoRA 训练。',
        trainingComplete: '训练完成',
        trainingFailed: '训练失败',
        examples: '样本',
        steps: '步数',
        finalLoss: '最终损失',
        trainingRunning: '训练中...',
        startTraining: '开始训练',
        speakingStyle: '说话风格',
        syncing: '同步中',
        syncServerStyle: '同步服务器风格',
        emptyProfilePanel: '选择精灵后会显示 TBL 资料与原始资源连接状态。',
        systemStatus: '本地运行状态',
        authSession: '网页认证会话',
        personaPack: '人格包',
        personaDb: '精灵数据库',
        chatDb: '聊天数据库',
        styleDb: '风格数据库',
        localModel: '本地 GGUF 模型',
        dataSync: '数据包同步',
        checking: '确认中',
        archiveCount: (count) => `已确认 ${count} 个`,
        noDbRows: '没有数据库记录',
        noLocalSession: '没有本地会话',
        sessionReady: '会话已确认',
        loadedCount: (count) => `已加载 ${count} 个`,
        roomCount: (count) => `${count} 个聊天室`,
        roomMessageCount: (rooms, messages) => `${rooms} 个聊天室 · ${messages} 条消息`,
        manualSyncWaiting: '等待手动同步',
        modelLoaded: 'ai/model GGUF 已加载',
        modelNotLoaded: '模型未加载',
        defaultProfileSet: (id) => `默认资料 ${id}`,
        loraTrainingNote: 'CPU 执行可能需要数分钟到数小时，首次执行时会下载基础模型。',
        appLoading: '正在连接 EverTalk 本地数据库',
        performanceGateTitle: '选择性能等级',
        performanceGateDescription: '已检测此电脑的规格。请选择响应速度与准确度之间的平衡。之后可在设置中随时更改。',
        performanceTierLight: '轻量',
        performanceTierBalanced: '均衡',
        performanceTierPerformance: '高性能',
        performanceTierLightDescription: '使用最少的线程/上下文，即使在低配置电脑上也能稳定运行。',
        performanceTierBalancedDescription: '适合大多数电脑的默认均衡设置。',
        performanceTierPerformanceDescription: '最大化使用核心与上下文，处理更快、更长的对话。',
        hardwareDetected: (cores, memoryGb) => `检测到的规格：物理核心 ${cores} 个 · 内存 ${memoryGb}GB`,
        recommendedTier: '推荐用于此电脑',
        activeSessionBadge: '会话已激活',
        setupProgressTitle: '正在初始化 EverTalk',
        setupStagePersonas: '正在准备精灵数据',
        setupStageCaching: '正在按所选语言缓存对话数据',
        setupStageModel: '正在加载本地模型',
        setupStageDone: '配置完成',
        setupProgressCount: (current, total) => `${current} / ${total}`,
        setupStepIndicator: (current, total) => `步骤 ${current} / ${total}`,
        setupLanguageStep: '语言',
        setupDownloadStep: '模型下载',
        setupPerformanceStep: '性能',
        downloadGateTitle: '下载本地模型',
        downloadGateDescription: '下载用于离线对话的本地语言模型。仅需一次。',
        downloadStart: '开始下载',
        downloadPreparing: '正在准备下载',
        downloadProgressDetail: (downloadedMb, totalMb) => `${downloadedMb} / ${totalMb} MB`,
        downloadComplete: '下载完成',
        downloadFailed: '下载失败，请重试。',
        appInfoTitle: '程序信息',
        appInfoDeveloper: '开发者',
        appInfoContact: '联系',
        appInfoWebsite: '网站',
        messageSendFailed: '生成回复失败，请重试。',
        downloadError: '模型文件下载失败或找不到文件。',
    },
};

function createLabelBag(labelsByLanguage: Record<AppLanguage, EverTalkLabels>): EverTalkLabelBag {
    const labelKeys = Object.keys(labelsByLanguage.ko) as Array<keyof EverTalkLabels>;
    const bag: Partial<Record<keyof EverTalkLabels, Record<AppLanguage, unknown>>> = {};
    for (const key of labelKeys) {
        bag[key] = {
            ko: labelsByLanguage.ko[key],
            en: labelsByLanguage.en[key],
            zh_cn: labelsByLanguage.zh_cn[key],
        };
    }
    return bag as EverTalkLabelBag;
}

export const EVERTALK_LABEL_BAG: EverTalkLabelBag = createLabelBag(EVERTALK_LABELS);

export function getEverTalkLabels(language: AppLanguage): EverTalkLabels {
    return EVERTALK_LABELS[language];
}
