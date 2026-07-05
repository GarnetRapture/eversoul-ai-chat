import type { AppLanguage } from '../../shared/types';

export interface AppSettings {
    default_persona_id: string | null;
    active_style_id: string | null;
    language: AppLanguage;
    language_configured: boolean;
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
};
