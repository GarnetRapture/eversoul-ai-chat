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


export type TrainingErrorCode =
    | 'db_lock'
    | 'persona_not_found'
    | 'query_failed'
    | 'insufficient_data'
    | 'architecture_mismatch'
    | 'base_model_load_failed'
    | 'tokenizer_load_failed'
    | 'thread_panic'
    | 'training_failed'
    | 'state_lock';

export interface TrainingErrorPayload {
    code: TrainingErrorCode;
    message: string;
}

export function isTrainingErrorPayload(value: unknown): value is TrainingErrorPayload {
    return (
        typeof value === 'object' &&
        value !== null &&
        typeof (value as TrainingErrorPayload).code === 'string' &&
        typeof (value as TrainingErrorPayload).message === 'string'
    );
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
