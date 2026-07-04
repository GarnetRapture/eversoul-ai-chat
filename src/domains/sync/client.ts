import { invoke } from '@tauri-apps/api/core';
import { SyncResult } from './types';

export const syncClient = {
  /**
   * 원격 API 서버로부터 페르소나 및 지식팩 데이터를 동기화하여 SQLite 로컬 DB에 적재한다.
   */
  async runSync(): Promise<SyncResult> {
    return invoke<SyncResult>('sync_run');
  },
};
