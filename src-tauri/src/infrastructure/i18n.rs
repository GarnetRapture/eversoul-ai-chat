/// `language`는 SettingsManager가 관리하는 앱 표시 언어 코드(`ko`/`en`/`zh_cn`/`zh_tw`)다.
/// 프론트엔드에 노출되는 에러 메시지뿐 아니라 백엔드 stderr 로그(`eprintln!`)도
/// 이 언어를 기준으로 렌더링해, 외국어 사용자가 디버그 빌드 콘솔이나 로그 파일을
/// 봤을 때도 이해할 수 있어야 한다는 원칙에 따라 전 도메인에서 공용으로 쓰는 선택 함수.
pub fn pick(language: &str, ko: String, en: String, zh: String) -> String {
    match language {
        "en" => en,
        "zh_cn" | "zh_tw" => zh,
        _ => ko,
    }
}
