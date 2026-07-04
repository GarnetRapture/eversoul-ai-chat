export interface UserSession {
  token: string;
  email: string;
  username: string;
  created_at: string;
}

export type AuthError =
  | { Database: string }
  | { Network: string }
  | 'InvalidCredentials'
  | { Unknown: string };
