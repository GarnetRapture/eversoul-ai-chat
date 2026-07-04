import { invokeCommand, tauriCommands } from '../../shared/api';
import { StyleProfile } from './types';
export const styleClient = {
    async list(): Promise<StyleProfile[]> {
        return invokeCommand<StyleProfile[]>(tauriCommands.style.list);
    },
    async update(id: string, tone: string, formality: string, emojiUsage: boolean, speechRules: string): Promise<void> {
        return invokeCommand<void>(tauriCommands.style.update, {
            id,
            tone,
            formality,
            emoji_usage: emojiUsage,
            speech_rules: speechRules,
        });
    },
    async selectActive(id: string): Promise<StyleProfile> {
        return invokeCommand<StyleProfile>(tauriCommands.style.selectActive, { id });
    },
    async getActive(): Promise<StyleProfile | null> {
        return invokeCommand<StyleProfile | null>(tauriCommands.style.getActive);
    },
};
