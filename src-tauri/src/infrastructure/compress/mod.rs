use flate2::read::ZlibDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

const PERSONAS_ARCHIVE: &[u8] = include_bytes!("../../../resources/personas.bin");

#[derive(Debug)]
pub struct PersonaEntry {
    pub offset: usize,
    pub length: usize,
}

static PERSONA_INDEX: OnceLock<HashMap<String, PersonaEntry>> = OnceLock::new();

pub struct PersonaLoader;

impl PersonaLoader {
    fn init_index() -> HashMap<String, PersonaEntry> {
        let mut index = HashMap::new();
        if PERSONAS_ARCHIVE.len() < 4 {
            return index;
        }

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

            let name_len = PERSONAS_ARCHIVE[pos] as usize;
            pos += 1;

            if pos + name_len > PERSONAS_ARCHIVE.len() {
                break;
            }

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

            let offset = u32::from_be_bytes([
                PERSONAS_ARCHIVE[pos],
                PERSONAS_ARCHIVE[pos + 1],
                PERSONAS_ARCHIVE[pos + 2],
                PERSONAS_ARCHIVE[pos + 3],
            ]) as usize;
            pos += 4;

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

#[cfg(test)]
mod tests {
    use super::PersonaLoader;

    #[test]
    fn personas_archive_lists_embedded_entries() {
        assert_eq!(PersonaLoader::list_personas().len(), 95);
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
