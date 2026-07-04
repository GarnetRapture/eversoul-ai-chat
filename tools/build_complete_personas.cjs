const fs = require('fs');
const path = require('path');
const cp = require('child_process');
const zlib = require('zlib');

// 경로 수정: tools/ 하위에서 프로젝트 전체를 상대 경로로 참조
const dbPath = path.join(__dirname, '..', 'ai', 'TBL', 'tbl_map.sqlite');
const outputDir = path.join(__dirname, '..', 'data', 'personas');

if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

console.log('1. 다국어 스트링 리소스 로드 및 메모리 캐싱 중...');
const stringLookupHexPath = path.join(__dirname, 'string_lookup_dump.txt');
cp.execSync(`sqlite3 "${dbPath}" ".headers off" ".mode list" ".separator |" ".output ${stringLookupHexPath}" "select value, kr from string_lookup;"`);

const stringMap = {};
if (fs.existsSync(stringLookupHexPath)) {
    const content = fs.readFileSync(stringLookupHexPath, 'utf8');
    const lines = content.split(/\r?\n/);
    for (const line of lines) {
        if (!line.trim()) continue;
        const idx = line.indexOf('|');
        if (idx === -1) continue;
        const val = line.substring(0, idx).trim();
        const kr = line.substring(idx + 1);
        stringMap[val] = kr;
    }
    fs.unlinkSync(stringLookupHexPath);
}
console.log(`- 캐싱 완료: ${Object.keys(stringMap).length}개 스트링 리소스 확보`);

function decompressTable(tableName) {
    const hexPath = path.join(__dirname, `hex_${tableName}.txt`);
    cp.execSync(`sqlite3 "${dbPath}" ".headers off" ".mode list" ".output ${hexPath}" "select row_key, hex(raw_json_zlib) from rows where table_name='${tableName}';"`);
    
    const result = {};
    if (fs.existsSync(hexPath)) {
        const content = fs.readFileSync(hexPath, 'utf8');
        const lines = content.split(/\r?\n/);
        for (const line of lines) {
            if (!line.trim()) continue;
            const idx = line.indexOf('|');
            if (idx === -1) continue;
            const key = line.substring(0, idx).trim();
            const hex = line.substring(idx + 1).trim();
            try {
                const decompressed = zlib.inflateSync(Buffer.from(hex, 'hex')).toString('utf8');
                result[key] = JSON.parse(decompressed);
            } catch (e) {
                // 패스
            }
        }
        fs.unlinkSync(hexPath);
    }
    return result;
}

console.log('2. 기초 메타데이터(Hero, HeroDesc, TalkActor, HeroComment) 파싱 중...');
const rawRefsPath = path.join(__dirname, 'raw_refs.txt');
cp.execSync(`sqlite3 "${dbPath}" ".headers off" ".mode list" ".separator |" ".output ${rawRefsPath}" "select table_name, row_key, field_path, label_json from resolved_refs where table_name in ('HeroDesc', 'Hero');"`);

const spirits = {};
const descToHeroMap = {};

if (fs.existsSync(rawRefsPath)) {
    const rawContent = fs.readFileSync(rawRefsPath, 'utf8');
    const lines = rawContent.split(/\r?\n/);
    for (const line of lines) {
        if (!line.trim()) continue;
        const parts = line.split('|');
        if (parts.length < 4) continue;

        const tableName = parts[0];
        const rowKey = parts[1];
        const fieldPath = parts[2];
        const labelJsonStr = parts.slice(3).join('|');

        let krVal = '';
        let enVal = '';
        let targetRowKey = '';

        try {
            const parsed = JSON.parse(labelJsonStr);
            krVal = parsed.kr || '';
            enVal = parsed.en || '';
            if (parsed.target_row_key) targetRowKey = parsed.target_row_key;
            else if (parsed.no) targetRowKey = parsed.no;
        } catch (e) {
            continue;
        }

        if (tableName === 'Hero') {
            if (!spirits[rowKey]) {
                spirits[rowKey] = {
                    id: rowKey,
                    name: '',
                    name_en: '',
                    grade: '',
                    race: '',
                    class: '',
                    sub_class: '',
                    stat: '',
                    nick_name: '',
                    like: '',
                    dislike: '',
                    hobby: '',
                    speciality: '',
                    introduction: '',
                    ment: '',
                    cv: '',
                    cv_jp: '',
                    constellation: '',
                    union: '',
                    height: null,
                    weight: null,
                    birthday: null,
                    comments: [],
                    story_dialogues: [],
                    evertalk_dialogues: []
                };
            }
            const s = spirits[rowKey];
            if (fieldPath === 'name_sno') {
                s.name = krVal;
                s.name_en = enVal;
            }
            else if (fieldPath === 'grade_sno') s.grade = krVal;
            else if (fieldPath === 'race_sno') s.race = krVal;
            else if (fieldPath === 'class_sno') s.class = krVal;
            else if (fieldPath === 'sub_class_sno') s.sub_class = krVal;
            else if (fieldPath === 'stat_sno') s.stat = krVal;

        } else if (tableName === 'HeroDesc') {
            if (fieldPath === 'hero_no' && targetRowKey) {
                descToHeroMap[rowKey] = targetRowKey;
            }
        }
    }
    fs.unlinkSync(rawRefsPath);
}

const heroDescRows = decompressTable('HeroDesc');
for (const descKey in heroDescRows) {
    const row = heroDescRows[descKey];
    const heroId = row.hero_no;
    if (heroId && spirits[heroId]) {
        const s = spirits[heroId];
        s.height = row.height || null;
        s.weight = row.weight || null;
        s.birthday = row.birthday || null;
        
        if (row.nick_name_sno) s.nick_name = stringMap[row.nick_name_sno] || s.nick_name;
        if (row.like_sno) s.like = stringMap[row.like_sno] || s.like;
        if (row.dislike_sno) s.dislike = stringMap[row.dislike_sno] || s.dislike;
        if (row.hobby_sno) s.hobby = stringMap[row.hobby_sno] || s.hobby;
        if (row.speciality_sno) s.speciality = stringMap[row.speciality_sno] || s.speciality;
        if (row.introduction_sno) s.introduction = stringMap[row.introduction_sno] || s.introduction;
        if (row.ment_sno) s.ment = stringMap[row.ment_sno] || s.ment;
        if (row.cv_sno) s.cv = stringMap[row.cv_sno] || s.cv;
        if (row.cv_jp_sno) s.cv_jp = stringMap[row.cv_jp_sno] || s.cv_jp;
        if (row.constellation_sno) s.constellation = stringMap[row.constellation_sno] || s.constellation;
        if (row.union_sno) s.union = stringMap[row.union_sno] || s.union;
    }
}

const talkActors = decompressTable('TalkActor');
const actorToHeroMap = {};
for (const actorKey in talkActors) {
    const actor = talkActors[actorKey];
    const nameKr = stringMap[actor.name_sno];
    if (nameKr) {
        actorToHeroMap[actor.no] = nameKr;
    }
}

const heroComments = decompressTable('HeroComment');
const heroIdToNameMap = {};
for (const id in spirits) {
    heroIdToNameMap[id] = spirits[id].name;
}
for (const commentKey in heroComments) {
    const comment = heroComments[commentKey];
    const targetHeroId = comment.hero_no;
    if (targetHeroId && spirits[targetHeroId]) {
        const writerName = heroIdToNameMap[comment.comment_hero_no] || '정령';
        const commentText = stringMap[comment.comment_desc] || '';
        
        if (commentText && commentText.trim() !== '퀘스트' && commentText.trim() !== '테스트') {
            spirits[targetHeroId].comments.push({
                writer: writerName,
                comment: commentText
            });
        }
    }
}

const validSpirits = {};
for (const id in spirits) {
    const s = spirits[id];
    if (s.name && s.name_en && s.cv && s.cv.trim() !== '') {
        const keyName = s.name.trim();
        if (!validSpirits[keyName]) {
            validSpirits[keyName] = s;
        }
    }
}
const validHeroIds = Object.values(validSpirits).map(s => s.id);
console.log(`- 필터링 완료: 플레이어블 정령 캐릭터 ${validHeroIds.length}명 선점`);

console.log('3. 정령 대화 및 메신저 전문 (Talk / EverTalkDesc) 매핑 및 노이즈 필터링 중...');

function isDummyMessage(msg) {
    if (!msg) return true;
    const trimmed = msg.trim();
    if (trimmed === '' || 
        trimmed === '퀘스트' || 
        trimmed === '테스트' || 
        trimmed === 'test' || 
        trimmed === 'system' ||
        trimmed.startsWith('<display:')
    ) {
        return true;
    }
    return false;
}

const talkHexPath = path.join(__dirname, 'temp_talk.txt');
cp.execSync(`sqlite3 "${dbPath}" ".headers off" ".mode list" ".output ${talkHexPath}" "select hex(raw_json_zlib) from rows where table_name='Talk' and row_key in (select row_key from cell_values where table_name='Talk' and field_path='hero_no' and value in (${validHeroIds.map(id => `'${id}'`).join(',')}));"`);

if (fs.existsSync(talkHexPath)) {
    const content = fs.readFileSync(talkHexPath, 'utf8');
    const lines = content.split(/\r?\n/);
    for (const line of lines) {
        if (!line.trim()) continue;
        try {
            const decompressed = zlib.inflateSync(Buffer.from(line.trim(), 'hex')).toString('utf8');
            const row = JSON.parse(decompressed);
            const heroId = row.hero_no;
            const speakerName = actorToHeroMap[row.speaker_no] || '구원자';
            const dialogueText = stringMap[row.speaking] || '';
            
            if (heroId && dialogueText && !isDummyMessage(dialogueText)) {
                for (const name in validSpirits) {
                    if (validSpirits[name].id === String(heroId)) {
                        validSpirits[name].story_dialogues.push({
                            speaker: speakerName,
                            message: dialogueText
                        });
                    }
                }
            }
        } catch (e) {
            // 패스
        }
    }
    fs.unlinkSync(talkHexPath);
}

const everTalkHexPath = path.join(__dirname, 'temp_evertalk.txt');
cp.execSync(`sqlite3 "${dbPath}" ".headers off" ".mode list" ".output ${everTalkHexPath}" "select hex(raw_json_zlib) from rows where table_name='EverTalkDesc' and row_key in (select row_key from cell_values where table_name='EverTalkDesc' and field_path='hero_no' and value in (${validHeroIds.map(id => `'${id}'`).join(',')}));"`);

if (fs.existsSync(everTalkHexPath)) {
    const content = fs.readFileSync(everTalkHexPath, 'utf8');
    const lines = content.split(/\r?\n/);
    for (const line of lines) {
        if (!line.trim()) continue;
        try {
            const decompressed = zlib.inflateSync(Buffer.from(line.trim(), 'hex')).toString('utf8');
            const row = JSON.parse(decompressed);
            const heroId = row.hero_no;
            const speakerName = row.speaker_no === 0 ? '구원자' : (heroIdToNameMap[row.speaker_no] || '정령');
            const dialogueText = stringMap[row.no] || '';
            
            if (heroId && dialogueText && !isDummyMessage(dialogueText)) {
                for (const name in validSpirits) {
                    if (validSpirits[name].id === String(heroId)) {
                        validSpirits[name].evertalk_dialogues.push({
                            speaker: speakerName,
                            message: dialogueText
                        });
                    }
                }
            }
        } catch (e) {
            // 패스
        }
    }
    fs.unlinkSync(everTalkHexPath);
}

console.log('4. 완성된 종합 페르소나 데이터팩 개별 JSON 파일 저장 중...');
let writtenCount = 0;

function parseCsvList(str) {
    if (!str) return [];
    return str.split(',').map(item => item.trim()).filter(item => item.length > 0);
}

function getSafeEnglishFileName(enName) {
    return enName.toLowerCase()
        .replace(/[^a-z0-9\s_-]/g, '')
        .trim()
        .replace(/\s+/g, '_');
}

for (const name in validSpirits) {
    const s = validSpirits[name];

    const selfSpeechPatterns = [];
    const maxPatterns = 50;
    
    for (const d of s.story_dialogues) {
        if (d.speaker === s.name && !isDummyMessage(d.message) && !selfSpeechPatterns.includes(d.message)) {
            selfSpeechPatterns.push(d.message);
            if (selfSpeechPatterns.length >= maxPatterns) break;
        }
    }
    
    if (selfSpeechPatterns.length < maxPatterns) {
        for (const d of s.evertalk_dialogues) {
            if (d.speaker === s.name && !isDummyMessage(d.message) && !selfSpeechPatterns.includes(d.message)) {
                selfSpeechPatterns.push(d.message);
                if (selfSpeechPatterns.length >= maxPatterns) break;
            }
        }
    }

    const completeData = {
        id: s.id,
        name: s.name,
        name_en: s.name_en,
        grade: s.grade,
        race: s.race,
        class: s.class,
        sub_class: s.sub_class,
        stat: s.stat,
        profile: {
            nick_name: s.nick_name || null,
            constellation: s.constellation || null,
            union: s.union || null,
            birthday: s.birthday ? String(s.birthday) : null,
            height: s.height ? Number(s.height) : null,
            weight: s.weight ? Number(s.weight) : null,
            cv_ko: s.cv || null,
            cv_jp: s.cv_jp || null,
            like: parseCsvList(s.like),
            dislike: parseCsvList(s.dislike),
            hobby: parseCsvList(s.hobby),
            speciality: parseCsvList(s.speciality)
        },
        personality: {
            description: s.introduction || null,
            greeting: s.ment || null
        },
        speech_patterns: selfSpeechPatterns,
        comments: s.comments,
        dialogues: {
            story: s.story_dialogues,
            evertalk: s.evertalk_dialogues
        }
    };

    const safeEnName = getSafeEnglishFileName(s.name_en);
    if (!safeEnName) continue;

    const fileName = `${safeEnName}.json`;
    const filePath = path.join(outputDir, fileName);

    fs.writeFileSync(filePath, JSON.stringify(completeData, null, 2), 'utf8');
    writtenCount++;
}

console.log(`\n🎉 [완벽한 아키텍처 데이터 구축 완료] 총 ${writtenCount}명 정령의 신체 프로필, 정령평가, 인연/메인 스토리 대사 전문, 에버톡 전문이 결합된 종합 데이터팩 JSON을 저장했습니다.`);
