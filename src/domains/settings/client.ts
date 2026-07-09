import { invokeCommand, tauriCommands } from '../../shared/api';
import type { AppLanguage, PerformanceTier } from '../../shared/types';
import { AppSettings, ExternalApiConfigRequest, ExternalApiTestResult, HardwareProfile, ResetSummary } from './types';
import type { ApiProvider, InferenceMode } from '../../shared/types';
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
    async completeInitialSetup(language: AppLanguage, inferenceMode: InferenceMode, apiProvider: ApiProvider | null, apiKey: string | null, tier: PerformanceTier): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.completeInitialSetup, { 
            language, 
            inference_mode: inferenceMode, 
            api_provider: apiProvider, 
            api_key: apiKey, 
            tier 
        });
    },
    async setSetupStage(stage: string): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.setSetupStage, { stage });
    },
    async setShowReasoning(showReasoning: boolean): Promise<AppSettings> {
        return invokeCommand<AppSettings>('settings_set_show_reasoning', { show_reasoning: showReasoning });
    },
    async setInferenceMode(mode: InferenceMode): Promise<AppSettings> {
        return invokeCommand<AppSettings>('settings_set_inference_mode', { mode });
    },
    async setApiProvider(provider: ApiProvider | null): Promise<AppSettings> {
        return invokeCommand<AppSettings>('settings_set_api_provider', { provider });
    },
    async setApiKey(key: string | null): Promise<AppSettings> {
        return invokeCommand<AppSettings>('settings_set_api_key', { key });
    },
    async setActiveLocalModel(model: string): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.setActiveLocalModel, { model });
    },
    async setExternalApiConfig(request: ExternalApiConfigRequest): Promise<AppSettings> {
        return invokeCommand<AppSettings>(tauriCommands.settings.setExternalApiConfig, { request });
    },
    async testExternalApi(): Promise<ExternalApiTestResult> {
        return invokeCommand<ExternalApiTestResult>(tauriCommands.settings.testExternalApi);
    },
};
