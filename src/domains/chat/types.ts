export interface ChatRoom {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  room_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  created_at: string;
}

export type ChatError =
  | { Database: string }
  | 'LlmEngineNotLoaded'
  | { LlmInferenceFailed: string }
  | { Unknown: string };
