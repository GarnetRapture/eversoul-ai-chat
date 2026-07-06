import type { PerformanceTier } from '../../../shared/types';
import type { PerformanceGatePanelProps } from '../types';

const TIER_OPTIONS: PerformanceTier[] = ['light', 'balanced', 'performance'];

export function PerformanceGatePanel({ open, tier, hardwareProfile, labels, onSelectTier }: PerformanceGatePanelProps) {
    if (!open) {
        return null;
    }

    function getTierLabel(option: PerformanceTier): string {
        if (option === 'light') {
            return labels.performanceTierLight;
        }
        if (option === 'performance') {
            return labels.performanceTierPerformance;
        }
        return labels.performanceTierBalanced;
    }

    function getTierDescription(option: PerformanceTier): string {
        if (option === 'light') {
            return labels.performanceTierLightDescription;
        }
        if (option === 'performance') {
            return labels.performanceTierPerformanceDescription;
        }
        return labels.performanceTierBalancedDescription;
    }

    return (
        <div className="ever-settings-overlay ever-performance-gate" role="dialog" aria-modal="true">
            <div className="ever-performance-gate__modal">
                <h2>{labels.performanceGateTitle}</h2>
                <p>{labels.performanceGateDescription}</p>
                {hardwareProfile && (
                    <p className="ever-performance-gate__hardware">
                        {labels.hardwareDetected(hardwareProfile.physical_core_count, Math.round(hardwareProfile.total_memory_mb / 1024))}
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
                            <strong>{getTierLabel(option)}</strong>
                            <span>{getTierDescription(option)}</span>
                            {hardwareProfile?.recommended_tier === option && (
                                <em>{labels.recommendedTier}</em>
                            )}
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
}
