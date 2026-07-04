import { invoke } from '@tauri-apps/api/core';
import { KnowledgeChunk } from './types';

export const knowledgeClient = {
  /**
   * 로컬 SQLite DB에서 질의와 유사한 로컬 지식 텍스트 청크를 검색한다 (RAG용).
   */
  async search(query: string, limit?: number): Promise<KnowledgeChunk[]> {
    return invoke<KnowledgeChunk[]>('knowledge_search', { query, limit });
  },
};
