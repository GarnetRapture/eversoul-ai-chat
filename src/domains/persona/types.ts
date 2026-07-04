export interface PersonaConfig {
    id: string;
    name: string;
    name_en: string;
    grade: string;
    race: string;
    class: string;
    sub_class: string;
    system_prompt: string;
    greeting: string;
    raw_json: string;
    created_at: string;
}
export interface SpiritDetail {
    id: string;
    name: string;
    name_en: string;
    grade: string;
    race: string;
    class: string;
    sub_class: string;
    stat: string;
    profile: {
        nick_name: string | null;
        constellation: string | null;
        union: string | null;
        birthday: string | null;
        height: number | null;
        weight: number | null;
        cv_ko: string | null;
        cv_jp: string | null;
        like: string[];
        dislike: string[];
        hobby: string[];
        speciality: string[];
    };
    personality: {
        description: string | null;
        greeting: string | null;
    };
    speech_patterns: string[];
    comments: Array<{
        writer: string;
        comment: string;
    }>;
}
export interface SpiritVisualAssets {
    assetFolder: string | null;
    avatarCandidates: string[];
    portraitCandidates: string[];
    background: string;
}
export interface BondRankingEntry {
    persona_id: string;
    name: string;
    name_en: string;
    message_count: number;
    memory_count: number;
    bond_score: number;
}
export type PersonaError = {
    Database: string;
} | {
    Archive: string;
} | {
    NotFound: string;
} | {
    Unknown: string;
};
