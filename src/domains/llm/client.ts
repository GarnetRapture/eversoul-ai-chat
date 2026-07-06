import { invokeCommand, tauriCommands } from '../../shared/api';
import {
    LlmStatus,
    LlmInferResponse,
    LlmModelValidation,
    LlmRequestStatus,
    LlmSessionStatus,
    LlmStreamInferRequest,
} from './types';
export const llmClient = {
    async loadEngine(): Promise<LlmStatus> {
        return invokeCommand<LlmStatus>(tauriCommands.llm.load);
    },
    async getStatus(): Promise<LlmStatus> {
        return invokeCommand<LlmStatus>(tauriCommands.llm.status);
    },
    async unloadEngine(): Promise<void> {
        return invokeCommand<void>(tauriCommands.llm.unload);
    },
    async infer(prompt: string, maxTokens?: number): Promise<LlmInferResponse> {
        return invokeCommand<LlmInferResponse>(tauriCommands.llm.infer, {
            prompt,
            max_tokens: maxTokens,
        });
    },
    async inferStream(request: LlmStreamInferRequest): Promise<LlmInferResponse> {
        return invokeCommand<LlmInferResponse>(tauriCommands.llm.inferStream, { request });
    },
    async cancelRequest(requestId: string): Promise<boolean> {
        return invokeCommand<boolean>(tauriCommands.llm.cancelRequest, { request_id: requestId });
    },
    async getActiveSessions(): Promise<string[]> {
        return invokeCommand<string[]>(tauriCommands.llm.activeSessions);
    },
    async getSessionStatuses(): Promise<LlmSessionStatus[]> {
        return invokeCommand<LlmSessionStatus[]>(tauriCommands.llm.sessionStatuses);
    },
    async getRequestStatuses(): Promise<LlmRequestStatus[]> {
        return invokeCommand<LlmRequestStatus[]>(tauriCommands.llm.requestStatuses);
    },
    async verifyModel(): Promise<LlmModelValidation> {
        return invokeCommand<LlmModelValidation>(tauriCommands.llm.verifyModel);
    },
    async selfTest(): Promise<LlmInferResponse> {
        return invokeCommand<LlmInferResponse>(tauriCommands.llm.selfTest);
    },
};
