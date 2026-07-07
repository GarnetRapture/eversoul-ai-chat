import type React from 'react';
import type { WarmupOverlayProps } from '../types';

export const WarmupOverlay: React.FC<WarmupOverlayProps> = ({ warmupState, labels }) => {
    if (!warmupState.isActive) {
        return null;
    }

    const { currentIndex, totalPersonas, currentPersonaName, progress } = warmupState;
    
    // Overall progress calculation
    const overallPercentage = totalPersonas > 0 ? (currentIndex / totalPersonas) * 100 : 0;
    
    // Current persona progress calculation
    const currentPercentage = progress && progress.total > 0 
        ? Math.min((progress.current / progress.total) * 100, 100) 
        : 0;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md transition-all duration-300">
            <div className="bg-slate-900 border border-indigo-500/30 rounded-xl p-8 max-w-lg w-full shadow-2xl flex flex-col items-center">
                
                <div className="w-16 h-16 mb-6 rounded-full bg-indigo-600/20 flex items-center justify-center border border-indigo-500/50">
                    <svg className="w-8 h-8 text-indigo-400 animate-spin" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                </div>
                
                <h2 className="text-2xl font-bold text-white mb-2 text-center tracking-tight">
                    {labels.warmupTitle}
                </h2>
                <p className="text-indigo-200/80 mb-8 text-center text-sm">
                    오프라인 캐시를 구축하여 즉각적인 응답 속도를 준비하고 있습니다.
                </p>

                {/* Overall Progress */}
                <div className="w-full mb-6">
                    <div className="flex justify-between text-xs text-indigo-200 mb-2 font-medium">
                        <span>{labels.warmupOverall(currentIndex + 1, totalPersonas)}</span>
                        <span>{Math.round(overallPercentage)}%</span>
                    </div>
                    <div className="w-full bg-slate-800 rounded-full h-2 overflow-hidden border border-slate-700">
                        <div 
                            className="bg-indigo-500 h-2 rounded-full transition-all duration-500 ease-out" 
                            style={{ width: `${overallPercentage}%` }}
                        />
                    </div>
                </div>

                {/* Current Persona Progress */}
                <div className="w-full">
                    <div className="flex justify-between text-xs text-indigo-300 mb-2 font-medium">
                        <span>{currentPersonaName}</span>
                        <span>
                            {progress 
                                ? labels.warmupProgress(progress.current, progress.total) 
                                : '대기 중...'}
                        </span>
                    </div>
                    <div className="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden border border-slate-700">
                        <div 
                            className="bg-emerald-400 h-1.5 rounded-full transition-all duration-200 ease-out" 
                            style={{ width: `${currentPercentage}%` }}
                        />
                    </div>
                </div>
                
            </div>
        </div>
    );
};
