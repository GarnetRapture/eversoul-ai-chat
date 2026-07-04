use flate2::read::DeflateDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

// 빌드 타임에 압축 아카이브 바이너리를 컴파일 결과물 내부로 직접 임베딩
const PERSONAS_ARCHIVE: &[u8] = include_bytes!("../../../resources/personas.bin");

#[derive(Debug)]
pub struct PersonaEntry {
    pub offset: usize,
    pub length: usize,
}

// 런타임에 싱글톤 형태로 유지되는 인덱스 맵
static PERSONA_INDEX: OnceLock<HashMap<String, PersonaEntry>> = OnceLock::new();

pub struct PersonaLoader;

impl PersonaLoader {
    /// 인덱스 헤더를 최초 1회 파싱하여 메모리에 적재
    fn init_index() -> HashMap<String, PersonaEntry> {
        let mut index = HashMap::new();
        if PERSONAS_ARCHIVE.len() < 4 {
            return index;
        }

        // 1. 전체 파일 수 획득 (4바이트 BigEndian)
        let file_count = u32::from_be_bytes([
            PERSONAS_ARCHIVE[0],
            PERSONAS_ARCHIVE[1],
            PERSONAS_ARCHIVE[2],
            PERSONAS_ARCHIVE[3],
        ]) as usize;

        let mut pos = 4;
        for _ in 0..file_count {
            if pos >= PERSONAS_ARCHIVE.len() {
                break;
            }

            // 2. 이름 문자열 길이 획득 (1바이트)
            let name_len = PERSONAS_ARCHIVE[pos] as usize;
            pos += 1;

            if pos + name_len > PERSONAS_ARCHIVE.len() {
                break;
            }

            // 3. 이름 문자열 파싱 (UTF-8)
            let name = match std::str::from_utf8(&PERSONAS_ARCHIVE[pos..pos + name_len]) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    pos += name_len;
                    continue;
                }
            };
            pos += name_len;

            if pos + 8 > PERSONAS_ARCHIVE.len() {
                break;
            }

            // 4. 오프셋 획득 (4바이트 BigEndian)
            let offset = u32::from_be_bytes([
                PERSONAS_ARCHIVE[pos],
                PERSONAS_ARCHIVE[pos + 1],
                PERSONAS_ARCHIVE[pos + 2],
                PERSONAS_ARCHIVE[pos + 3],
            ]) as usize;
            pos += 4;

            // 5. 압축 데이터 크기 획득 (4바이트 BigEndian)
            let length = u32::from_be_bytes([
                PERSONAS_ARCHIVE[pos],
                PERSONAS_ARCHIVE[pos + 1],
                PERSONAS_ARCHIVE[pos + 2],
                PERSONAS_ARCHIVE[pos + 3],
            ]) as usize;
            pos += 4;

            index.insert(name, PersonaEntry { offset, length });
        }

        index
    }

    /// 인덱스 맵 참조 반환 (싱글톤)
    fn get_index() -> &'static HashMap<String, PersonaEntry> {
        PERSONA_INDEX.get_or_init(Self::init_index)
    }

    /// 특정 정령의 영어명을 기준으로 바이너리 아카이브에서 데이터를 찾아 온디맨드 부분 압축 해제
    pub fn load_persona(name_en: &str) -> Result<serde_json::Value, String> {
        let index = Self::get_index();
        let name_lower = name_en
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "");

        let entry = index
            .get(&name_lower)
            .ok_or_else(|| format!("페르소나 데이터를 찾을 수 없습니다: {}", name_en))?;

        if entry.offset + entry.length > PERSONAS_ARCHIVE.len() {
            return Err("아카이브 데이터 오프셋 손상 오류가 발생했습니다.".to_string());
        }

        // 해당 세그먼트 바이트 데이터 획득
        let compressed_slice = &PERSONAS_ARCHIVE[entry.offset..entry.offset + entry.length];

        // Deflate 압축 온디맨드 해제
        let mut decoder = DeflateDecoder::new(compressed_slice);
        let mut json_str = String::new();
        decoder
            .read_to_string(&mut json_str)
            .map_err(|e| format!("Deflate 압축 해제 실패: {}", e))?;

        // JSON 파싱
        let value: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON 역직렬화 실패: {}", e))?;

        Ok(value)
    }

    /// 아카이브에 포함된 모든 정령 영어명 리스트 반환
    pub fn list_personas() -> Vec<String> {
        let index = Self::get_index();
        index.keys().cloned().collect()
    }
}
