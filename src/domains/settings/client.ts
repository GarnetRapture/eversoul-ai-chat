import { invokeCommand, tauriCommands } from '../../shared/api';
import type { AppLanguage } from '../../shared/types';
import { AppSettings, ResetSummary } from './types';
export const settingsClient = {
    async get(): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.get);
    },
    async reset(): Promise<ResetSummary> {
        return invokeCommand<ResetSummary>(tauriCommands.settings.reset);
    },
    async setLanguage(language: AppLanguage): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.setLanguage, { language });
    },
};
