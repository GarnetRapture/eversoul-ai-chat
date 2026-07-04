import { invoke } from '@tauri-apps/api/core';
import { UserSession } from './types';

export const authClient = {
  /**
   * 원격 API 서버를 통해 토큰을 검증하고 세션을 수립한 뒤 로컬 SQLite에 저장한다.
   */
  async login(email: string, token: string): Promise<UserSession> {
    return invoke<UserSession>('auth_login', { email, token });
  },

  /**
   * 세션을 삭제하여 로그아웃 처리를 수행한다.
   */
  async logout(): Promise<void> {
    return invoke<void>('auth_logout');
  },

  /**
   * 현재 로컬 데이터베이스에 기록된 유효한 세션을 확인한다.
   */
  async getSession(): Promise<UserSession | null> {
    return invoke<UserSession | null>('auth_get_session');
  },
};
