import { invoke } from '@tauri-apps/api/core';

import type { TauriCommandName } from './commands';

export function invokeCommand<Response>(
  command: TauriCommandName,
  args?: Record<string, unknown>
): Promise<Response> {
  return invoke<Response>(command, args);
}
