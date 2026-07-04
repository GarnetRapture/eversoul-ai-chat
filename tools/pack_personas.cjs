const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

// 상대 경로 교정
const sourceDir = path.join(__dirname, '..', 'data', 'personas');
const outputDir = path.join(__dirname, '..', 'src-tauri', 'resources');
const outputFile = path.join(outputDir, 'personas.bin');

if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

if (!fs.existsSync(sourceDir)) {
    console.error(`소스 경로가 존재하지 않습니다: ${sourceDir}`);
    process.exit(1);
}

const files = fs.readdirSync(sourceDir).filter(f => f.endsWith('.json'));
console.log(`총 ${files.length}개의 JSON 페르소나 데이터 수집 완료.`);

const entries = [];

for (const file of files) {
    const filePath = path.join(sourceDir, file);
    const rawData = fs.readFileSync(filePath, 'utf8');
    const nameEn = path.basename(file, '.json');

    try {
        JSON.parse(rawData);
    } catch (e) {
        console.warn(`[경고] 유효하지 않은 JSON 스킵: ${file}`);
        continue;
    }

    const compressed = zlib.deflateSync(Buffer.from(rawData, 'utf8'));
    
    entries.push({
        name: nameEn,
        length: compressed.length,
        compressedData: compressed
    });
}

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

console.log(`\n🎉 [패킹 성공] 총 ${entries.length}개의 정령 데이터를 완벽하게 압축하여 단일 바이너리로 출력했습니다.`);
console.log(`- 아카이브 파일 경로: ${outputFile}`);
console.log(`- 압축 전 전체 크기: ${(files.reduce((acc, f) => acc + fs.statSync(path.join(sourceDir, f)).size, 0) / 1024).toFixed(2)} KB`);
console.log(`- 압축 후 패키지 크기: ${(finalBuffer.length / 1024).toFixed(2)} KB`);
