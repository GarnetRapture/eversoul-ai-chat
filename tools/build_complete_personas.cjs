const fs = require('fs');
const path = require('path');

const tblJsonDir = path.join(__dirname, '..', 'ai', 'tbl_json');
const outputDir = path.join(__dirname, '..', 'data', 'personas');

const LANGUAGES = ['ko', 'en', 'zh_tw', 'zh_cn'];
const STRING_FILE_PREFIX = 'String';
const MAX_SPEECH_PATTERNS = 50;

if (!fs.existsSync(tblJsonDir)) {
  console.error(`TBL JSON 경로가 존재하지 않습니다: ${tblJsonDir}`);
  process.exit(1);
}

if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

function loadTable(tableName) {
  const filePath = path.join(tblJsonDir, `${tableName}.json`);
  if (!fs.existsSync(filePath)) {
    return [];
  }

  const parsed = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  return Array.isArray(parsed.json) ? parsed.json : [];
}

function normalizeLocalizedRow(row) {
  const ko = String(row.kr ?? '').trim();
  const en = String(row.en ?? '').trim();
  const zhTw = String(row.zh_tw ?? '').trim();
  const zhCn = String(row.zh_cn ?? '').trim();

  return {
    ko,
    en: en || ko,
    zh_tw: zhTw || zhCn || en || ko,
    zh_cn: zhCn || zhTw || en || ko,
  };
}

function loadStringMap() {
  const map = new Map();
  const files = fs
    .readdirSync(tblJsonDir)
    .filter((file) => file.startsWith(STRING_FILE_PREFIX) && file.endsWith('.json'));

  for (const file of files) {
    const rows = loadTable(path.basename(file, '.json'));
    for (const row of rows) {
      if (row.no === undefined || row.no === null) {
        continue;
      }
      map.set(String(row.no), normalizeLocalizedRow(row));
    }
  }

  return map;
}

function emptyText() {
  return { ko: '', en: '', zh_tw: '', zh_cn: '' };
}

function textBySno(stringMap, sno) {
  if (sno === undefined || sno === null || sno === '') {
    return emptyText();
  }
  return stringMap.get(String(sno)) ?? emptyText();
}

function primary(text) {
  return text.ko || text.en || text.zh_tw || text.zh_cn || '';
}

function parseCsvList(value) {
  if (!value) {
    return [];
  }
  return value
    .split(',')
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

function parseLocalizedCsvList(text) {
  const result = {};
  for (const language of LANGUAGES) {
    result[language] = parseCsvList(text[language]);
  }
  return result;
}

function getSafeEnglishFileName(enName) {
  return enName
    .toLowerCase()
    .replace(/[^a-z0-9\s_-]/g, '')
    .trim()
    .replace(/\s+/g, '_');
}

function isDummyMessage(message) {
  const trimmed = String(message ?? '').trim();
  return (
    trimmed.length === 0 ||
    trimmed === '퀘스트' ||
    trimmed === '테스트' ||
    trimmed.toLowerCase() === 'test' ||
    trimmed.toLowerCase() === 'system' ||
    trimmed.startsWith('<display:')
  );
}

function isPlayableSpirit(spirit) {
  return Boolean(spirit.name.ko && spirit.name.en && spirit.profile.cv_ko.ko);
}

function createSpiritFromHero(row, stringMap) {
  return {
    id: String(row.no),
    name: textBySno(stringMap, row.name_sno),
    grade: textBySno(stringMap, row.grade_sno),
    race: textBySno(stringMap, row.race_sno),
    class: textBySno(stringMap, row.class_sno),
    sub_class: textBySno(stringMap, row.sub_class_sno),
    stat: textBySno(stringMap, row.stat_sno),
    profile: {
      nick_name: emptyText(),
      constellation: emptyText(),
      union: emptyText(),
      birthday: row.birthday ? String(row.birthday) : null,
      height: null,
      weight: null,
      cv_ko: emptyText(),
      cv_jp: emptyText(),
      like: emptyText(),
      dislike: emptyText(),
      hobby: emptyText(),
      speciality: emptyText(),
    },
    personality: {
      description: emptyText(),
      greeting: emptyText(),
    },
    comments: [],
    story_dialogues: [],
    evertalk_dialogues: [],
  };
}

function localizedDialogue(speaker, message) {
  const value = {};
  for (const language of LANGUAGES) {
    value[language] = {
      speaker: speaker[language] || speaker.ko || speaker.en || '정령',
      message: message[language] || message.ko || message.en || '',
    };
  }
  return value;
}

console.log('1. ai/tbl_json 문자열 리소스 로드 중...');
const stringMap = loadStringMap();
console.log(`- 문자열 리소스 ${stringMap.size}개 로드`);

console.log('2. Hero/HeroDesc/HeroComment/TalkActor 파싱 중...');
const spiritsById = new Map();
for (const hero of loadTable('Hero')) {
  const spirit = createSpiritFromHero(hero, stringMap);
  if (primary(spirit.name)) {
    spiritsById.set(spirit.id, spirit);
  }
}

for (const desc of loadTable('HeroDesc')) {
  const spirit = spiritsById.get(String(desc.hero_no));
  if (!spirit) {
    continue;
  }

  spirit.profile.height = desc.height ? Number(desc.height) : null;
  spirit.profile.weight = desc.weight ? Number(desc.weight) : null;
  spirit.profile.birthday = desc.birthday ? String(desc.birthday) : null;
  spirit.profile.nick_name = textBySno(stringMap, desc.nick_name_sno);
  spirit.profile.constellation = textBySno(stringMap, desc.constellation_sno);
  spirit.profile.union = textBySno(stringMap, desc.union_sno);
  spirit.profile.cv_ko = textBySno(stringMap, desc.cv_sno);
  spirit.profile.cv_jp = textBySno(stringMap, desc.cv_jp_sno);
  spirit.profile.like = textBySno(stringMap, desc.like_sno);
  spirit.profile.dislike = textBySno(stringMap, desc.dislike_sno);
  spirit.profile.hobby = textBySno(stringMap, desc.hobby_sno);
  spirit.profile.speciality = textBySno(stringMap, desc.speciality_sno);
  spirit.personality.description = textBySno(stringMap, desc.introduction_sno);
  spirit.personality.greeting = textBySno(stringMap, desc.ment_sno);
}

const actorNameByNo = new Map();
for (const actor of loadTable('TalkActor')) {
  actorNameByNo.set(String(actor.no), textBySno(stringMap, actor.name_sno));
}

const heroNameById = new Map();
for (const [id, spirit] of spiritsById) {
  heroNameById.set(id, spirit.name);
}

for (const comment of loadTable('HeroComment')) {
  const spirit = spiritsById.get(String(comment.hero_no));
  if (!spirit) {
    continue;
  }

  const writer = heroNameById.get(String(comment.comment_hero_no)) ?? {
    ko: '정령',
    en: 'Soul',
    zh_tw: '精靈',
    zh_cn: '精灵',
  };
  const commentText = textBySno(stringMap, comment.comment_desc);
  if (!isDummyMessage(commentText.ko)) {
    spirit.comments.push({
      writer: primary(writer),
      comment: primary(commentText),
      i18n: localizedDialogue(writer, commentText),
    });
  }
}

const playableSpirits = [...spiritsById.values()].filter(isPlayableSpirit);
const playableHeroIds = new Set(playableSpirits.map((spirit) => spirit.id));
console.log(`- 플레이어블 정령 ${playableSpirits.length}명 확인`);

console.log('3. Talk/EverTalkDesc 전문 연결 중...');
for (const talk of loadTable('Talk')) {
  const spirit = spiritsById.get(String(talk.hero_no));
  if (!spirit || !playableHeroIds.has(spirit.id)) {
    continue;
  }

  const speaker =
    actorNameByNo.get(String(talk.speaker_no)) ??
    heroNameById.get(String(talk.speaker_no)) ?? {
      ko: '구원자',
      en: 'Savior',
      zh_tw: '救援者',
      zh_cn: '救援者',
    };
  const message = textBySno(stringMap, talk.speaking);
  if (!isDummyMessage(message.ko)) {
    spirit.story_dialogues.push({
      speaker: primary(speaker),
      message: primary(message),
      i18n: localizedDialogue(speaker, message),
    });
  }
}

for (const talk of loadTable('EverTalkDesc')) {
  const spirit = spiritsById.get(String(talk.hero_no));
  if (!spirit || !playableHeroIds.has(spirit.id)) {
    continue;
  }

  const speaker =
    Number(talk.speaker_no) === 0
      ? { ko: '구원자', en: 'Savior', zh_tw: '救援者', zh_cn: '救援者' }
      : heroNameById.get(String(talk.speaker_no)) ?? spirit.name;
  const message = textBySno(stringMap, talk.no);
  if (!isDummyMessage(message.ko)) {
    spirit.evertalk_dialogues.push({
      speaker: primary(speaker),
      message: primary(message),
      i18n: localizedDialogue(speaker, message),
    });
  }
}

console.log('4. data/personas JSON 저장 중...');
let writtenCount = 0;

for (const spirit of playableSpirits) {
  const speechPatterns = [];
  const speechPatternsI18n = [];
  const collectSpeech = (dialogues) => {
    for (const dialogue of dialogues) {
      if (
        dialogue.speaker === spirit.name.ko &&
        !isDummyMessage(dialogue.message) &&
        !speechPatterns.includes(dialogue.message)
      ) {
        speechPatterns.push(dialogue.message);
        speechPatternsI18n.push(dialogue.i18n);
        if (speechPatterns.length >= MAX_SPEECH_PATTERNS) {
          return;
        }
      }
    }
  };

  collectSpeech(spirit.story_dialogues);
  if (speechPatterns.length < MAX_SPEECH_PATTERNS) {
    collectSpeech(spirit.evertalk_dialogues);
  }

  const completeData = {
    id: spirit.id,
    name: spirit.name.ko,
    name_en: spirit.name.en,
    grade: spirit.grade.ko,
    race: spirit.race.ko,
    class: spirit.class.ko,
    sub_class: spirit.sub_class.ko,
    stat: spirit.stat.ko,
    profile: {
      nick_name: spirit.profile.nick_name.ko || null,
      constellation: spirit.profile.constellation.ko || null,
      union: spirit.profile.union.ko || null,
      birthday: spirit.profile.birthday,
      height: spirit.profile.height,
      weight: spirit.profile.weight,
      cv_ko: spirit.profile.cv_ko.ko || null,
      cv_jp: spirit.profile.cv_jp.ko || null,
      like: parseCsvList(spirit.profile.like.ko),
      dislike: parseCsvList(spirit.profile.dislike.ko),
      hobby: parseCsvList(spirit.profile.hobby.ko),
      speciality: parseCsvList(spirit.profile.speciality.ko),
    },
    personality: {
      description: spirit.personality.description.ko || null,
      greeting: spirit.personality.greeting.ko || null,
    },
    speech_patterns: speechPatterns,
    comments: spirit.comments.map((comment) => ({
      writer: comment.writer,
      comment: comment.comment,
    })),
    dialogues: {
      story: spirit.story_dialogues.map((dialogue) => ({
        speaker: dialogue.speaker,
        message: dialogue.message,
      })),
      evertalk: spirit.evertalk_dialogues.map((dialogue) => ({
        speaker: dialogue.speaker,
        message: dialogue.message,
      })),
    },
    i18n: {
      name: spirit.name,
      grade: spirit.grade,
      race: spirit.race,
      class: spirit.class,
      sub_class: spirit.sub_class,
      stat: spirit.stat,
      profile: {
        nick_name: spirit.profile.nick_name,
        constellation: spirit.profile.constellation,
        union: spirit.profile.union,
        cv_ko: spirit.profile.cv_ko,
        cv_jp: spirit.profile.cv_jp,
        like: parseLocalizedCsvList(spirit.profile.like),
        dislike: parseLocalizedCsvList(spirit.profile.dislike),
        hobby: parseLocalizedCsvList(spirit.profile.hobby),
        speciality: parseLocalizedCsvList(spirit.profile.speciality),
      },
      personality: {
        description: spirit.personality.description,
        greeting: spirit.personality.greeting,
      },
      speech_patterns: speechPatternsI18n,
      comments: spirit.comments.map((comment) => comment.i18n),
      dialogues: {
        story: spirit.story_dialogues.map((dialogue) => dialogue.i18n),
        evertalk: spirit.evertalk_dialogues.map((dialogue) => dialogue.i18n),
      },
    },
  };

  const safeEnName = getSafeEnglishFileName(spirit.name.en);
  if (!safeEnName) {
    continue;
  }

  fs.writeFileSync(
    path.join(outputDir, `${safeEnName}.json`),
    JSON.stringify(completeData, null, 2),
    'utf8',
  );
  writtenCount++;
}

console.log(`\n완료: ${writtenCount}명 정령 persona JSON을 ai/tbl_json 기반으로 갱신했습니다.`);
