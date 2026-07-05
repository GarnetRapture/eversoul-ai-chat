import { invokeCommand, tauriCommands } from '../../shared/api';
import { BondRankingEntry, FamiliarityEntry, PersonaConfig } from './types';
export const personaClient = {
    async list(): Promise<PersonaConfig[]> {
        return invokeCommand<PersonaConfig[]>(tauriCommands.persona.list);
    },
    async update(id: string, systemPrompt: string, greeting: string): Promise<void> {
        return invokeCommand<void>(tauriCommands.persona.update, { id, system_prompt: systemPrompt, greeting });
    },
    async listArchive(): Promise<string[]> {
        return invokeCommand<string[]>(tauriCommands.persona.listArchive);
    },
    async getPack(nameEn: string): Promise<unknown> {
        return invokeCommand<unknown>(tauriCommands.persona.getPack, { name_en: nameEn });
    },
    async selectPreset(nameEn: string): Promise<PersonaConfig> {
        return invokeCommand<PersonaConfig>(tauriCommands.persona.selectPreset, { name_en: nameEn });
    },
    async getDefault(): Promise<string | null> {
        return invokeCommand<string | null>(tauriCommands.persona.getDefault);
    },
    async setDefault(id: string): Promise<string> {
        return invokeCommand<string>(tauriCommands.persona.setDefault, { id });
    },
    async getBondRanking(): Promise<BondRankingEntry[]> {
        return invokeCommand<BondRankingEntry[]>(tauriCommands.persona.bondRanking);
    },
    async getFamiliarityList(): Promise<FamiliarityEntry[]> {
        return invokeCommand<FamiliarityEntry[]>(tauriCommands.persona.familiarityList);
    },
};
