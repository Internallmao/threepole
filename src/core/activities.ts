
export const KNOWN_RAIDS = {
    2122313384: "Last Wish",
    3213556450: "Scourge of the Past",
    2693136600: "Garden of Salvation",
    1042180643: "Garden of Salvation",
    910380154: "Deep Stone Crypt",
    3881495763: "Vault of Glass",
    1441982566: "Vow of the Disciple",
    1374392663: "King's Fall",
    2381413764: "Root of Nightmares",
    107319834: "Crota's End",
    1541433876: "Salvation's Edge",
    1044919065: "The Desert Perpetual",
    3817322389: "The Desert Perpetual (Epic)",
};

export const KNOWN_DUNGEONS = {
    2032534090: "The Shattered Throne",
    2582501063: "Pit of Heresy",
    1077850348: "Prophecy",
    4078656646: "Grasp of Avarice",
    2823159265: "Duality",
    1262462921: "Spire of the Watcher",
    313828469: "Ghosts of the Deep",
    300092127: "Vesper's Host",
    3834447244: "The Sundered Doctrine",
};

export function getRaidName(activityHash: number): string | null {
    return KNOWN_RAIDS[activityHash] || null;
}

export function getDungeonName(activityHash: number): string | null {
    return KNOWN_DUNGEONS[activityHash] || null;
}

export function isKnownRaid(activityHash: number): boolean {
    return activityHash in KNOWN_RAIDS;
}

export function isKnownDungeon(activityHash: number): boolean {
    return activityHash in KNOWN_DUNGEONS;
}

export function getUniqueRaids(): Array<{ hash: number; name: string; allHashes: number[] }> {
    const uniqueRaids = new Map<string, { hash: number; name: string; allHashes: number[] }>();
    
    for (const [hash, name] of Object.entries(KNOWN_RAIDS)) {
        const hashNum = parseInt(hash);
        if (uniqueRaids.has(name)) {
            uniqueRaids.get(name)!.allHashes.push(hashNum);
        } else {
            uniqueRaids.set(name, { hash: hashNum, name, allHashes: [hashNum] });
        }
    }
    
    return Array.from(uniqueRaids.values());
}

export function getUniqueDungeons(): Array<{ hash: number; name: string }> {
    const uniqueDungeons = new Map<string, { hash: number; name: string }>();
    
    for (const [hash, name] of Object.entries(KNOWN_DUNGEONS)) {
        const hashNum = parseInt(hash);
        if (!uniqueDungeons.has(name)) {
            uniqueDungeons.set(name, { hash: hashNum, name });
        }
    }
    
    return Array.from(uniqueDungeons.values());
}