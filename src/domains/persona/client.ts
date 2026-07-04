import { invokeCommand, tauriCommands } from '../../shared/api';
import { PersonaConfig } from './types';

export const personaClient = {
  /**
   * 로컬 DB에 수립된 페르소나 설정 목록을 조회한다.
   */
  async list(): Promise<PersonaConfig[]> {
    return invokeCommand<PersonaConfig[]>(tauriCommands.persona.list);
  },

  /**
   * 페르소나의 설정값을 수정하여 SQLite 로컬 DB에 반영한다.
   */
  async update(
    id: string,
    systemPrompt: string,
    greeting: string
  ): Promise<void> {
    return invokeCommand<void>(tauriCommands.persona.update, { id, system_prompt: systemPrompt, greeting });
  },

  /**
   * 압축 아카이브(personas.bin)에 내장된 모든 정령의 영어명 리스트를 가져온다.
   */
  async listArchive(): Promise<string[]> {
    return invokeCommand<string[]>(tauriCommands.persona.listArchive);
  },

  /**
   * 특정 영어명을 기준으로 압축 팩에서 해당 정령의 종합 데이터 JSON을 추출한다.
   */
  async getPack(nameEn: string): Promise<unknown> {
    return invokeCommand<unknown>(tauriCommands.persona.getPack, { name_en: nameEn });
  },

  /**
   * 특정 영어명을 프리셋으로 선택하여 아카이브로부터 데이터를 로컬 DB 설정에Upsert 주입한다.
   */
  async selectPreset(nameEn: string): Promise<PersonaConfig> {
    return invokeCommand<PersonaConfig>(tauriCommands.persona.selectPreset, { name_en: nameEn });
  },
};
