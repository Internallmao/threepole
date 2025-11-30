export type TauriEvent<T> = {
    payload: T
};

export type BungieProfile = {
    membershipType: number;
    membershipId: string;
    bungieGlobalDisplayName: string;
    bungieGlobalDisplayNameCode: number;
    crossSaveOverride: number;
};

export type Profiles = {
    savedProfiles: Profile[],
    selectedProfile: Profile,
}

export type Profile = {
    accountPlatform: number;
    accountId: string;
};

export type ProfileInfo = {
    privacy: number;
    displayName: string;
    displayTag: number;
    characterIds: string[];
};

export type ColorPreferences = {
    completedDotColor: string;
    incompleteDotColor: string;
    notificationBackgroundColor: string;
    textBackgroundColor: string;
    textColor: string;
    mapBackgroundColor: string;
};

export type FilterPreferences = {
    showRaids: boolean;
    showDungeons: boolean;
    showStrikes: boolean;
    showLostSectors: boolean;
    showCompleted: boolean;
    showIncomplete: boolean;
    showFreshStart: boolean;
    showCheckpoint: boolean;
    minDurationSeconds: number | null;
    maxDurationSeconds: number | null;
    specificRaids: {
        [activityHash: number]: boolean;
    };
    specificDungeons: {
        [activityHash: number]: boolean;
    };
};

export type SortPreferences = {
    sortBy: 'time' | 'duration' | 'activity';
    sortOrder: 'asc' | 'desc';
    timeRange: 'all' | 'today' | 'week' | 'month';
};

export type Preferences = {
    enableOverlay: boolean;
    displayDailyClears: boolean;
    displayClearNotifications: boolean;
    displayMilliseconds: boolean;
    colors: ColorPreferences;
    filters: FilterPreferences;
    sorting: SortPreferences;
};

export type PlayerDataStatus = {
    lastUpdate?: PlayerData | null,
    error?: string | null,
}

export type PlayerData = {
    currentActivity: CurrentActivity;
    activityHistory: CompletedActivity[];
    profileInfo: ProfileInfo;
};

export type CurrentActivity = {
    startDate: string;
    activityHash: number;
    activityInfo: ActivityInfo;
};

export type ActivityInfo = {
    name: string;
    activityModes: number[];
    backgroundImage: string;
};

export type CompletedActivity = {
    period: string;
    instanceId: string;
    completed: boolean;
    activityDuration: string;
    activityDurationSeconds: number;
    activityHash: number;
    modes: number[];
    completionReason: number;
    startingPhaseIndex?: number;
    activityWasStartedFromBeginning?: boolean;
};
