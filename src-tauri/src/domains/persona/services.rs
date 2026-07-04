use super::repositories::PersonaRepository;
use super::types::{PersonaConfig, PersonaError, UpdatePersonaRequest};
use crate::infrastructure::compress::PersonaLoader;
use rusqlite::Connection;
use std::time::SystemTime;

pub struct PersonaService<'a> {
    conn: &'a Connection,
}

impl<'a> PersonaService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// 특정 캐릭터(페르소나)의 설정값을 수정하여 로컬 SQLite에 반영한다.
    pub fn update_persona_settings(&self, req: UpdatePersonaRequest) -> Result<(), PersonaError> {
        let existing = PersonaRepository::get_persona(self.conn, &req.id)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        if let Some(mut config) = existing {
            config.system_prompt = req.system_prompt;
            config.greeting = req.greeting;
            PersonaRepository::save_persona(self.conn, &config)
                .map_err(|e| PersonaError::Database(e.to_string()))
        } else {
            Err(PersonaError::NotFound(req.id))
        }
    }

    /// 완벽 압축 아카이브에서 정령 데이터를 풀고 프로필 지침을 동적 조립하여 로컬 DB에 프리셋으로 저장(Upsert)한다.
    pub fn load_and_save_preset(&self, name_en: &str) -> Result<PersonaConfig, PersonaError> {
        // 1. 아카이브 바이너리로부터 정령 JSON 데이터 온디맨드 해제
        let json_val =
            PersonaLoader::load_persona(name_en).map_err(|e| PersonaError::Archive(e))?;

        // 2. JSON 데이터로부터 필드 안전하게 획득
        let name = json_val["name"].as_str().unwrap_or(name_en);
        let name_en_val = json_val["name_en"].as_str().unwrap_or(name_en);
        let grade = json_val["grade"].as_str().unwrap_or("-");
        let race = json_val["race"].as_str().unwrap_or("-");
        let class = json_val["class"].as_str().unwrap_or("-");
        let sub_class = json_val["sub_class"].as_str().unwrap_or("-");
        let stat = json_val["stat"].as_str().unwrap_or("-");

        let nick_name = json_val["profile"]["nick_name"].as_str().unwrap_or("-");
        let constellation = json_val["profile"]["constellation"].as_str().unwrap_or("-");
        let union = json_val["profile"]["union"].as_str().unwrap_or("-");
        let birthday = json_val["profile"]["birthday"].as_str().unwrap_or("-");

        let height = json_val["profile"]["height"]
            .as_f64()
            .map(|h| format!("{}cm", h))
            .unwrap_or_else(|| "-".to_string());
        let weight = json_val["profile"]["weight"]
            .as_f64()
            .map(|w| format!("{}kg", w))
            .unwrap_or_else(|| "-".to_string());

        let cv_ko = json_val["profile"]["cv_ko"].as_str().unwrap_or("-");
        let cv_jp = json_val["profile"]["cv_jp"].as_str().unwrap_or("-");

        // 배열 데이터 스트링 조합
        let parse_array = |field_key: &str| -> String {
            json_val["profile"][field_key]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "-".to_string())
        };

        let likes = parse_array("like");
        let dislikes = parse_array("dislike");
        let hobby = parse_array("hobby");
        let speciality = parse_array("speciality");

        let description = json_val["personality"]["description"]
            .as_str()
            .unwrap_or("-");
        let greeting = json_val["personality"]["greeting"]
            .as_str()
            .unwrap_or("안녕!");

        // 말투 패턴 예시 중 대표 3개 추출
        let mut speech_lines = Vec::new();
        if let Some(patterns) = json_val["speech_patterns"].as_array() {
            for p in patterns.iter().take(3) {
                if let Some(s) = p.as_str() {
                    speech_lines.push(format!("- \"{}\"", s));
                }
            }
        }
        let speech_patterns_str = if speech_lines.is_empty() {
            "- \"안녕, 구원자님!\"".to_string()
        } else {
            speech_lines.join("\n")
        };

        // 3. 인연채팅 시스템 프롬프트 수칙 고도화 빌딩
        let system_prompt = format!(
            "당신은 다음 프로필을 가진 정령 캐릭터 역할을 맡아 구원자와 대화해야 합니다.
반드시 지정된 성격 특징과 대표 대사 말투(종결어미)를 100% 철저히 모사하고 캐릭터 성격을 유지하십시오.

[정령 신체 및 프로필 정보]
- 이름: {} ({})
- 별칭: {}
- 등급/종족/클래스: {} / {} / {} ({}) / {} 계열
- 별자리/소속: {} / {}
- 생일/신체: {} / 키: {}, 몸무게: {}
- 성우: 한국어 - {} / 일본어 - {}
- 좋아하는 것: {}
- 싫어하는 것: {}
- 취미 / 특기: {} / {}

[성격 특징 묘사]
{}

[대표 대사 말투 예시 (Few-shot)]
{}
",
            name, name_en_val, nick_name, grade, race, class, sub_class, stat, constellation, union, birthday, height, weight, cv_ko, cv_jp, likes, dislikes, hobby, speciality, description, speech_patterns_str
        );

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        // 영어명을 고유 ID로 매핑하여 Upsert 처리 (예: "aki")
        let persona_id = name_en_val
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "");

        let config = PersonaConfig {
            id: persona_id,
            name: name.to_string(),
            name_en: name_en_val.to_string(),
            grade: grade.to_string(),
            race: race.to_string(),
            class: class.to_string(),
            sub_class: sub_class.to_string(),
            system_prompt,
            greeting: greeting.to_string(),
            raw_json: json_val.to_string(),
            created_at: now,
        };

        // SQLite DB 저장소에 페르소나 설정 영속화
        PersonaRepository::save_persona(self.conn, &config)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        Ok(config)
    }

    /// LLM 추론을 위한 시스템 프롬프트를 구성하여 반환한다.
    pub fn get_assembled_system_prompt(&self, id: &str) -> Result<String, String> {
        let persona = PersonaRepository::get_persona(self.conn, id).map_err(|e| e.to_string())?;

        if let Some(p) = persona {
            let prompt = format!(
                "You are {}. 아래 지침과 캐릭터 프로필 및 어투 가이드를 참고하여 인연채팅 대화에 임하십시오.\n\n{}\n\n",
                p.name, p.system_prompt
            );
            Ok(prompt)
        } else {
            Err(format!("페르소나 정보를 찾을 수 없습니다: {}", id))
        }
    }

    /// 사용 가능한 모든 캐릭터 목록을 획득한다.
    pub fn get_available_personas(&self) -> Result<Vec<PersonaConfig>, PersonaError> {
        let existing = PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        if !existing.is_empty() {
            return Ok(existing);
        }

        let mut names = PersonaLoader::list_personas();
        names.sort();

        for name in names {
            self.load_and_save_preset(&name)?;
        }

        PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))
    }
}
