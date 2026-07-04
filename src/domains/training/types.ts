export interface TrainingSummary {
    persona_id: string;
    examples_used: number;
    steps: number;
    final_loss: number;
    adapter_path: string;
}
export type TrainingError = {
    Database: string;
} | {
    InsufficientData: string;
} | {
    TrainingFailed: string;
};
