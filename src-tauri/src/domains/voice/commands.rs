use crate::infrastructure::compress::VoiceLoader;
// [TTS 연동 보류] 합성 음성 품질 미흡으로 TTS 연동 보류 (2026-07-07). 재개 시 주석 해제.
// use crate::infrastructure::tts::TtsEngine;

#[tauri::command(rename_all = "snake_case")]
pub fn voice_list(persona_id: String) -> Vec<String> {
    VoiceLoader::list_voices(&persona_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn voice_get(persona_id: String, file_name: String) -> Result<Vec<u8>, String> {
    VoiceLoader::load_voice(&persona_id, &file_name)
}

/* [TTS 연동 보류] 합성 음성 품질 미흡으로 TTS 연동 보류 (2026-07-07). 재개 시 주석 해제.
#[tauri::command(rename_all = "snake_case")]
pub fn voice_synthesize(persona_id: String, text: String) -> Result<Vec<u8>, String> {
    let files = VoiceLoader::list_voices(&persona_id);
    let ref_file = files
        .first()
        .ok_or_else(|| format!("참조 음성이 없습니다: {}", persona_id))?;
    let ref_bytes = VoiceLoader::load_voice(&persona_id, ref_file)?;

    let temp_dir = std::env::temp_dir();
    let ref_ogg_path = temp_dir.join(format!("eversoul_ref_{}.ogg", persona_id));
    std::fs::write(&ref_ogg_path, &ref_bytes).map_err(|e| e.to_string())?;
    let ref_wav_path = temp_dir.join(format!("eversoul_ref_{}.wav", persona_id));
    let output_path = temp_dir.join(format!("eversoul_out_{}.wav", persona_id));

    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "프로젝트 루트 경로를 확인할 수 없습니다".to_string())?;
    let binary_path = project_root.join("third_party/qwen3-tts/build/qwen_tts.exe");
    let model_dir = project_root.join("third_party/qwen3-tts/qwen3-tts-0.6b-base");

    let engine = TtsEngine::new(binary_path, model_dir);
    engine.synthesize(&text, &ref_ogg_path, &ref_wav_path, &output_path)
}
*/
