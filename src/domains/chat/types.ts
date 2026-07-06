export interface ChatRoom {
    id: string;
    title: string;
    persona_id: string | null;
    session_started_at: string;
    created_at: string;
    updated_at: string;
}
export interface ChatMessage {
    id: string;
    room_id: string;
    persona_id: string | null;
    role: 'user' | 'assistant' | 'system';
    content: string;
    created_at: string;
}
export type ChatError = {
    Database: string;
} | 'LlmEngineNotLoaded' | {
    LlmInferenceFailed: string;
} | {
    Unknown: string;
};
