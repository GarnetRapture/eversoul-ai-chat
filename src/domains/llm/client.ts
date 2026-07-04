import { invokeCommand, tauriCommands } from '../../shared/api';
import { LlmStatus, LlmInferResponse } from './types';

export const llmClient = {
  /**
   * 로컬 모델 디렉터리를 가리켜 CPU 기반 qwen2.5 추론 엔진을 메모리에 탑재한다.
   */
  async loadEngine(): Promise<LlmStatus> {
    return invokeCommand<LlmStatus>(tauriCommands.llm.load);
  },

  /**
   * 현재 로컬 LLM 로딩 현황 상태 정보를 획득한다.
   */
  async getStatus(): Promise<LlmStatus> {
    return invokeCommand<LlmStatus>(tauriCommands.llm.status);
  },

  /**
   * 임의의 프롬프트와 옵션을 엮어 로컬 LLM 추론 결과를 직접 받아온다 (자유 테스트용).
   */
  async infer(prompt: string, maxTokens?: number): Promise<LlmInferResponse> {
    return invokeCommand<LlmInferResponse>(tauriCommands.llm.infer, {
      prompt,
      max_tokens: maxTokens,
    });
  },
};
