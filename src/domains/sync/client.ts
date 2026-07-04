import { invokeCommand, tauriCommands } from '../../shared/api';
import { SyncResult } from './types';
export const syncClient = {
    async runSync(): Promise<SyncResult> {
        return invokeCommand<SyncResult>(tauriCommands.sync.run);
    },
};
