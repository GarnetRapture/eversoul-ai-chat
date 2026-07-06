import { invoke } from '@tauri-apps/api/core';
import type { TauriCommandName } from './commands';
export async function invokeCommand<Response>(command: TauriCommandName, args?: Record<string, unknown>): Promise<Response> {
    console.info(`[eversoul-frontend] invoke:${command}:start`);
    try {
        const response = await invoke<Response>(command, args);
        console.info(`[eversoul-frontend] invoke:${command}:done`);
        return response;
    }
    catch (error) {
        console.error(`[eversoul-frontend] invoke:${command}:error`, error);
        throw error;
    }
}
