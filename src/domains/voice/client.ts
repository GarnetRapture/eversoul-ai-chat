import { invokeCommand, tauriCommands } from '../../shared/api';
export const voiceClient = {
    async list(personaId: string): Promise<string[]> {
        return invokeCommand<string[]>(tauriCommands.voice.list, { persona_id: personaId });
    },
    async get(personaId: string, fileName: string): Promise<number[]> {
        return invokeCommand<number[]>(tauriCommands.voice.get, {
            persona_id: personaId,
            file_name: fileName,
        });
    },
    async synthesize(personaId: string, text: string): Promise<number[]> {
        return invokeCommand<number[]>(tauriCommands.voice.synthesize, {
            persona_id: personaId,
            text,
        });
    },
};
