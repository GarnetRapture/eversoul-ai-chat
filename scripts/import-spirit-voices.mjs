import { existsSync, mkdirSync, readdirSync, copyFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = join(SCRIPT_DIR, '..');

const SPIRITS_ROOT = process.argv[2] ?? process.env.EVERSOUL_SPIRITS_ROOT;
const DATA_ROOT = join(PROJECT_ROOT, 'data');
const VOICE_LANGUAGE = 'KR';
const LOBBY_CATEGORY = 'lobby';
const LOBBY_LIMIT_BY_GRADE = { epic: 2, common: 1, rare: 1 };

function listDirectories(path) {
    return readdirSync(path, { withFileTypes: true })
        .filter((entry) => entry.isDirectory())
        .map((entry) => entry.name);
}

function main() {
    if (!SPIRITS_ROOT) {
        console.error('원본 정령 자산 경로를 인자 또는 EVERSOUL_SPIRITS_ROOT 환경 변수로 지정해야 합니다.');
        process.exit(1);
    }

    if (!existsSync(SPIRITS_ROOT)) {
        console.error(`원본 정령 자산 경로가 없습니다: ${SPIRITS_ROOT}`);
        process.exit(1);
    }

    let totalCopied = 0;
    const perCharacter = {};

    for (const gradeName of listDirectories(SPIRITS_ROOT)) {
        const lobbyLimit = LOBBY_LIMIT_BY_GRADE[gradeName];
        if (lobbyLimit === undefined) {
            continue;
        }

        const gradeDir = join(SPIRITS_ROOT, gradeName);
        for (const characterName of listDirectories(gradeDir)) {
            const lobbyDir = join(gradeDir, characterName, 'audio', VOICE_LANGUAGE, LOBBY_CATEGORY);
            if (!existsSync(lobbyDir)) {
                continue;
            }

            const personaId = characterName.toLowerCase();
            const lobbyFiles = readdirSync(lobbyDir, { withFileTypes: true })
                .filter((entry) => entry.isFile() && entry.name.toLowerCase().endsWith('.ogg'))
                .map((entry) => entry.name)
                .sort();

            for (const fileName of lobbyFiles.slice(0, lobbyLimit)) {
                const sourceFile = join(lobbyDir, fileName);
                const targetFile = join(DATA_ROOT, personaId, fileName);
                mkdirSync(dirname(targetFile), { recursive: true });
                copyFileSync(sourceFile, targetFile);
                totalCopied += 1;
                perCharacter[personaId] = (perCharacter[personaId] ?? 0) + 1;
            }
        }
    }

    const characters = Object.keys(perCharacter).sort();
    for (const id of characters) {
        console.info(`${id}: ${perCharacter[id]}개`);
    }
    console.info(`정령 ${characters.length}명, 한국어(${VOICE_LANGUAGE}) 로비 음성 ${totalCopied}개를 data/ 하위로 배치했습니다.`);
}

main();
