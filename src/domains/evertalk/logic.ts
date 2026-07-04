import type { PersonaConfig, SpiritDetail } from '../persona';
import type { BondProgress, SpiritRosterMeta, TalkChoice } from './types';

export function filterSpirits(spirits: PersonaConfig[], searchQuery: string): PersonaConfig[] {
  const query = searchQuery.trim().toLowerCase();
  if (!query) {
    return spirits;
  }

  return spirits.filter((spirit) => (
    spirit.name.toLowerCase().includes(query) ||
    spirit.name_en.toLowerCase().includes(query) ||
    spirit.race.toLowerCase().includes(query) ||
    spirit.class.toLowerCase().includes(query)
  ));
}

export function createRoomTitle(spiritName: string): string {
  return `${spiritName}의 인연스토리`;
}

export function createLocalTimestamp(): string {
  return new Date().toLocaleTimeString();
}

export function createRosterMeta(spirit: PersonaConfig, index: number): SpiritRosterMeta {
  const preview = spirit.greeting || `${spirit.name}에게 인연 메시지가 도착했습니다.`;
  const unreadCount = index % 5 === 0 ? 3 : index % 3 === 0 ? 1 : 0;

  return {
    preview,
    unreadCount,
    isNew: unreadCount > 0,
  };
}

export function createBondProgress(detail: SpiritDetail | null): BondProgress {
  if (!detail) {
    return {
      level: 0,
      current: 0,
      max: 60,
      percent: 0,
      meterClass: 'is-0',
    };
  }

  const nameScore = Array.from(detail.name_en).reduce((sum, char) => sum + char.charCodeAt(0), 0);
  const level = (nameScore % 6) + 1;
  const current = 10 + (nameScore % 43);
  const max = 60;

  const percent = Math.min(100, Math.round((current / max) * 100));
  const meterStep = Math.min(100, Math.max(0, Math.round(percent / 10) * 10));

  return {
    level,
    current,
    max,
    percent,
    meterClass: `is-${meterStep}`,
  };
}

export function createTalkChoices(detail: SpiritDetail | null): TalkChoice[] {
  if (!detail) {
    return [];
  }

  const source = [
    ...detail.profile.like,
    ...detail.profile.hobby,
    ...detail.profile.speciality,
  ].filter(Boolean);

  const primary = source[0] ?? detail.race;
  const secondary = source[1] ?? detail.class;
  const rare = detail.profile.nick_name ?? detail.name_en;

  return [
    {
      id: 'daily-topic',
      label: `${primary} 이야기하기`,
      detail: '일상 대화',
      reward: '인연 포인트 +15',
      rarity: 'normal',
    },
    {
      id: 'outing-plan',
      label: `${secondary} 나들이 제안`,
      detail: '컨텍스트 확장',
      reward: '인연 포인트 +15',
      rarity: 'normal',
    },
    {
      id: 'memory-keyword',
      label: `${rare} 키워드`,
      detail: '희귀 화제',
      reward: '인연 포인트 +30',
      rarity: 'rare',
    },
  ];
}

export function createConversationSummary(detail: SpiritDetail | null): string {
  if (!detail) {
    return '정령을 선택하면 대화 맥락과 인연 키워드가 구성됩니다.';
  }

  return detail.personality.description || detail.personality.greeting || `${detail.name}의 인연 대화가 준비되었습니다.`;
}
