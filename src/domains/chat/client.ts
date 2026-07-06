import { invokeCommand, tauriCommands } from '../../shared/api';
import { ChatRoom, ChatMessage } from './types';
export const chatClient = {
    async createRoom(title: string): Promise<ChatRoom> {
        return invokeCommand<ChatRoom>(tauriCommands.chat.createRoom, { title });
    },
    async createSessionRoom(title: string, personaId: string): Promise<ChatRoom> {
        return invokeCommand<ChatRoom>(tauriCommands.chat.createSessionRoom, {
            title,
            persona_id: personaId,
        });
    },
    async getEverTalkSessionRoom(): Promise<ChatRoom> {
        return invokeCommand<ChatRoom>(tauriCommands.chat.getEverTalkSessionRoom);
    },
    async getLatestSessionRoom(personaId: string): Promise<ChatRoom | null> {
        return invokeCommand<ChatRoom | null>(tauriCommands.chat.getLatestSessionRoom, {
            persona_id: personaId,
        });
    },
    async listRooms(): Promise<ChatRoom[]> {
        return invokeCommand<ChatRoom[]>(tauriCommands.chat.listRooms);
    },
    async listMessages(roomId: string): Promise<ChatMessage[]> {
        return invokeCommand<ChatMessage[]>(tauriCommands.chat.listMessages, { room_id: roomId });
    },
    async listMessagesForPersona(roomId: string, personaId: string): Promise<ChatMessage[]> {
        return invokeCommand<ChatMessage[]>(tauriCommands.chat.listMessagesForPersona, {
            room_id: roomId,
            persona_id: personaId,
        });
    },
    async sendMessage(roomId: string, content: string, personaId: string): Promise<ChatMessage> {
        return invokeCommand<ChatMessage>(tauriCommands.chat.sendMessage, {
            room_id: roomId,
            content,
            persona_id: personaId,
        });
    },
};
