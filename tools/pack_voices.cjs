const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

// 상대 경로 교정
const sourceDir = path.join(__dirname, '..', 'data');
const outputDir = path.join(__dirname, '..', 'src-tauri', 'resources');
const outputFile = path.join(outputDir, 'voices.bin');

if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

if (!fs.existsSync(sourceDir)) {
    console.error(`소스 경로가 존재하지 않습니다: ${sourceDir}`);
    process.exit(1);
}

const personaDirs = fs.readdirSync(sourceDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name !== 'personas')
    .map((entry) => entry.name)
    .sort();

const entries = [];

for (const personaId of personaDirs) {
    const personaDir = path.join(sourceDir, personaId);
    const oggFiles = fs.readdirSync(personaDir)
        .filter((f) => f.toLowerCase().endsWith('.ogg'))
        .sort();

    for (const fileName of oggFiles) {
        const rawData = fs.readFileSync(path.join(personaDir, fileName));
        const compressed = zlib.deflateSync(rawData);
        entries.push({
            name: `${personaId}/${fileName}`,
            length: compressed.length,
            compressedData: compressed,
        });
    }
}

console.log(`총 ${entries.length}개의 정령 음성 수집 완료.`);

let headerSize = 4; // 파일 수 4바이트
for (const e of entries) {
    const nameBuf = Buffer.from(e.name, 'utf8');
    headerSize += 1 + nameBuf.length + 4 + 4;
}

const headerBuffer = Buffer.alloc(headerSize);
headerBuffer.writeUInt32BE(entries.length, 0);

let currentOffset = headerSize;
let headerPos = 4;

const bodyBuffers = [];

for (const e of entries) {
    const nameBuf = Buffer.from(e.name, 'utf8');

    headerBuffer.writeUInt8(nameBuf.length, headerPos);
    headerPos += 1;

    nameBuf.copy(headerBuffer, headerPos);
    headerPos += nameBuf.length;

    headerBuffer.writeUInt32BE(currentOffset, headerPos);
    headerPos += 4;

    headerBuffer.writeUInt32BE(e.length, headerPos);
    headerPos += 4;

    bodyBuffers.push(e.compressedData);
    currentOffset += e.length;
}

const finalBuffer = Buffer.concat([headerBuffer, ...bodyBuffers]);
fs.writeFileSync(outputFile, finalBuffer);

console.log(`\n🎉 [패킹 성공] 총 ${entries.length}개의 정령 음성을 단일 바이너리로 출력했습니다.`);
console.log(`- 아카이브 파일 경로: ${outputFile}`);
console.log(`- 패키지 크기: ${(finalBuffer.length / 1024).toFixed(2)} KB`);
