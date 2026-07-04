export interface SyncResult {
  success: boolean;
  synced_items: number;
  error_message: string | null;
}

export type SyncError =
  | { Database: string }
  | { Network: string }
  | { Unknown: string };
