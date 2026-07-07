import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface TrainingSummary {
    persona_id: string;
    examples_used: number;
    steps: number;
    final_loss: number;
}

export interface TrainingProgress {
    persona_id: string;
    step: number;
    total_steps: number;
    loss: number;
}

export const trainingClient = {
    async run(personaId: string): Promise<TrainingSummary> {
        return invoke('train_lora', { personaId });
    },
    
    async onProgress(callback: (progress: TrainingProgress) => void): Promise<UnlistenFn> {
        return listen<TrainingProgress>('training-progress', (event) => {
            callback(event.payload);
        });
    }
};
