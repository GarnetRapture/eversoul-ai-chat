use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
}

pub struct UpdateManager {
    current_version: String,
}

impl UpdateManager {
    /// 현재 클라이언트의 버전을 받아 UpdateManager를 생성한다.
    pub fn new(current_version: String) -> Self {
        Self { current_version }
    }

    /// 원격 서버에서 반환받은 최신 버전과 비교하여 업데이트 필요 여부를 판단한다.
    pub fn check_for_updates(&self, latest_version: &str, download_url: &str) -> UpdateCheckResult {
        let has_update =
            self.parse_version(&self.current_version) < self.parse_version(latest_version);

        UpdateCheckResult {
            has_update,
            current_version: self.current_version.clone(),
            latest_version: latest_version.to_string(),
            download_url: if has_update {
                Some(download_url.to_string())
            } else {
                None
            },
        }
    }

    /// "major.minor.patch" 형태의 문자열을 비교용 튜플로 파싱한다.
    fn parse_version(&self, version: &str) -> (u32, u32, u32) {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return (0, 0, 0);
        }

        let major = parts[0].parse::<u32>().unwrap_or(0);
        let minor = parts[1].parse::<u32>().unwrap_or(0);
        let patch = parts[2].parse::<u32>().unwrap_or(0);

        (major, minor, patch)
    }
}
