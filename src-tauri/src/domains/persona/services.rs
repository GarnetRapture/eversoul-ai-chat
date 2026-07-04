use super::repositories::PersonaRepository;
use super::types::{BondRankingEntry, PersonaConfig, PersonaError, UpdatePersonaRequest};
use crate::domains::chat::repositories::ChatRepository;
use crate::infrastructure::compress::PersonaLoader;
use crate::infrastructure::settings::SettingsManager;
use rusqlite::Connection;
use std::time::SystemTime;

pub struct PersonaService<'a> {
    conn: &'a Connection,
}

impl<'a> PersonaService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

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

    pub fn load_and_save_preset(&self, name_en: &str) -> Result<PersonaConfig, PersonaError> {

        let json_val =
            PersonaLoader::load_persona(name_en).map_err(|e| PersonaError::Archive(e))?;

        let name = json_val["name"].as_str().unwrap_or(name_en);
        let name_en_val = json_val["name_en"].as_str().unwrap_or(name_en);
        let grade = json_val["grade"].as_str().unwrap_or("-");
        let race = json_val["race"].as_str().unwrap_or("-");
        let class = json_val["class"].as_str().unwrap_or("-");
        let sub_class = json_val["sub_class"].as_str().unwrap_or("-");
        let stat = json_val["stat"].as_str().unwrap_or("-");

        let nick_name = json_val["profile"]["nick_name"].as_str().unwrap_or("-");

        let constellation_raw = json_val["profile"]["constellation"].as_str().unwrap_or("-");
        let constellation = if constellation_raw.ends_with("자리") {
            constellation_raw
        } else {
            "-"
        };

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
        let greeting = json_val["personality"]["greeting"].as_str().unwrap_or("");

        let mut speech_lines = Vec::new();
        let mut seen_speech: std::collections::HashSet<&str> = std::collections::HashSet::new();
        if let Some(patterns) = json_val["speech_patterns"].as_array() {
            for p in patterns.iter() {
                if speech_lines.len() >= 12 {
                    break;
                }
                if let Some(s) = p.as_str() {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() && seen_speech.insert(trimmed) {
                        speech_lines.push(format!("- \"{}\"", trimmed));
                    }
                }
            }
        }
        let speech_patterns_str = if speech_lines.is_empty() {
            "-".to_string()
        } else {
            speech_lines.join("\n")
        };

        let dialogue_sample = Self::extract_dialogue_sample(&json_val["dialogues"]["evertalk"], 16);

        let mut comment_lines = Vec::new();
        if let Some(comments) = json_val["comments"].as_array() {
            for c in comments.iter() {
                let writer = c["writer"].as_str().unwrap_or("").trim();
                let comment = c["comment"].as_str().unwrap_or("").trim();
                if !writer.is_empty() && !comment.is_empty() {
                    comment_lines.push(format!("- {}: \"{}\"", writer, comment));
                }
            }
        }
        let comments_str = if comment_lines.is_empty() {
            "-".to_string()
        } else {
            comment_lines.join("\n")
        };

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

[대표 대사 말투 예시]
{}

[실제 에버톡 대화 예시 - 이 캐릭터가 구원자와 실제로 나눈 대화 흐름을 그대로 참고할 것]
{}

[다른 정령들이 보는 이 캐릭터]
{}
",
            name, name_en_val, nick_name, grade, race, class, sub_class, stat, constellation, union, birthday, height, weight, cv_ko, cv_jp, likes, dislikes, hobby, speciality, description, speech_patterns_str, dialogue_sample, comments_str
        );

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

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

        PersonaRepository::save_persona(self.conn, &config)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        Ok(config)
    }

    fn extract_dialogue_sample(evertalk: &serde_json::Value, limit: usize) -> String {
        let Some(entries) = evertalk.as_array() else {
            return "-".to_string();
        };

        let mut lines: Vec<String> = Vec::new();
        let mut last: Option<(String, String)> = None;

        for entry in entries {
            let speaker = entry["speaker"].as_str().unwrap_or("").trim();
            let message = entry["message"].as_str().unwrap_or("").trim();

            if speaker.is_empty() || message.is_empty() || !message.chars().any(|c| c.is_alphanumeric())
            {
                continue;
            }

            let current = (speaker.to_string(), message.to_string());
            if last.as_ref() == Some(&current) {
                continue;
            }
            last = Some(current.clone());

            lines.push(format!("{}: {}", current.0, current.1));
            if lines.len() >= limit {
                break;
            }
        }

        if lines.is_empty() {
            "-".to_string()
        } else {
            lines.join("\n")
        }
    }

    pub fn get_assembled_system_prompt(&self, id: &str) -> Result<String, String> {
        let persona = PersonaRepository::get_persona(self.conn, id).map_err(|e| e.to_string())?;

        if let Some(p) = persona {
            let prompt = format!(
                "You are {}. 아래 지침과 캐릭터 프로필 및 어투 가이드를 참고하여 인연채팅 대화에 임하십시오.\n\n{}\n\n\
                [대화 상대 호칭 규칙 - 반드시 준수]\n\
                - 지금 대화하는 상대는 이 세계관의 유일한 플레이어인 '구원자'입니다.\n\
                - 상대를 부를 때는 반드시 '구원자님'이라고만 호칭하십시오.\n\
                - 대화 내용, 지식 데이터, 자기 자신의 이름 등 어디에서 유래했든 '구원자' 이외의\n\
                  임의의 사람 이름(예: 다른 정령의 이름, 존재하지 않는 이름 등)을 상대의 이름으로\n\
                  지어내거나 사용하는 것을 절대 금지합니다.\n\n",
                p.name, p.system_prompt
            );
            Ok(prompt)
        } else {
            Err(format!("페르소나 정보를 찾을 수 없습니다: {}", id))
        }
    }

    pub fn get_available_personas(&self) -> Result<Vec<PersonaConfig>, PersonaError> {
        let existing = PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        if !existing.is_empty() {
            return Ok(existing);
        }

        let mut names = PersonaLoader::list_personas();
        names.sort();

        for name in names {
            if let Err(err) = self.load_and_save_preset(&name) {
                eprintln!("페르소나 프리셋 수립 실패: {err}");
            }
        }

        PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))
    }

    pub fn set_default_persona_id(
        &self,
        settings: &SettingsManager,
        id: &str,
    ) -> Result<String, PersonaError> {
        let existing = PersonaRepository::get_persona(self.conn, id)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        if existing.is_none() {
            return Err(PersonaError::NotFound(id.to_string()));
        }

        settings
            .set_default_persona_id(id)
            .map_err(|e| PersonaError::Unknown(e.to_string()))?;
        Ok(id.to_string())
    }

    pub fn get_bond_ranking(&self) -> Result<Vec<BondRankingEntry>, PersonaError> {
        let personas = PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;
        let message_counts = ChatRepository::count_messages_by_persona(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;
        let memory_counts = ChatRepository::count_episodic_memories_by_persona(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        let mut entries: Vec<BondRankingEntry> = personas
            .into_iter()
            .filter_map(|p| {
                let message_count = *message_counts.get(&p.id).unwrap_or(&0);
                let memory_count = *memory_counts.get(&p.id).unwrap_or(&0);
                if message_count == 0 && memory_count == 0 {
                    return None;
                }

                let bond_score = message_count + memory_count * 3;
                Some(BondRankingEntry {
                    persona_id: p.id,
                    name: p.name,
                    name_en: p.name_en,
                    message_count,
                    memory_count,
                    bond_score,
                })
            })
            .collect();

        entries.sort_by(|a, b| b.bond_score.cmp(&a.bond_score));
        Ok(entries)
    }
}
