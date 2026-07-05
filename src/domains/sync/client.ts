import { invokeCommand, tauriCommands } from '../../shared/api';
import { LocalStatusSnapshot, SyncResult } from './types';
export const syncClient = {
    async runSync(): Promise<SyncResult> {
        return invokeCommand<SyncResult>(tauriCommands.sync.run);
    },
    async getLocalStatus(): Promise<LocalStatusSnapshot> {
        return invokeCommand<LocalStatusSnapshot>(tauriCommands.sync.localStatus);
    },
};
