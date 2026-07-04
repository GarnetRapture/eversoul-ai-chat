export const tauriCommands = {
    auth: {
        login: 'auth_login',
        logout: 'auth_logout',
        getSession: 'auth_get_session',
    },
    chat: {
        createRoom: 'chat_create_room',
        createSessionRoom: 'chat_create_session_room',
        getLatestSessionRoom: 'chat_get_latest_session_room',
        listRooms: 'chat_list_rooms',
        listMessages: 'chat_list_messages',
        sendMessage: 'chat_send_message',
    },
    knowledge: {
        search: 'knowledge_search',
    },
    llm: {
        load: 'llm_load',
        status: 'llm_status',
        infer: 'llm_infer',
    },
    persona: {
        list: 'persona_list',
        update: 'persona_update',
        listArchive: 'persona_list_archive',
        getPack: 'persona_get_pack',
        selectPreset: 'persona_select_preset',
        getDefault: 'persona_get_default',
        setDefault: 'persona_set_default',
        bondRanking: 'persona_bond_ranking',
    },
    style: {
        list: 'style_list',
        update: 'style_update',
        selectActive: 'style_select_active',
        getActive: 'style_get_active',
    },
    sync: {
        run: 'sync_run',
    },
    settings: {
        get: 'settings_get',
        reset: 'settings_reset',
    },
    training: {
        run: 'training_run',
    },
} as const;
type CommandGroup = (typeof tauriCommands)[keyof typeof tauriCommands];
type CommandValue<T> = T extends Record<string, infer Value> ? Value : never;
export type TauriCommandName = CommandValue<CommandGroup>;
