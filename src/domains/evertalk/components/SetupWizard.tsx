import type { AppLanguage, PerformanceTier } from '../../../shared/types';
import type { SetupWizardProps } from '../types';

const LANGUAGE_OPTIONS: AppLanguage[] = ['ko', 'en', 'zh_cn'];
const TIER_OPTIONS: PerformanceTier[] = ['light', 'balanced', 'performance'];
const SETUP_ORDER = ['language', 'mode', 'modelSelect', 'download', 'performance'] as const;

export function SetupWizard({
    open,
    stage,
    language,
    inferenceMode,
    apiProvider,
    apiKey,
    availableModels,
    selectedLocalModel,
    tier,
    hardwareProfile,
    downloadProgress,
    downloadError,
    isDownloading,
    labels,
    onSelectLanguage,
    onSelectInferenceMode,
    onSelectLocalModel,
    onSelectApiProvider,
    onChangeApiKey,
    onStartDownload,
    onNextStage,
    onSelectTier,
    onCompleteSetup,
}: SetupWizardProps) {
    if (!open) {
        return null;
    }

    function languageLabel(option: AppLanguage): string {
        if (option === 'en') {
            return labels.languageEn;
        }
        if (option === 'zh_cn') {
            return labels.languageZhCn;
        }
        return labels.languageKo;
    }

    function tierLabel(option: PerformanceTier): string {
        if (option === 'light') {
            return labels.performanceTierLight;
        }
        if (option === 'performance') {
            return labels.performanceTierPerformance;
        }
        return labels.performanceTierBalanced;
    }

    function tierDescription(option: PerformanceTier): string {
        if (option === 'light') {
            return labels.performanceTierLightDescription;
        }
        if (option === 'performance') {
            return labels.performanceTierPerformanceDescription;
        }
        return labels.performanceTierBalancedDescription;
    }

    function stepName(step: (typeof SETUP_ORDER)[number]): string {
        if (step === 'language') {
            return labels.setupLanguageStep;
        }
        if (step === 'mode') {
            return labels.inferenceMode;
        }
        if (step === 'download') {
            return labels.setupDownloadStep;
        }
        if (step === 'modelSelect') {
            return '로컬 모델 선택'; // 다국어 지원은 labels 쪽에 나중에 추가
        }
        return labels.setupPerformanceStep;
    }

    const currentIndex = SETUP_ORDER.indexOf(stage as (typeof SETUP_ORDER)[number]);
    const downloadedMb = downloadProgress
        ? Math.round(downloadProgress.downloaded_bytes / (1024 * 1024))
        : 0;
    const totalMb = downloadProgress
        ? Math.round(downloadProgress.total_bytes / (1024 * 1024))
        : 0;
    const ratioPercent = downloadProgress ? Math.round(downloadProgress.ratio * 100) : 0;

    return (
        <div className="ever-settings-overlay ever-setup-wizard" role="dialog" aria-modal="true">
            <div className="ever-setup-wizard__modal">
                <ol className="ever-setup-wizard__steps">
                    {SETUP_ORDER.map((step, index) => (
                        <li
                            key={step}
                            className={
                                index === currentIndex
                                    ? 'is-active'
                                    : index < currentIndex
                                        ? 'is-done'
                                        : ''
                            }
                        >
                            <span className="ever-setup-wizard__step-index">
                                {labels.setupStepIndicator(index + 1, SETUP_ORDER.length)}
                            </span>
                            <span className="ever-setup-wizard__step-name">{stepName(step)}</span>
                        </li>
                    ))}
                </ol>

                {stage === 'language' && (
                    <div className="ever-setup-wizard__body">
                        <h2>{labels.languageGateTitle}</h2>
                        <p>{labels.languageGateDescription}</p>
                        <div className="ever-language-gate__options">
                            {LANGUAGE_OPTIONS.map((option) => (
                                <button
                                    key={option}
                                    type="button"
                                    className={language === option ? 'is-active' : ''}
                                    onClick={() => void onSelectLanguage(option)}
                                >
                                    {languageLabel(option)}
                                </button>
                            ))}
                        </div>
                        <button type="button" onClick={() => void onNextStage()} style={{ marginTop: '1rem' }}>
                            {labels.continue ?? 'Next'}
                        </button>
                    </div>
                )}

                {stage === 'mode' && (
                    <div className="ever-setup-wizard__body">
                        <h2>{labels.setupModeGateTitle}</h2>
                        <p>{labels.setupModeGateDescription}</p>
                        <div className="ever-language-gate__options" style={{ marginBottom: '1rem' }}>
                            <button
                                type="button"
                                className={inferenceMode === 'local' ? 'is-active' : ''}
                                onClick={() => onSelectInferenceMode('local')}
                            >
                                <strong>{labels.modeLocal}</strong>
                                <span>{labels.modeLocalDescription}</span>
                            </button>
                            <button
                                type="button"
                                className={inferenceMode === 'api' ? 'is-active' : ''}
                                onClick={() => onSelectInferenceMode('api')}
                            >
                                <strong>{labels.modeExternalApi}</strong>
                                <span>{labels.modeExternalApiDescription}</span>
                            </button>
                        </div>
                        
                        {inferenceMode === 'api' && (
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem', textAlign: 'left' }}>
                                <label>
                                    <span style={{ display: 'block', fontSize: '0.9rem', marginBottom: '0.2rem' }}>{labels.apiProvider}</span>
                                    <select 
                                        value={apiProvider ?? 'openai'} 
                                        onChange={(e) => onSelectApiProvider(e.target.value as any)}
                                        style={{ padding: '0.5rem', width: '100%', background: 'rgba(0,0,0,0.2)', color: 'white', border: '1px solid rgba(255,255,255,0.1)' }}
                                    >
                                        <option value="openai">OpenAI (ChatGPT)</option>
                                        <option value="anthropic">Anthropic (Claude)</option>
                                        <option value="gemini">Google (Gemini)</option>
                                    </select>
                                </label>
                                <label>
                                    <span style={{ display: 'block', fontSize: '0.9rem', marginBottom: '0.2rem' }}>{labels.apiKey}</span>
                                    <input 
                                        type="password" 
                                        value={apiKey ?? ''}
                                        placeholder={labels.apiKeyPlaceholder}
                                        onChange={(e) => onChangeApiKey(e.target.value)}
                                        style={{ padding: '0.5rem', width: '100%', background: 'rgba(0,0,0,0.2)', color: 'white', border: '1px solid rgba(255,255,255,0.1)' }}
                                    />
                                </label>
                            </div>
                        )}

                        <button 
                            type="button" 
                            onClick={() => inferenceMode === 'api' ? onCompleteSetup() : onNextStage()}
                        >
                            {labels.continue ?? 'Next'}
                        </button>
                    </div>
                )}

                {stage === 'modelSelect' && (
                    <div className="ever-setup-wizard__body">
                        <h2>로컬 모델 선택</h2>
                        <p>PC 환경에서 독립적으로 실행할 로컬 인공지능 모델을 선택해주세요.</p>
                        <div className="ever-language-gate__options" style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem' }}>
                            {availableModels?.map(model => (
                                <button
                                    key={model.id}
                                    type="button"
                                    className={selectedLocalModel === model.id ? 'is-active' : ''}
                                    onClick={() => onSelectLocalModel(model.id)}
                                >
                                    <div style={{ display: 'flex', justifyContent: 'space-between', width: '100%' }}>
                                        <strong>{model.name}</strong>
                                        {model.is_downloaded && <span style={{ color: '#4caf50' }}>[설치됨]</span>}
                                    </div>
                                    <div style={{ fontSize: '0.8rem', opacity: 0.7, textAlign: 'left', marginTop: '0.2rem' }}>
                                        {model.is_downloaded ? '이 모델은 즉시 사용 가능합니다.' : '추가 다운로드가 필요합니다.'}
                                    </div>
                                </button>
                            ))}
                        </div>
                        <button type="button" onClick={() => void onNextStage()} style={{ marginTop: '1rem' }} disabled={!selectedLocalModel}>
                            {labels.continue ?? 'Next'}
                        </button>
                    </div>
                )}

                {stage === 'download' && (
                    <div className="ever-setup-wizard__body">
                        <h2>{labels.downloadGateTitle}</h2>
                        <p>{labels.downloadGateDescription}</p>
                        <div className="ever-setup-wizard__progress">
                            <div
                                className="ever-setup-wizard__progress-bar"
                                style={{ width: `${ratioPercent}%` }}
                            />
                        </div>
                        <p className="ever-setup-wizard__progress-detail">
                            {isDownloading
                                ? `${labels.downloadProgressDetail(downloadedMb, totalMb)} (${ratioPercent}%)`
                                : downloadProgress?.done
                                    ? labels.downloadComplete
                                    : labels.downloadPreparing}
                        </p>
                        {downloadError && (
                            <p className="ever-setup-wizard__error">{downloadError}</p>
                        )}
                        {downloadProgress?.done ? (
                            <button
                                type="button"
                                onClick={() => void onNextStage()}
                            >
                                {labels.continue ?? '다음'}
                            </button>
                        ) : (
                            <button
                                type="button"
                                disabled={isDownloading}
                                onClick={() => void onStartDownload()}
                            >
                                {labels.downloadStart}
                            </button>
                        )}
                    </div>
                )}

                {stage === 'performance' && (
                    <div className="ever-setup-wizard__body">
                        <h2>{labels.performanceGateTitle}</h2>
                        <p>{labels.performanceGateDescription}</p>
                        {hardwareProfile && (
                            <p className="ever-performance-gate__hardware">
                                {labels.hardwareDetected(
                                    hardwareProfile.physical_core_count,
                                    Math.round(hardwareProfile.total_memory_mb / 1024),
                                )}
                            </p>
                        )}
                        <div className="ever-performance-gate__options">
                            {TIER_OPTIONS.map((option) => (
                                <button
                                    key={option}
                                    type="button"
                                    className={tier === option ? 'is-active' : ''}
                                    onClick={() => void onSelectTier(option)}
                                >
                                    <strong>{tierLabel(option)}</strong>
                                    <span>{tierDescription(option)}</span>
                                    {hardwareProfile?.recommended_tier === option && (
                                        <em>{labels.recommendedTier}</em>
                                    )}
                                </button>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
