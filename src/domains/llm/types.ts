export interface LlmStatus {
    is_loaded: boolean;
    model_path: string | null;
    error_message: string | null;
}
export interface ModelDownloadProgress {
    downloaded_bytes: number;
    total_bytes: number;
    ratio: number;
    done: boolean;
}
export interface LlmInferResponse {
    text: string;
    time_taken_ms: number;
}
export interface LlmStreamInferRequest {
    request_id: string;
    prompt: string;
    max_tokens?: number;
    persona_id?: string | null;
    token_event: string;
    done_event: string;
}
export interface LlmModelValidation {
    path: string;
    size_bytes: number;
    sha256: string;
    sidecar_sha256: string | null;
    hash_matches_sidecar: boolean | null;
}
export interface LlmRequestStatus {
    request_id: string;
    persona_id: string | null;
    state: string;
    prompt_tokens: number;
    generated_tokens: number;
    reused_prefix_tokens: number;
    truncated_prompt_tokens: number;
    cache_reset: boolean;
    error_message: string | null;
}
export interface LlmSessionGenerationStats {
    prompt_tokens: number;
    cached_tokens: number;
    generated_tokens: number;
    reused_prefix_tokens: number;
    truncated_prompt_tokens: number;
    cache_reset: boolean;
}
export interface LlmSessionStatus {
    persona_id: string;
    cached_tokens: number;
    lora_adapter_mounted: boolean;
    last_access: number;
    last_generation: LlmSessionGenerationStats | null;
}
export type LlmError = {
    ModelFileNotFound: {
        path: string;
    };
} | {
    ModelDownload: string;
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
