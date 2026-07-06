import type { AppLanguage, PerformanceTier } from '../../shared/types';

export interface AppSettings {
    default_persona_id: string | null;
    active_style_id: string | null;
    language: AppLanguage;
    language_configured: boolean;
    performance_tier: PerformanceTier;
    performance_configured: boolean;
}
export interface HardwareProfile {
    physical_core_count: number;
    logical_core_count: number;
    total_memory_mb: number;
    recommended_tier: PerformanceTier;
}
export type SetupStage = 'personas' | 'caching' | 'model' | 'done';
export interface SetupProgress {
    stage: SetupStage;
    current: number;
    total: number;
}
export interface ResetSummary {
    cleared_chat_rooms: number;
    cleared_chat_messages: number;
    cleared_personas: number;
    cleared_styles: number;
    cleared_knowledge_chunks: number;
    cleared_persona_memories: number;
}
export type SettingsError = {
    Io: string;
} | {
    Database: string;
} | {
    Validation: string;
};
