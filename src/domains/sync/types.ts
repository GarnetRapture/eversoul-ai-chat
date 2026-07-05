export interface SyncResult {
    success: boolean;
    synced_items: number;
    error_message: string | null;
}
export interface LocalStatusSnapshot {
    persona_count: number;
    chat_room_count: number;
    chat_message_count: number;
    style_count: number;
    knowledge_chunk_count: number;
    memory_count: number;
    last_sync_status: string | null;
    last_sync_error: string | null;
}
export type SyncError = {
    Database: string;
} | {
    Network: string;
} | {
    Unknown: string;
};
