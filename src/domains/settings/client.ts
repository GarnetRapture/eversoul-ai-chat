import { invokeCommand, tauriCommands } from '../../shared/api';
import type { AppLanguage, PerformanceTier } from '../../shared/types';
import { AppSettings, HardwareProfile, ResetSummary } from './types';
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
    async setPerformanceTier(tier: PerformanceTier): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.setPerformanceTier, { tier });
    },
    async detectHardware(): Promise<HardwareProfile> {
        return invokeCommand<HardwareProfile>(tauriCommands.settings.detectHardware);
    },
    async completeInitialSetup(language: AppLanguage, tier: PerformanceTier): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.completeInitialSetup, { language, tier });
    },
};
