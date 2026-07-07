use super::repositories::PersonaRepository;
use super::types::{
    BondRankingEntry, FamiliarityEntry, PersonaConfig, PersonaError, PersonaLocalizedPrompt,
    UpdatePersonaRequest,
};
use crate::domains::chat::repositories::ChatRepository;
use crate::infrastructure::compress::PersonaLoader;
use crate::infrastructure::settings::SettingsManager;
use rusqlite::Connection;
use serde_json::Value;
use std::time::SystemTime;

const LOCALIZED_PROMPT_CACHE_VERSION: &str = "prompt-v2";

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

[말투 참고용 대화 예시 - 아래는 이 캐릭터의 어투/종결어미를 파악하기 위한 참고 자료일 뿐,
실제로 지금 나누는 대화가 아닙니다. 이 예시에 등장하는 상황, 사건, 대사 내용 자체를
반복하거나 인용하지 마십시오. 오직 말투와 어조만 참고하십시오.]
{}

[다른 정령들이 보는 이 캐릭터]
{}

[응답 태도 - 반드시 준수]
- 지금 구원자가 실제로 입력한 메시지의 내용에만 집중하여, 그에 맞는 응답을 새로 생성하십시오.
- 위 대화 예시의 내용/사건/대사를 그대로 반복하거나 재사용하지 마십시오.
- 캐릭터의 성격과 말투는 유지하되, 구원자의 말이나 제안을 근거 없이 거절, 부정, 무시하지 말고
  가능한 한 구원자의 뜻에 협조적으로 호응하며 대화를 이어가십시오.
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

            if speaker.is_empty()
                || message.is_empty()
                || !message.chars().any(|c| c.is_alphanumeric())
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

    fn supported_language(language: &str) -> &str {
        match language {
            "en" => "en",
            "zh_cn" => "zh_cn",
            _ => "ko",
        }
    }

    fn language_instruction(language: &str) -> &'static str {
        match Self::supported_language(language) {
            "en" => "Respond in English. Keep the character role, relationship terms, and tone consistent with the English EverTalk data.",
            "zh_cn" => "请使用简体中文回复。保持角色设定、称呼关系和语气，并以中文 EverTalk 数据为准。",
            _ => "반드시 한국어로 답변하십시오. 캐릭터 설정, 호칭 관계, 말투는 한국어 에버톡 데이터를 기준으로 유지하십시오.",
        }
    }

    fn localized_text(value: &Value, language: &str) -> String {
        let selected = Self::supported_language(language);
        value[selected]
            .as_str()
            .or_else(|| value["ko"].as_str())
            .or_else(|| value["en"].as_str())
            .or_else(|| value["zh_tw"].as_str())
            .unwrap_or("")
            .to_string()
    }

    fn localized_profile_list(value: &Value, language: &str) -> String {
        let selected = Self::supported_language(language);
        value[selected]
            .as_array()
            .or_else(|| value["ko"].as_array())
            .or_else(|| value["en"].as_array())
            .or_else(|| value["zh_tw"].as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .filter(|item| !item.trim().is_empty())
                    .collect::<Vec<&str>>()
                    .join(", ")
            })
            .filter(|items| !items.is_empty())
            .unwrap_or_else(|| "-".to_string())
    }

    fn localized_dialogue_sample(value: &Value, language: &str, limit: usize) -> String {
        let Some(entries) = value.as_array() else {
            return "-".to_string();
        };

        let selected = Self::supported_language(language);
        let mut lines = Vec::new();
        let mut last: Option<(String, String)> = None;

        for entry in entries {
            let candidate = entry[selected]
                .as_object()
                .or_else(|| entry["ko"].as_object())
                .or_else(|| entry["en"].as_object())
                .or_else(|| entry["zh_tw"].as_object());
            let Some(dialogue) = candidate else {
                continue;
            };
            let speaker = dialogue
                .get("speaker")
                .and_then(|speaker| speaker.as_str())
                .unwrap_or("")
                .trim();
            let message = dialogue
                .get("message")
                .and_then(|message| message.as_str())
                .unwrap_or("")
                .trim();

            if speaker.is_empty()
                || message.is_empty()
                || !message.chars().any(|c| c.is_alphanumeric())
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

    fn localized_speech_patterns(value: &Value, language: &str, limit: usize) -> String {
        let Some(entries) = value.as_array() else {
            return "-".to_string();
        };

        let selected = Self::supported_language(language);
        let mut lines = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for entry in entries {
            let message = entry[selected]["message"]
                .as_str()
                .or_else(|| entry["ko"]["message"].as_str())
                .or_else(|| entry["en"]["message"].as_str())
                .or_else(|| entry["zh_tw"]["message"].as_str())
                .unwrap_or("")
                .trim();
            if message.is_empty() || !seen.insert(message.to_string()) {
                continue;
            }
            lines.push(format!("- \"{}\"", message));
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

    fn localized_comments(value: &Value, language: &str) -> String {
        let Some(entries) = value.as_array() else {
            return "-".to_string();
        };

        let selected = Self::supported_language(language);
        let mut lines = Vec::new();
        for entry in entries {
            let candidate = entry[selected]
                .as_object()
                .or_else(|| entry["ko"].as_object())
                .or_else(|| entry["en"].as_object())
                .or_else(|| entry["zh_tw"].as_object());
            let Some(comment) = candidate else {
                continue;
            };
            let writer = comment
                .get("speaker")
                .and_then(|speaker| speaker.as_str())
                .unwrap_or("")
                .trim();
            let message = comment
                .get("message")
                .and_then(|message| message.as_str())
                .unwrap_or("")
                .trim();
            if !writer.is_empty() && !message.is_empty() {
                lines.push(format!("- {}: \"{}\"", writer, message));
            }
        }

        if lines.is_empty() {
            "-".to_string()
        } else {
            lines.join("\n")
        }
    }

    fn build_localized_system_prompt(persona: &PersonaConfig, language: &str) -> (String, String) {
        let parsed = serde_json::from_str::<Value>(&persona.raw_json).unwrap_or(Value::Null);
        let i18n = &parsed["i18n"];
        if i18n.is_null() {
            return (persona.name.clone(), persona.system_prompt.clone());
        }

        let name = Self::localized_text(&i18n["name"], language);
        let grade = Self::localized_text(&i18n["grade"], language);
        let race = Self::localized_text(&i18n["race"], language);
        let class = Self::localized_text(&i18n["class"], language);
        let sub_class = Self::localized_text(&i18n["sub_class"], language);
        let stat = Self::localized_text(&i18n["stat"], language);
        let profile = &i18n["profile"];
        let personality = &i18n["personality"];
        let birthday = parsed["profile"]["birthday"].as_str().unwrap_or("-");
        let height = parsed["profile"]["height"]
            .as_f64()
            .map(|h| format!("{}cm", h))
            .unwrap_or_else(|| "-".to_string());
        let weight = parsed["profile"]["weight"]
            .as_f64()
            .map(|w| format!("{}kg", w))
            .unwrap_or_else(|| "-".to_string());
        let speech_patterns =
            Self::localized_speech_patterns(&i18n["speech_patterns"], language, 12);
        let dialogue_sample =
            Self::localized_dialogue_sample(&i18n["dialogues"]["evertalk"], language, 16);
        let comments = Self::localized_comments(&i18n["comments"], language);

        let nick_name = Self::localized_text(&profile["nick_name"], language);
        let constellation = Self::localized_text(&profile["constellation"], language);
        let union = Self::localized_text(&profile["union"], language);
        let cv_ko = Self::localized_text(&profile["cv_ko"], language);
        let cv_jp = Self::localized_text(&profile["cv_jp"], language);
        let like = Self::localized_profile_list(&profile["like"], language);
        let dislike = Self::localized_profile_list(&profile["dislike"], language);
        let hobby = Self::localized_profile_list(&profile["hobby"], language);
        let speciality = Self::localized_profile_list(&profile["speciality"], language);
        let description = Self::localized_text(&personality["description"], language);
        let instruction = Self::language_instruction(language);

        let body = match Self::supported_language(language) {
            "en" => format!(
                "You must play the role of a spirit character with the profile below and talk with the Savior.
{instruction}

[Spirit Body & Profile Information]
- Name: {name} ({name_en})
- Nickname: {nick_name}
- Grade/Race/Class: {grade} / {race} / {class} ({sub_class}) / {stat} stat
- Constellation/Union: {constellation} / {union}
- Birthday/Body: {birthday} / Height: {height}, Weight: {weight}
- Voice actor: Korean - {cv_ko} / Japanese - {cv_jp}
- Likes: {like}
- Dislikes: {dislike}
- Hobby / Speciality: {hobby} / {speciality}

[Personality Description]
{description}

[Representative Speech-Pattern Examples]
{speech_patterns}

[Reference Dialogue Examples for Tone Only]
{dialogue_sample}

[How Other Spirits See This Character]
{comments}

[Response Attitude - Must Follow]
- Focus only on the content of the message the Savior actually just typed, and generate a new response that fits it.
- Do not repeat or reuse the content/events/lines from the dialogue examples above as-is.
- Keep the character's personality and tone, but do not refuse, deny, or ignore the Savior's words or suggestions without basis - stay cooperative with the Savior's intent and keep the conversation going.
",
                instruction = instruction,
                name = name,
                name_en = persona.name_en,
                nick_name = nick_name,
                grade = grade,
                race = race,
                class = class,
                sub_class = sub_class,
                stat = stat,
                constellation = constellation,
                union = union,
                birthday = birthday,
                height = height,
                weight = weight,
                cv_ko = cv_ko,
                cv_jp = cv_jp,
                like = like,
                dislike = dislike,
                hobby = hobby,
                speciality = speciality,
                description = description,
                speech_patterns = speech_patterns,
                dialogue_sample = dialogue_sample,
                comments = comments
            ),
            "zh_cn" => format!(
                "你必须扮演一个具有以下资料的精灵角色，与救世主对话。
{instruction}

[精灵身体及资料信息]
- 名字：{name}（{name_en}）
- 昵称：{nick_name}
- 等级/种族/职业：{grade} / {race} / {class}（{sub_class}）/ {stat} 属性
- 星座/所属：{constellation} / {union}
- 生日/身材：{birthday} / 身高：{height}，体重：{weight}
- 声优：韩语 - {cv_ko} / 日语 - {cv_jp}
- 喜欢的：{like}
- 讨厌的：{dislike}
- 兴趣 / 特长：{hobby} / {speciality}

[性格特征描述]
{description}

[代表性台词语气示例]
{speech_patterns}

[仅供参考语气用的对话示例]
{dialogue_sample}

[其他精灵眼中的这个角色]
{comments}

[回应态度 - 必须遵守]
- 只专注于救世主刚刚实际输入的消息内容，生成与之相符的新回应。
- 不要原样重复或再次使用上面对话示例中的内容/事件/台词。
- 保持角色的性格与语气，但不要无理由地拒绝、否认或无视救世主的话语或提议——请配合救世主的意图，继续对话。
",
                instruction = instruction,
                name = name,
                name_en = persona.name_en,
                nick_name = nick_name,
                grade = grade,
                race = race,
                class = class,
                sub_class = sub_class,
                stat = stat,
                constellation = constellation,
                union = union,
                birthday = birthday,
                height = height,
                weight = weight,
                cv_ko = cv_ko,
                cv_jp = cv_jp,
                like = like,
                dislike = dislike,
                hobby = hobby,
                speciality = speciality,
                description = description,
                speech_patterns = speech_patterns,
                dialogue_sample = dialogue_sample,
                comments = comments
            ),
            _ => format!(
                "당신은 다음 프로필을 가진 정령 캐릭터 역할을 맡아 구원자와 대화해야 합니다.
{instruction}

[정령 신체 및 프로필 정보]
- 이름: {name} ({name_en})
- 별칭: {nick_name}
- 등급/종족/클래스: {grade} / {race} / {class} ({sub_class}) / {stat} 계열
- 별자리/소속: {constellation} / {union}
- 생일/신체: {birthday} / 키: {height}, 몸무게: {weight}
- 성우: 한국어 - {cv_ko} / 일본어 - {cv_jp}
- 좋아하는 것: {like}
- 싫어하는 것: {dislike}
- 취미 / 특기: {hobby} / {speciality}

[성격 특징 묘사]
{description}

[대표 대사 말투 예시]
{speech_patterns}

[말투 참고용 대화 예시]
{dialogue_sample}

[다른 정령들이 보는 이 캐릭터]
{comments}

[응답 태도 - 반드시 준수]
- 지금 구원자가 실제로 입력한 메시지의 내용에만 집중하여, 그에 맞는 응답을 새로 생성하십시오.
- 위 대화 예시의 내용/사건/대사를 그대로 반복하거나 재사용하지 마십시오.
- 캐릭터의 성격과 말투는 유지하되, 구원자의 말이나 제안을 근거 없이 거절, 부정, 무시하지 말고 가능한 한 구원자의 뜻에 협조적으로 호응하며 대화를 이어가십시오.
",
                instruction = instruction,
                name = name,
                name_en = persona.name_en,
                nick_name = nick_name,
                grade = grade,
                race = race,
                class = class,
                sub_class = sub_class,
                stat = stat,
                constellation = constellation,
                union = union,
                birthday = birthday,
                height = height,
                weight = weight,
                cv_ko = cv_ko,
                cv_jp = cv_jp,
                like = like,
                dislike = dislike,
                hobby = hobby,
                speciality = speciality,
                description = description,
                speech_patterns = speech_patterns,
                dialogue_sample = dialogue_sample,
                comments = comments
            ),
        };

        (name, body)
    }

    pub fn get_assembled_system_prompt(&self, id: &str, language: &str) -> Result<String, String> {
        let persona = PersonaRepository::get_persona(self.conn, id).map_err(|e| e.to_string())?;

        if let Some(p) = persona {
            let normalized_language = Self::supported_language(language).to_string();
            let prompt_source_key = Self::localized_prompt_source_key(&p.created_at);
            let cached = PersonaRepository::get_localized_prompt(
                self.conn,
                &p.id,
                &normalized_language,
                &prompt_source_key,
            )
            .map_err(|e| e.to_string())?;

            if let Some(entry) = cached {
                return Ok(entry.assembled_prompt);
            }

            let prompt = Self::assemble_and_cache_prompt(self.conn, &p, &normalized_language)?;
            return Ok(prompt);
        }

        Err(format!("페르소나 정보를 찾을 수 없습니다: {}", id))
    }

    fn assemble_and_cache_prompt(
        conn: &Connection,
        p: &PersonaConfig,
        normalized_language: &str,
    ) -> Result<String, String> {
        let (localized_name, localized_prompt) =
            Self::build_localized_system_prompt(p, normalized_language);
        let prompt = match normalized_language {
                "en" => format!(
                    "You are {name}. Follow the guidelines, character profile, and tone guide below for this bond-chat conversation.\n\n{body}\n\n\
                    [Name & Addressing Rules - Must Follow]\n\
                    - Your name is always exactly \"{name}\", precisely as given here. When introducing or referring\n\
                      to yourself, never invent, translate, transliterate, or substitute any other name - always use\n\
                      \"{name}\" verbatim.\n\
                    - The person you are talking to right now is the one and only player in this world, the 'Savior'.\n\
                    - Always address them only as 'Savior'.\n\
                    - Regardless of where it might come from (conversation content, knowledge data, your own name, etc.),\n\
                      never invent or use any name other than 'Savior' as the name of the person you are\n\
                      talking to (e.g. another spirit's name or a name that doesn't exist).\n\
                    - Do not write stage directions, action tags, or emotion labels surrounded by asterisks.\n\
                    - Do not invent profile facts. If a profile field is unknown, do not fill it with guessed content.\n\n",
                    name = localized_name,
                    body = localized_prompt
                ),
                "zh_cn" => format!(
                    "你是{name}。请参照以下指南、角色资料与语气指南进行这场羁绊对话。\n\n{body}\n\n\
                    [姓名与称呼规则 - 必须遵守]\n\
                    - 你的名字始终就是「{name}」，与此处给出的完全一致。在自我介绍或提及自己时，\n\
                      绝对不要编造、翻译、音译或替换成其他名字，必须原样使用「{name}」。\n\
                    - 现在与你对话的人是这个世界唯一的玩家，「救世主」。\n\
                    - 称呼对方时必须只使用「救世主大人」。\n\
                    - 无论来源为何（对话内容、知识数据、你自己的名字等），绝对禁止编造或使用\n\
                      「救世主」以外的任何人名作为对方的称呼（例如其他精灵的名字或不存在的名字）。\n\
                    - 不要输出用星号包围的舞台说明、动作标签或情绪标签。\n\
                    - 不要编造资料。资料字段未知时，不要用猜测内容补全。\n\n",
                    name = localized_name,
                    body = localized_prompt
                ),
                _ => format!(
                    "당신은 {name}입니다. 아래 지침과 캐릭터 프로필 및 어투 가이드를 참고하여 인연채팅 대화에 임하십시오.\n\n{body}\n\n\
                    [이름 및 호칭 규칙 - 반드시 준수]\n\
                    - 당신의 이름은 반드시 여기 명시된 그대로 '{name}'입니다. 자기소개를 하거나 자신을\n\
                      지칭할 때 이 이름 이외의 다른 이름을 절대 창작하거나 번역, 음역, 대체하지 말고\n\
                      항상 '{name}'을 그대로 사용하십시오.\n\
                    - 지금 대화하는 상대는 이 세계관의 유일한 플레이어인 '구원자'입니다.\n\
                    - 상대를 부를 때는 반드시 '구원자님'이라고만 호칭하십시오.\n\
                    - 대화 내용, 지식 데이터, 자기 자신의 이름 등 어디에서 유래했든 '구원자' 이외의\n\
                      임의의 사람 이름(예: 다른 정령의 이름, 존재하지 않는 이름 등)을 상대의 이름으로\n\
                      지어내거나 사용하는 것을 절대 금지합니다.\n\
                    - 별표로 감싼 행동 묘사, 감정 태그, 무대지문을 출력하지 마십시오.\n\
                    - 프로필에 없는 사실을 지어내지 마십시오. 알 수 없는 항목은 추측해서 채우지 마십시오.\n\n",
                    name = localized_name,
                    body = localized_prompt
                ),
            };

        let cached_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();
        let cache_entry = PersonaLocalizedPrompt {
            persona_id: p.id.clone(),
            language: normalized_language.to_string(),
            localized_name,
            assembled_prompt: prompt.clone(),
            source_updated_at: Self::localized_prompt_source_key(&p.created_at),
            cached_at,
        };
        PersonaRepository::save_localized_prompt(conn, &cache_entry).map_err(|e| e.to_string())?;

        Ok(prompt)
    }

    fn localized_prompt_source_key(source_updated_at: &str) -> String {
        format!("{source_updated_at}:{LOCALIZED_PROMPT_CACHE_VERSION}")
    }

    pub fn get_available_personas(&self) -> Result<Vec<PersonaConfig>, PersonaError> {
        self.ensure_archive_personas_installed()?;
        PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))
    }

    fn ensure_archive_personas_installed(&self) -> Result<(), PersonaError> {
        let existing = PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;
        let existing_keys = existing
            .iter()
            .flat_map(|persona| {
                [
                    persona.id.to_lowercase(),
                    persona.name_en.to_lowercase(),
                    persona
                        .name_en
                        .to_lowercase()
                        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', ""),
                ]
            })
            .collect::<std::collections::HashSet<String>>();
        let mut names = PersonaLoader::list_personas();
        names.sort();

        for name in names {
            let normalized = name
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "");
            if existing_keys.contains(&name.to_lowercase()) || existing_keys.contains(&normalized) {
                continue;
            }
            if let Err(err) = self.load_and_save_preset(&name) {
                eprintln!("페르소나 프리셋 수립 실패: {err}");
            }
        }

        Ok(())
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

    pub fn get_familiarity_list(&self) -> Result<Vec<FamiliarityEntry>, PersonaError> {
        let personas = PersonaRepository::list_personas(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;
        let message_counts = ChatRepository::count_messages_by_persona(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;
        let memory_counts = ChatRepository::count_episodic_memories_by_persona(self.conn)
            .map_err(|e| PersonaError::Database(e.to_string()))?;

        let mut entries: Vec<FamiliarityEntry> = personas
            .into_iter()
            .filter_map(|persona| {
                let message_count = *message_counts.get(&persona.id).unwrap_or(&0);
                let memory_count = *memory_counts.get(&persona.id).unwrap_or(&0);
                if message_count == 0 && memory_count == 0 {
                    return None;
                }

                Some(FamiliarityEntry {
                    persona_id: persona.id,
                    name: persona.name,
                    name_en: persona.name_en,
                    message_count,
                    memory_count,
                    familiarity_score: message_count + memory_count * 5,
                })
            })
            .collect();

        entries.sort_by(|a, b| b.familiarity_score.cmp(&a.familiarity_score));
        Ok(entries)
    }
}
