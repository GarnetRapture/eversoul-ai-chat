import type { PersonaConfig, SpiritDetail } from '../persona';
import type { EverTalkLabels } from './i18n';
import type { ApiConnectionState, ApiStatusItem, SpiritRosterMeta, TalkChoice } from './types';
export function filterSpirits(spirits: PersonaConfig[], searchQuery: string): PersonaConfig[] {
    const query = searchQuery.trim().toLowerCase();
    if (!query) {
        return spirits;
    }
    return spirits.filter((spirit) => (spirit.name.toLowerCase().includes(query) ||
        spirit.name_en.toLowerCase().includes(query) ||
        spirit.race.toLowerCase().includes(query) ||
        spirit.class.toLowerCase().includes(query)));
}
export function createRosterMeta(spirit: PersonaConfig): SpiritRosterMeta {
    const preview = spirit.greeting || spirit.class || spirit.race;
    return {
        preview,
    };
}
export function createTalkChoices(detail: SpiritDetail | null, labels: EverTalkLabels): TalkChoice[] {
    if (!detail) {
        return [];
    }
    return [
        ...detail.profile.like.map((label, index) => ({
            id: `like-${index}-${label}`,
            label,
            source: labels.like,
        })),
        ...detail.profile.hobby.map((label, index) => ({
            id: `hobby-${index}-${label}`,
            label,
            source: labels.hobby,
        })),
        ...detail.profile.speciality.map((label, index) => ({
            id: `speciality-${index}-${label}`,
            label,
            source: labels.speciality,
        })),
    ].filter((choice) => choice.label.trim().length > 0);
}
export function pickRandomSpeechLine(detail: SpiritDetail | null): string {
    if (!detail || detail.speech_patterns.length === 0) {
        return '';
    }
    const candidates = detail.speech_patterns.filter((line) => line.trim().length > 0);
    if (candidates.length === 0) {
        return '';
    }
    const index = Math.floor(Math.random() * candidates.length);
    return candidates[index];
}
export function pickPokeReactionLine(detail: SpiritDetail | null, excluding: string): string {
    if (!detail || detail.speech_patterns.length === 0) {
        return '';
    }
    const candidates = detail.speech_patterns.filter((line) => line.trim().length > 0 && line !== excluding);
    const pool = candidates.length > 0 ? candidates : detail.speech_patterns;
    const index = Math.floor(Math.random() * pool.length);
    return pool[index];
}
export function createRoomTitle(name: string): string {
    const trimmed = name.trim();
    return trimmed.length > 0 ? trimmed : 'EverTalk';
}
export function createConversationSummary(detail: SpiritDetail | null): string {
    if (!detail) {
        return '';
    }
    return detail.personality.description || detail.personality.greeting || detail.class || detail.race;
}
export function createApiStatus(id: string, label: string, state: ApiConnectionState, detail: string): ApiStatusItem {
    return {
        id,
        label,
        state,
        detail,
    };
}
export function formatUnknownError(err: unknown): string {
    if (err instanceof Error) {
        return err.message;
    }
    if (typeof err === 'string') {
        return err;
    }
    return JSON.stringify(err);
}
