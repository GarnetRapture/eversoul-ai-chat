import { copyFile, cp, mkdir, rm, stat } from 'node:fs/promises';
import { constants } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const portableDir = path.join(rootDir, 'build');
const releaseExePath = path.join(
  rootDir,
  'src-tauri',
  'target',
  'release',
  'eversoul-ai-chat.exe',
);
const portableExePath = path.join(portableDir, 'eversoul-ai-chat.exe');
const sourceDataDir = path.join(rootDir, 'data');
const portableDataDir = path.join(portableDir, 'data');
const portableModelDir = path.join(portableDir, 'ai', 'model');

async function ensureFile(filePath, label) {
  const entry = await stat(filePath).catch(() => null);
  if (!entry?.isFile()) {
    throw new Error(`${label} 파일을 찾을 수 없습니다: ${filePath}`);
  }
}

async function ensureDirectory(dirPath, label) {
  const entry = await stat(dirPath).catch(() => null);
  if (!entry?.isDirectory()) {
    throw new Error(`${label} 폴더를 찾을 수 없습니다: ${dirPath}`);
  }
}

await ensureFile(releaseExePath, 'Tauri release exe');
await ensureDirectory(sourceDataDir, 'data');

await rm(portableDir, { recursive: true, force: true });
await mkdir(portableDir, { recursive: true });
await copyFile(releaseExePath, portableExePath, constants.COPYFILE_FICLONE);
await cp(sourceDataDir, portableDataDir, {
  recursive: true,
  force: true,
  errorOnExist: false,
});
await mkdir(portableModelDir, { recursive: true });

console.info(`포터블 빌드 생성 완료: ${portableDir}`);
console.info(`- exe: ${portableExePath}`);
console.info(`- data: ${portableDataDir}`);
console.info(`- model folder: ${portableModelDir}`);
