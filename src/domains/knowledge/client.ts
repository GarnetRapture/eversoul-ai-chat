import { invokeCommand, tauriCommands } from '../../shared/api';
import { KnowledgeChunk } from './types';
export const knowledgeClient = {
    async search(query: string, limit?: number): Promise<KnowledgeChunk[]> {
        return invokeCommand<KnowledgeChunk[]>(tauriCommands.knowledge.search, { query, limit });
    },
};
