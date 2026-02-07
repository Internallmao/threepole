import { ACTIVITY_TYPES } from "./consts";
import { KNOWN_RAIDS, KNOWN_DUNGEONS } from "./activities";
import type { ActivityInfo, CompletedActivity, FilterPreferences, SortPreferences } from "./types";

// Pre-build reverse lookup: raid name -> set of hashes with that name
const RAID_NAME_TO_HASHES: { [name: string]: Set<number> } = {};
for (const [hash, name] of Object.entries(KNOWN_RAIDS)) {
    if (!RAID_NAME_TO_HASHES[name]) {
        RAID_NAME_TO_HASHES[name] = new Set();
    }
    RAID_NAME_TO_HASHES[name].add(parseInt(hash));
}

export function getDestinyResetTime(date: Date = new Date()): Date {
    const resetTime = new Date(date);
    resetTime.setUTCHours(17, 0, 0, 0);

    if (date.getTime() < resetTime.getTime()) {
        resetTime.setUTCDate(resetTime.getUTCDate() - 1);
    }

    return resetTime;
}

export function getDestinyWeeklyResetTime(date: Date = new Date()): Date {
    const resetTime = getDestinyResetTime(date);

    const daysSinceLastTuesday = (resetTime.getUTCDay() + 5) % 7;
    resetTime.setUTCDate(resetTime.getUTCDate() - daysSinceLastTuesday);

    return resetTime;
}

export function countDailyClears(activityHistory: CompletedActivity[]): number {
    const dailyResetTime = getDestinyResetTime();
    let clearCount = 0;

    for (let activity of activityHistory) {
        if (activity.completed && new Date(activity.period) >= dailyResetTime) {
            clearCount++;
        }
    }

    return clearCount;
}

export function formatTime(millis: number): string {
    let seconds = Math.floor(millis / 1000);

    let minutes = Math.floor(seconds / 60);
    seconds = seconds - (minutes * 60);

    let hours = Math.floor(minutes / 60);
    minutes = minutes - (hours * 60);

    return (hours > 0 ? (hours + ":") : "") + String(minutes).padStart(2, "0") + ":" + String(seconds).padStart(2, "0");
}

export function formatMillis(millis: number): string {
    return ":" + String(millis % 1000).padStart(3, "0").substring(0, 2);
}

export function countClears(activityHistory: CompletedActivity[]): number {
    let clearCount = 0;
    for (let activity of activityHistory) {
        if (activity.completed) {
            clearCount++;
        }
    }

    return clearCount;
}

export function determineActivityType(modes: number[]): string | undefined {
    if (!modes) {
        return;
    }

    for (const mode of modes) {
        if (ACTIVITY_TYPES[mode]) {
            return ACTIVITY_TYPES[mode];
        }
    }
}

export function filterActivities(
    activities: CompletedActivity[],
    filters: FilterPreferences,
    activityInfoMap: { [hash: number]: ActivityInfo }
): CompletedActivity[] {
    return activities.filter(activity => {
        const activityType = determineActivityType(activity.modes);
        let typeMatch = false;

        switch (activityType) {
            case "Raid":
                if (!filters.showRaids) return false;

                if (filters.specificRaids && Object.keys(filters.specificRaids).length > 0) {
                    const hasSpecificRaidSelected = Object.values(filters.specificRaids).some(enabled => enabled);
                    if (hasSpecificRaidSelected) {
                        const activityName = KNOWN_RAIDS[activity.activityHash];
                        if (activityName) {
                            const hashesForName = RAID_NAME_TO_HASHES[activityName];
                            typeMatch = hashesForName
                                ? Array.from(hashesForName).some(h => filters.specificRaids[h] === true)
                                : false;
                        } else {
                            typeMatch = filters.specificRaids[activity.activityHash] === true;
                        }
                    } else {
                        typeMatch = false;
                    }
                } else {
                    typeMatch = true;
                }
                break;
            case "Dungeon":
                if (!filters.showDungeons) return false;

                if (filters.specificDungeons && Object.keys(filters.specificDungeons).length > 0) {
                    const hasSpecificDungeonSelected = Object.values(filters.specificDungeons).some(enabled => enabled);
                    if (hasSpecificDungeonSelected) {
                        typeMatch = filters.specificDungeons[activity.activityHash] === true;
                    } else {
                        typeMatch = false;
                    }
                } else {
                    typeMatch = true;
                }
                break;
            case "Strike":
                typeMatch = filters.showStrikes;
                break;
            case "Lost Sector":
                typeMatch = filters.showLostSectors;
                break;
            default:
                typeMatch = false;
        }

        if (!typeMatch) return false;

        // Apply checkpoint filtering (independent of completed/incomplete)
        if (activity.activityWasStartedFromBeginning !== undefined && activity.activityWasStartedFromBeginning !== null) {
            // activityWasStartedFromBeginning: true = fresh start, false = checkpoint
            if (activity.activityWasStartedFromBeginning === true && !filters.showFreshStart) return false;
            if (activity.activityWasStartedFromBeginning === false && !filters.showCheckpoint) return false;
        } else if (activity.startingPhaseIndex !== undefined) {
            // Fallback: startingPhaseIndex 0 = fresh start, >0 = checkpoint
            const isFreshStart = activity.startingPhaseIndex === 0;
            if (isFreshStart && !filters.showFreshStart) return false;
            if (!isFreshStart && !filters.showCheckpoint) return false;
        }

        // Then apply completed/incomplete filtering
        if (activity.completed && !filters.showCompleted) return false;
        if (!activity.completed && !filters.showIncomplete) return false;

        if (filters.minDurationSeconds !== null && filters.minDurationSeconds !== undefined) {
            if (activity.activityDurationSeconds < filters.minDurationSeconds) {
                return false;
            }
        }

        if (filters.maxDurationSeconds !== null && filters.maxDurationSeconds !== undefined) {
            if (activity.activityDurationSeconds > filters.maxDurationSeconds) {
                return false;
            }
        }

        return true;
    });
}

export function sortActivities(
    activities: CompletedActivity[],
    sorting: SortPreferences,
    activityInfoMap: { [hash: number]: ActivityInfo }
): CompletedActivity[] {
    const now = new Date();

    let filteredActivities = activities.filter(activity => {
        const activityDate = new Date(activity.period);

        switch (sorting.timeRange) {
            case "today":
                const dailyReset = getDestinyResetTime(now);
                return activityDate >= dailyReset;
            case "week":
                const weeklyReset = getDestinyWeeklyResetTime(now);
                return activityDate >= weeklyReset;
            case "month":
                const monthlyReset = getDestinyWeeklyResetTime(now);
                monthlyReset.setUTCDate(monthlyReset.getUTCDate() - 28);
                return activityDate >= monthlyReset;
            case "all":
            default:
                return true;
        }
    });

    return filteredActivities.sort((a, b) => {
        let comparison = 0;

        switch (sorting.sortBy) {
            case "time":
                comparison = new Date(a.period).getTime() - new Date(b.period).getTime();
                break;
            case "duration":
                comparison = a.activityDurationSeconds - b.activityDurationSeconds;
                break;
            case "activity":
                const aInfo = activityInfoMap[a.activityHash];
                const bInfo = activityInfoMap[b.activityHash];
                const aName = aInfo?.name || "";
                const bName = bInfo?.name || "";
                comparison = aName.localeCompare(bName);
                break;
        }

        return sorting.sortOrder === "desc" ? -comparison : comparison;
    });
}

export function getDefaultPreferences() {
    return {
        enableOverlay: false,
        displayDailyClears: true,
        displayClearNotifications: true,
        displayMilliseconds: false,
        colors: {
            completedDotColor: "#33ee33",
            incompleteDotColor: "#ee3333",
            notificationBackgroundColor: "#12171c",
            textBackgroundColor: "rgba(0, 0, 0, 0.7)",
            textColor: "#ffffff",
            mapBackgroundColor: "#12171c"
        },
        filters: {
            showRaids: true,
            showDungeons: true,
            showStrikes: true,
            showLostSectors: true,
            showCompleted: true,
            showIncomplete: true,
            showFreshStart: true,
            showCheckpoint: true,
            minDurationSeconds: null,
            maxDurationSeconds: null,
            specificRaids: {},
            specificDungeons: {}
        },
        sorting: {
            sortBy: "time" as const,
            sortOrder: "desc" as const,
            timeRange: "all" as const
        }
    };
}
