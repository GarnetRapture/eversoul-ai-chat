import { invokeCommand, tauriCommands } from '../../shared/api';
import { ChatRoom, ChatMessage } from './types';

export const chatClient = {
  /**
   * 새 대화방을 생성한다.
   */
  async createRoom(title: string): Promise<ChatRoom> {
    return invokeCommand<ChatRoom>(tauriCommands.chat.createRoom, { title });
  },

  /**
   * 대화방 목록을 조회한다.
   */
  async listRooms(): Promise<ChatRoom[]> {
    return invokeCommand<ChatRoom[]>(tauriCommands.chat.listRooms);
  },

  /**
   * 특정 대화방의 상세 메시지 목록을 가져온다.
   */
  async listMessages(roomId: string): Promise<ChatMessage[]> {
    return invokeCommand<ChatMessage[]>(tauriCommands.chat.listMessages, { room_id: roomId });
  },

  /**
   * 사용자 메시지를 기록하고 AI 응답을 수립(추론)하여 받아온다.
   */
  async sendMessage(
    roomId: string,
    content: string,
    personaId: string
  ): Promise<ChatMessage> {
    return invokeCommand<ChatMessage>(tauriCommands.chat.sendMessage, {
      room_id: roomId,
      content,
      persona_id: personaId,
    });
  },
};
