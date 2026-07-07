use flate2::read::ZlibDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

const PERSONAS_ARCHIVE: &[u8] = include_bytes!("../../../resources/personas.bin");
const VOICES_ARCHIVE: &[u8] = include_bytes!("../../../resources/voices.bin");

#[derive(Debug)]
pub struct PersonaEntry {
    pub offset: usize,
    pub length: usize,
}

static PERSONA_INDEX: OnceLock<HashMap<String, PersonaEntry>> = OnceLock::new();

pub struct PersonaLoader;

impl PersonaLoader {
    fn init_index() -> HashMap<String, PersonaEntry> {
        parse_archive_index(PERSONAS_ARCHIVE)
    }

    fn get_index() -> &'static HashMap<String, PersonaEntry> {
        PERSONA_INDEX.get_or_init(Self::init_index)
    }

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

        let compressed_slice = &PERSONAS_ARCHIVE[entry.offset..entry.offset + entry.length];

        let mut decoder = ZlibDecoder::new(compressed_slice);
        let mut json_str = String::new();
        decoder
            .read_to_string(&mut json_str)
            .map_err(|e| format!("Zlib 압축 해제 실패: {}", e))?;

        let value: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON 역직렬화 실패: {}", e))?;

        Ok(value)
    }

    pub fn list_personas() -> Vec<String> {
        let index = Self::get_index();
        index.keys().cloned().collect()
    }
}

fn parse_archive_index(archive: &[u8]) -> HashMap<String, PersonaEntry> {
    let mut index = HashMap::new();
    if archive.len() < 4 {
        return index;
    }

    let file_count = u32::from_be_bytes([archive[0], archive[1], archive[2], archive[3]]) as usize;

    let mut pos = 4;
    for _ in 0..file_count {
        if pos >= archive.len() {
            break;
        }

        let name_len = archive[pos] as usize;
        pos += 1;

        if pos + name_len > archive.len() {
            break;
        }

        let name = match std::str::from_utf8(&archive[pos..pos + name_len]) {
            Ok(s) => s.to_string(),
            Err(_) => {
                pos += name_len;
                continue;
            }
        };
        pos += name_len;

        if pos + 8 > archive.len() {
            break;
        }

        let offset = u32::from_be_bytes([
            archive[pos],
            archive[pos + 1],
            archive[pos + 2],
            archive[pos + 3],
        ]) as usize;
        pos += 4;

        let length = u32::from_be_bytes([
            archive[pos],
            archive[pos + 1],
            archive[pos + 2],
            archive[pos + 3],
        ]) as usize;
        pos += 4;

        index.insert(name, PersonaEntry { offset, length });
    }

    index
}

static VOICE_INDEX: OnceLock<HashMap<String, PersonaEntry>> = OnceLock::new();

pub struct VoiceLoader;

impl VoiceLoader {
    fn get_index() -> &'static HashMap<String, PersonaEntry> {
        VOICE_INDEX.get_or_init(|| parse_archive_index(VOICES_ARCHIVE))
    }

    pub fn list_voices(persona_id: &str) -> Vec<String> {
        let prefix = format!("{}/", persona_id);
        let mut names: Vec<String> = Self::get_index()
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .map(|key| key[prefix.len()..].to_string())
            .collect();
        names.sort();
        names
    }

    pub fn load_voice(persona_id: &str, file_name: &str) -> Result<Vec<u8>, String> {
        let key = format!("{}/{}", persona_id, file_name);
        let index = Self::get_index();
        let entry = index
            .get(&key)
            .ok_or_else(|| format!("음성 데이터를 찾을 수 없습니다: {}", key))?;

        if entry.offset + entry.length > VOICES_ARCHIVE.len() {
            return Err("음성 아카이브 데이터 오프셋 손상 오류가 발생했습니다.".to_string());
        }

        let compressed_slice = &VOICES_ARCHIVE[entry.offset..entry.offset + entry.length];

        let mut decoder = ZlibDecoder::new(compressed_slice);
        let mut bytes = Vec::new();
        decoder
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Zlib 압축 해제 실패: {}", e))?;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::PersonaLoader;

    #[test]
    fn personas_archive_lists_embedded_entries() {
        assert_eq!(PersonaLoader::list_personas().len(), 99);
    }

    #[test]
    fn personas_archive_loads_zlib_compressed_json() {
        let value = PersonaLoader::load_persona("adrianne")
            .expect("embedded adrianne persona should decompress");

        assert_eq!(value["name_en"].as_str(), Some("Adrianne"));
        assert!(value["profile"].is_object());
        assert!(value["personality"].is_object());
    }
}
