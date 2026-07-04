import { invokeCommand, tauriCommands } from '../../shared/api';
import { TrainingSummary } from './types';
export const trainingClient = {
    async run(personaId: string): Promise<TrainingSummary> {
        return invokeCommand<TrainingSummary>(tauriCommands.training.run, { persona_id: personaId });
    },
};
