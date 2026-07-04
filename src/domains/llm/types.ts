export interface LlmStatus {
    is_loaded: boolean;
    model_path: string | null;
    error_message: string | null;
}
export interface LlmInferResponse {
    text: string;
    time_taken_ms: number;
}
export type LlmError = {
    ModelFileNotFound: {
        path: string;
    };
} | {
    BackendInit: string;
} | {
    ModelLoad: string;
} | {
    ContextCreate: string;
} | {
    Tokenize: string;
} | {
    Infer: string;
} | 'EngineNotLoaded' | {
    Unknown: string;
};
