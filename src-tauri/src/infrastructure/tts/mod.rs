use std::path::{Path, PathBuf};
use std::process::Command;

const SYNTH_LANGUAGE: &str = "Korean";
const REF_SAMPLE_RATE: &str = "24000";

pub struct TtsEngine {
    binary_path: PathBuf,
    model_dir: PathBuf,
}

impl TtsEngine {
    pub fn new(binary_path: PathBuf, model_dir: PathBuf) -> Self {
        Self {
            binary_path,
            model_dir,
        }
    }

    pub fn synthesize(
        &self,
        text: &str,
        ref_ogg_path: &Path,
        ref_wav_path: &Path,
        output_path: &Path,
    ) -> Result<Vec<u8>, String> {
        if !self.binary_path.exists() {
            return Err(format!(
                "Qwen3-TTS 실행 파일이 없습니다: {}",
                self.binary_path.display()
            ));
        }
        if !self.model_dir.exists() {
            return Err(format!(
                "Qwen3-TTS 모델 디렉터리가 없습니다: {}",
                self.model_dir.display()
            ));
        }

        // 참조 음성(ogg)을 24kHz mono WAV로 변환 (Qwen3-TTS voice cloning 요구 형식)
        let convert = Command::new("ffmpeg")
            .arg("-y")
            .arg("-loglevel")
            .arg("error")
            .arg("-i")
            .arg(ref_ogg_path)
            .arg("-ar")
            .arg(REF_SAMPLE_RATE)
            .arg("-ac")
            .arg("1")
            .arg(ref_wav_path)
            .status()
            .map_err(|e| format!("ffmpeg 실행 실패(참조 음성 변환): {}", e))?;
        if !convert.success() {
            return Err(format!(
                "참조 음성 WAV 변환 실패: 종료 코드 {:?}",
                convert.code()
            ));
        }

        let status = Command::new(&self.binary_path)
            .arg("-d")
            .arg(&self.model_dir)
            .arg("--ref-audio")
            .arg(ref_wav_path)
            .arg("--text")
            .arg(text)
            .arg("-l")
            .arg(SYNTH_LANGUAGE)
            .arg("-o")
            .arg(output_path)
            .status()
            .map_err(|e| format!("Qwen3-TTS 실행 실패: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Qwen3-TTS 합성 실패: 종료 코드 {:?}",
                status.code()
            ));
        }

        std::fs::read(output_path).map_err(|e| format!("합성 결과 읽기 실패: {}", e))
    }
}
