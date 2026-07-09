use super::types::{ImportedModule, ModuleControl, ModuleControlOption, ModuleError, ModuleStore};
use crate::infrastructure::settings::SettingsManager;
use serde_json::Value;
use std::time::SystemTime;
use uuid::Uuid;

const RPACK_DECODE_MAP: [u8; 256] = [
    44, 247, 132, 139, 201, 101, 251, 182, 159, 174, 179, 3, 45, 1, 105, 116, 31, 228, 163, 236,
    238, 92, 52, 33, 147, 74, 15, 106, 226, 98, 2, 158, 34, 156, 253, 60, 252, 113, 199, 198, 173,
    89, 103, 5, 112, 109, 138, 68, 18, 250, 36, 134, 95, 175, 209, 122, 71, 206, 254, 80, 99, 221,
    81, 6, 111, 24, 224, 82, 168, 9, 157, 86, 115, 76, 184, 83, 108, 195, 160, 14, 25, 207, 62, 13,
    126, 7, 50, 104, 70, 234, 72, 249, 153, 46, 171, 164, 73, 32, 94, 85, 53, 56, 12, 188, 211,
    177, 88, 22, 121, 40, 10, 26, 225, 242, 205, 196, 57, 219, 162, 186, 96, 114, 118, 125, 149,
    239, 127, 200, 192, 222, 55, 148, 191, 181, 20, 129, 146, 37, 69, 172, 231, 245, 102, 167, 43,
    54, 90, 193, 19, 227, 75, 58, 232, 141, 131, 27, 124, 39, 176, 154, 66, 235, 135, 170, 220, 84,
    142, 120, 38, 210, 87, 41, 212, 183, 248, 47, 143, 137, 117, 240, 65, 119, 194, 30, 255, 216,
    21, 17, 229, 4, 151, 23, 243, 49, 208, 155, 0, 215, 202, 180, 79, 42, 59, 217, 178, 107, 218,
    93, 161, 63, 48, 97, 189, 145, 61, 78, 230, 223, 190, 77, 130, 140, 29, 35, 16, 152, 100, 244,
    133, 51, 123, 144, 67, 187, 169, 136, 241, 214, 165, 28, 246, 204, 110, 185, 91, 11, 150, 237,
    213, 233, 197, 203, 8, 166, 128, 64,
];

pub struct ModuleService;

impl ModuleService {
    pub fn list(settings: &SettingsManager) -> Result<Vec<ImportedModule>, ModuleError> {
        Ok(Self::load_store(settings)?.modules)
    }

    pub fn import_from_path(
        settings: &SettingsManager,
        path: &str,
    ) -> Result<ImportedModule, ModuleError> {
        let data = std::fs::read(path).map_err(|e| ModuleError::Io(e.to_string()))?;
        let mut module = Self::parse_risum(&data, Some(path.to_string()))?;
        let mut store = Self::load_store(settings)?;
        module.id = Uuid::new_v4().to_string();
        store
            .modules
            .retain(|existing| existing.name != module.name);
        store.modules.push(module.clone());
        Self::save_store(settings, &store)?;
        Ok(module)
    }

    pub fn set_enabled(
        settings: &SettingsManager,
        id: &str,
        enabled: bool,
    ) -> Result<Vec<ImportedModule>, ModuleError> {
        let mut store = Self::load_store(settings)?;
        let mut found = false;
        for module in &mut store.modules {
            if module.id == id {
                module.enabled = enabled;
                found = true;
                break;
            }
        }
        if !found {
            return Err(ModuleError::Storage(format!("module not found: {id}")));
        }
        Self::save_store(settings, &store)?;
        Ok(store.modules)
    }

    pub fn delete(
        settings: &SettingsManager,
        id: &str,
    ) -> Result<Vec<ImportedModule>, ModuleError> {
        let mut store = Self::load_store(settings)?;
        store.modules.retain(|module| module.id != id);
        Self::save_store(settings, &store)?;
        Ok(store.modules)
    }

    pub fn update_controls(
        settings: &SettingsManager,
        id: &str,
        controls: Vec<ModuleControl>,
    ) -> Result<Vec<ImportedModule>, ModuleError> {
        let mut store = Self::load_store(settings)?;
        let Some(module) = store.modules.iter_mut().find(|module| module.id == id) else {
            return Err(ModuleError::Storage(format!("module not found: {id}")));
        };
        module.controls = controls;
        Self::save_store(settings, &store)?;
        Ok(store.modules)
    }

    pub fn active_prompt(settings: &SettingsManager) -> Result<String, ModuleError> {
        let modules = Self::load_store(settings)?.modules;
        let active = modules
            .into_iter()
            .filter(|module| module.enabled && !module.prompt.trim().is_empty())
            .collect::<Vec<_>>();
        if active.is_empty() {
            return Ok(String::new());
        }

        let mut block = String::from("\n[Imported Risu Modules]\n");
        for module in active {
            block.push_str(&format!("\n## {}\n{}\n", module.name, module.prompt.trim()));
            let enabled_controls = module
                .controls
                .iter()
                .filter(|control| !control.value.trim().is_empty() && control.value != "0")
                .collect::<Vec<_>>();
            if !enabled_controls.is_empty() {
                block.push_str("\n[Module Controls]\n");
                for control in enabled_controls {
                    block.push_str(&format!(
                        "- {} ({}): {}\n",
                        control.label,
                        control.id,
                        control.value.trim()
                    ));
                }
            }
        }
        Ok(block)
    }

    fn parse_risum(
        data: &[u8],
        source_path: Option<String>,
    ) -> Result<ImportedModule, ModuleError> {
        if data.len() < 6 || data[0] != 111 || data[1] != 0 {
            return Err(ModuleError::InvalidFormat(
                "invalid .risum magic or version".to_string(),
            ));
        }
        let main_len = u32::from_le_bytes([data[2], data[3], data[4], data[5]]) as usize;
        let main_start = 6;
        let main_end = main_start + main_len;
        if main_end > data.len() {
            return Err(ModuleError::InvalidFormat(
                "module payload length exceeds file size".to_string(),
            ));
        }

        let decoded = data[main_start..main_end]
            .iter()
            .map(|byte| RPACK_DECODE_MAP[*byte as usize])
            .collect::<Vec<u8>>();
        let root = serde_json::from_slice::<Value>(&decoded)
            .map_err(|e| ModuleError::InvalidFormat(e.to_string()))?;
        if root.get("type").and_then(Value::as_str) != Some("risuModule") {
            return Err(ModuleError::InvalidFormat(
                "payload is not a Risu module".to_string(),
            ));
        }
        let module = root
            .get("module")
            .ok_or_else(|| ModuleError::InvalidFormat("missing module body".to_string()))?;

        let name = module
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Imported Risu Module")
            .to_string();
        let description = module
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let lorebook = module
            .get("lorebook")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let regex_count = module
            .get("regex")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        let trigger_count = module
            .get("trigger")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        Ok(ImportedModule {
            id: Uuid::new_v4().to_string(),
            name,
            description: description.clone(),
            source_path,
            enabled: true,
            prompt: Self::build_prompt(&description, module, &lorebook),
            controls: Self::build_controls(module),
            lorebook_count: lorebook.len(),
            regex_count,
            trigger_count,
            created_at,
        })
    }

    fn build_prompt(description: &str, module: &Value, lorebook: &[Value]) -> String {
        let mut prompt = String::new();
        if !description.trim().is_empty() {
            prompt.push_str("[Description]\n");
            prompt.push_str(description.trim());
            prompt.push('\n');
        }
        if let Some(toggle) = module.get("customModuleToggle").and_then(Value::as_str) {
            if !toggle.trim().is_empty() {
                prompt.push_str("\n[Module Toggle]\n");
                prompt.push_str(toggle.trim());
                prompt.push('\n');
            }
        }
        if !lorebook.is_empty() {
            prompt.push_str("\n[Lorebook]\n");
            for entry in lorebook {
                let comment = entry.get("comment").and_then(Value::as_str).unwrap_or("");
                let content = entry.get("content").and_then(Value::as_str).unwrap_or("");
                if content.trim().is_empty() {
                    continue;
                }
                if !comment.trim().is_empty() {
                    prompt.push_str(&format!("### {}\n", comment.trim()));
                }
                prompt.push_str(content.trim());
                prompt.push_str("\n\n");
            }
        }
        prompt
    }

    fn build_controls(module: &Value) -> Vec<ModuleControl> {
        let mut source = String::new();
        if let Ok(text) = serde_json::to_string(module) {
            source = text;
        }
        let mut ids = Vec::<String>::new();
        for marker in ["getglobalvar::", "setglobalvar::"] {
            let mut rest = source.as_str();
            while let Some(index) = rest.find(marker) {
                let after = &rest[index + marker.len()..];
                let id = after
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                    .collect::<String>();
                if !id.is_empty() && !ids.contains(&id) {
                    ids.push(id);
                }
                rest = after;
            }
        }
        ids.sort();

        ids.into_iter()
            .map(|id| {
                let (kind, value, options) = Self::control_shape(&id);
                ModuleControl {
                    label: Self::control_label(&id),
                    id,
                    kind: kind.to_string(),
                    value: value.to_string(),
                    options,
                }
            })
            .collect()
    }

    fn control_shape(id: &str) -> (&'static str, &'static str, Vec<ModuleControlOption>) {
        let options = |items: &[(&str, &str)]| {
            items
                .iter()
                .map(|(value, label)| ModuleControlOption {
                    value: (*value).to_string(),
                    label: (*label).to_string(),
                })
                .collect::<Vec<_>>()
        };

        match id {
            "toggle_response_mode" => (
                "select",
                "0",
                options(&[("0", "출력용"), ("1", "간결"), ("2", "소설형")]),
            ),
            "toggle_writer" => (
                "select",
                "Gemini",
                options(&[("Gemini", "Gemini"), ("Claude", "Claude"), ("GPT", "GPT")]),
            ),
            "toggle_RPD" | "toggle_RPreq" => (
                "select",
                "0",
                options(&[("0", "비활성"), ("1", "활성")]),
            ),
            id if id.contains("keyword") || id.contains("word") => ("text", "", Vec::new()),
            _ => ("boolean", "0", Vec::new()),
        }
    }

    fn control_label(id: &str) -> String {
        match id {
            "toggle_response_mode" => "응답 형식".to_string(),
            "toggle_writer" => "모델 선택".to_string(),
            "toggle_wordRequest" => "반영할 키워드".to_string(),
            "toggle_RPD" => "문장부호".to_string(),
            "toggle_RPreq" => "TRPG 모드".to_string(),
            "toggle_possessive" => "소유욕 필터".to_string(),
            "toggle_endover" => "검열해제".to_string(),
            _ => id
                .trim_start_matches("toggle_")
                .replace('_', " ")
                .split_whitespace()
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    fn load_store(settings: &SettingsManager) -> Result<ModuleStore, ModuleError> {
        let path = settings.modules_path();
        if !path.exists() {
            return Ok(ModuleStore::default());
        }
        let data =
            std::fs::read_to_string(path).map_err(|e| ModuleError::Storage(e.to_string()))?;
        serde_json::from_str::<ModuleStore>(&data).map_err(|e| ModuleError::Storage(e.to_string()))
    }

    fn save_store(settings: &SettingsManager, store: &ModuleStore) -> Result<(), ModuleError> {
        let path = settings.modules_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ModuleError::Storage(e.to_string()))?;
        }
        let data =
            serde_json::to_string_pretty(store).map_err(|e| ModuleError::Storage(e.to_string()))?;
        std::fs::write(path, data).map_err(|e| ModuleError::Storage(e.to_string()))
    }
}
