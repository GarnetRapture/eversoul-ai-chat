import { invokeCommand, tauriCommands } from '../../shared/api';
import type { ImportedModule, ModuleControl } from './types';

export const modulesClient = {
    async list(): Promise<ImportedModule[]> {
        return invokeCommand<ImportedModule[]>(tauriCommands.modules.list);
    },
    async importFromPath(path: string): Promise<ImportedModule> {
        return invokeCommand<ImportedModule>(tauriCommands.modules.importFromPath, { path });
    },
    async setEnabled(id: string, enabled: boolean): Promise<ImportedModule[]> {
        return invokeCommand<ImportedModule[]>(tauriCommands.modules.setEnabled, { id, enabled });
    },
    async updateControls(id: string, controls: ModuleControl[]): Promise<ImportedModule[]> {
        return invokeCommand<ImportedModule[]>(tauriCommands.modules.updateControls, { id, controls });
    },
    async delete(id: string): Promise<ImportedModule[]> {
        return invokeCommand<ImportedModule[]>(tauriCommands.modules.delete, { id });
    },
};
