import { invokeCommand, tauriCommands } from '../../shared/api';
import { LlmStatus, LlmInferResponse } from './types';
export const llmClient = {
    async loadEngine(): Promise<LlmStatus> {
        return invokeCommand<LlmStatus>(tauriCommands.llm.load);
    },
    async getStatus(): Promise<LlmStatus> {
        return invokeCommand<LlmStatus>(tauriCommands.llm.status);
    },
    async infer(prompt: string, maxTokens?: number): Promise<LlmInferResponse> {
        return invokeCommand<LlmInferResponse>(tauriCommands.llm.infer, {
            prompt,
            max_tokens: maxTokens,
        });
    },
};
