import { invokeCommand, tauriCommands } from '../../shared/api';
import { StyleProfile } from './types';

export const styleClient = {
  /**
   * 로컬 DB에 수립된 스타일(말투) 프로필 목록을 조회한다.
   */
  async list(): Promise<StyleProfile[]> {
    return invokeCommand<StyleProfile[]>(tauriCommands.style.list);
  },

  /**
   * 스타일 프로필의 설정값을 수정하여 SQLite 로컬 DB에 반영한다.
   */
  async update(
    id: string,
    tone: string,
    formality: string,
    emojiUsage: boolean,
    speechRules: string
  ): Promise<void> {
    return invokeCommand<void>(tauriCommands.style.update, {
      id,
      tone,
      formality,
      emoji_usage: emojiUsage,
      speech_rules: speechRules,
    });
  },

  /**
   * 지정된 스타일을 현재 활성 스타일로 선택한다.
   */
  async selectActive(id: string): Promise<StyleProfile> {
    return invokeCommand<StyleProfile>(tauriCommands.style.selectActive, { id });
  },

  /**
   * 현재 활성화된 스타일 프로필을 조회한다.
   */
  async getActive(): Promise<StyleProfile | null> {
    return invokeCommand<StyleProfile | null>(tauriCommands.style.getActive);
  },
};
