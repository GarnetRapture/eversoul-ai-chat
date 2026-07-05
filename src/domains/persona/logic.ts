import type { PersonaConfig, SpiritDetail, SpiritVisualAssets } from './types';
export const ASSET_ROOT = '/eversoul-assets';
const explicitAssetFolders: Record<string, string> = {
    'Ayame': 'Oyome',
    'Bryce': 'Blyce',
    'Catherine (Radiance)': 'CatherineBrave',
    'Cherrie': 'Catarina',
    'Cherrie (Romantic)': 'CherrieRoman',
    'Claire': 'Beatrice',
    'Flynn': 'Milia',
    'Garnet': 'Olivia',
    'Garnet (Rapture)': 'GarnetRapture',
    'Haru': 'Mia',
    'Haru (Kamuy)': 'HaruKamuy',
    'Honglan (Peerless)': 'HonglanCombat',
    'Kurumi Tokisaki': 'Tokisaki',
    'Mephistopheles': 'Mephisto',
    'Mephistopheles (Dawn)': 'MephistoDawn',
    'Miriam (Afterimage)': 'MiriamMirage',
    'Naiah': 'Nyah',
    'Petra (Awakened Soul)': 'PetraAwaken',
    'Renee': 'Leah',
    'Renee (Argent)': 'ReneeSilver',
    'Rose (Prominence)': 'RoseCrimson',
    'Sakuyo (Inferno)': 'SakuyoShin',
    'Sharinne': 'Sharing',
    'Soonie': 'Sunny',
    'Tohka Yatogami': 'Yatogami',
    'Violette': 'Amelia',
    'Yuria (Apollyon)': 'YuriaApollyon',
};
const knownAssetFolders = new Set([
    'Adrianne', 'Aira', 'Aki', 'Alisha', 'Amelia', 'Apollyon', 'AyameTsukuyomi',
    'Beatrice', 'Beleth', 'Blyce', 'Carnelian', 'Catarina', 'Catherine',
    'CatherineBrave', 'CherrieRoman', 'Chloe', 'Clara', 'Claudia',
    'ClaudiaArchangel', 'Daphne', 'Dominique', 'Dora', 'Edith', 'Eileen',
    'Erika', 'Erusha', 'Eve', 'GarnetRapture', 'Hanul', 'HaruKamuy', 'Hazel',
    'Honglan', 'HonglanCombat', 'Ina', 'Jacqueline', 'Jade', 'Jiho', 'JihoMir',
    'Joanne', 'Kanna', 'Karen', 'Larimar', 'Laura', 'Leah', 'Lewayne', 'Lilith', 'Linzy',
    'LinzyThanatos', 'Lizelotte', 'Lute', 'Manon', 'Melfice', 'Mephisto',
    'MephistoDawn', 'Meryl', 'Mia', 'Mica', 'Milia', 'Miriam', 'MiriamMirage', 'Nameless', 'Naomi',
    'Nia', 'Nicole', 'Nini', 'Nyah', 'Olivia', 'Onyx', 'Otoha', 'Oyome',
    'Petra', 'PetraAwaken', 'Prim', 'Rebecca', 'ReneeSilver', 'Rita', 'RoseCrimson', 'Rose', 'Ruri',
    'Sakuyo', 'SakuyoShin', 'Seeha', 'Sigrid', 'Sunny', 'Talia', 'Tasha',
    'Velanna', 'Vivienne', 'Weiss', 'Wheri', 'Xiaolian', 'Yuria',
    'YuriaApollyon', 'YuriaQueen',
]);
const raceBackgrounds: Record<string, string> = {
    '인간형': 'Talk_BG_StreetCafe.png',
    '요정형': 'Talk_BG_LuckyFlowerField.png',
    '야수형': 'Talk_BG_Forest_Day.png',
    '불사형': 'Talk_BG_NightCity.png',
    '천사형': 'Talk_BG_Castleout.png',
    '악마형': 'Talk_BG_Galaxy.png',
};
export function parseSpiritDetail(persona: PersonaConfig): SpiritDetail {
    return JSON.parse(persona.raw_json) as SpiritDetail;
}
export function resolveSpiritAssetFolder(nameEn: string): string | null {
    const explicit = explicitAssetFolders[nameEn];
    if (explicit) {
        return explicit;
    }
    const compact = nameEn.replace(/[^a-zA-Z0-9]/g, '');
    if (knownAssetFolders.has(compact)) {
        return compact;
    }
    return null;
}
export function getSpiritVisualAssets(detail: SpiritDetail): SpiritVisualAssets {
    const assetFolder = resolveSpiritAssetFolder(detail.name_en);
    const backgroundFile = raceBackgrounds[detail.race] ?? 'Talk_BG_Lounge.png';
    if (!assetFolder) {
        return {
            assetFolder,
            avatarCandidates: [],
            portraitCandidates: [],
            background: `${ASSET_ROOT}/backgrounds/talk/${backgroundFile}`,
        };
    }
    return {
        assetFolder,
        avatarCandidates: [
            `${ASSET_ROOT}/spirits/${assetFolder}/base/${assetFolder}_512.png`,
            `${ASSET_ROOT}/spirits/${assetFolder}/base/${assetFolder}_1024.png`,
        ],
        portraitCandidates: [
            `${ASSET_ROOT}/spirits/${assetFolder}/base/${assetFolder}_2048.png`,
            `${ASSET_ROOT}/spirits/${assetFolder}/costume/${assetFolder}_Costume01_2048.png`,
            `${ASSET_ROOT}/spirits/${assetFolder}/raid/${assetFolder}_Raid_2048.png`,
        ],
        background: `${ASSET_ROOT}/backgrounds/talk/${backgroundFile}`,
    };
}
export function getRaceTone(race: string): string {
    switch (race) {
        case '인간형':
            return 'tone-human';
        case '요정형':
            return 'tone-fairy';
        case '야수형':
            return 'tone-beast';
        case '불사형':
            return 'tone-undead';
        case '천사형':
            return 'tone-angel';
        case '악마형':
            return 'tone-demon';
        default:
            return 'tone-neutral';
    }
}
