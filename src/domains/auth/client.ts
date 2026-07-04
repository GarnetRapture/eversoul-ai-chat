import { invokeCommand, tauriCommands } from '../../shared/api';
import { UserSession } from './types';
export const authClient = {
    async login(email: string, token: string): Promise<UserSession> {
        return invokeCommand<UserSession>(tauriCommands.auth.login, { email, token });
    },
    async logout(): Promise<void> {
        return invokeCommand<void>(tauriCommands.auth.logout);
    },
    async getSession(): Promise<UserSession | null> {
        return invokeCommand<UserSession | null>(tauriCommands.auth.getSession);
    },
};
