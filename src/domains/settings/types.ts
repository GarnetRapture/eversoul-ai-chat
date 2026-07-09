import type { AppLanguage, PerformanceTier, InferenceMode, ApiProvider } from '../../shared/types';

export type SetupPhase = 'language' | 'mode' | 'download' | 'performance' | 'done';
export interface AppSettings {
    default_persona_id: string | null;
    active_style_id: string | null;
    language: AppLanguage;
    language_configured: boolean;
    inference_mode: InferenceMode;
    api_provider: ApiProvider | null;
    api_key: string | null;
    performance_tier: PerformanceTier;
    performance_configured: boolean;
    setup_stage: SetupPhase;
    show_reasoning: boolean;
    external_api: ExternalApiSettings;
}
export interface ExternalApiSettings {
    enabled: boolean;
    base_url: string;
    api_key_configured: boolean;
    model: string;
}
export interface ExternalApiConfigRequest {
    enabled: boolean;
    base_url: string;
    api_key: string;
    model: string;
}
export interface ExternalApiTestResult {
    ok: boolean;
    message: string;
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
