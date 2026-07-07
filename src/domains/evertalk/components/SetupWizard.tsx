import type { AppLanguage, PerformanceTier } from '../../../shared/types';
import type { SetupWizardProps } from '../types';

const LANGUAGE_OPTIONS: AppLanguage[] = ['ko', 'en', 'zh_cn'];
const TIER_OPTIONS: PerformanceTier[] = ['light', 'balanced', 'performance'];
const SETUP_ORDER = ['language', 'download', 'performance'] as const;

export function SetupWizard({
    open,
    stage,
    language,
    tier,
    hardwareProfile,
    downloadProgress,
    downloadError,
    isDownloading,
    labels,
    onSelectLanguage,
    onStartDownload,
    onSelectTier,
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
        if (step === 'download') {
            return labels.setupDownloadStep;
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
                        <button
                            type="button"
                            disabled={isDownloading}
                            onClick={() => void onStartDownload()}
                        >
                            {labels.downloadStart}
                        </button>
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
