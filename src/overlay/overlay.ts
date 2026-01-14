import "../core/global.css"
import "./overlay.css"
import { appWindow } from "@tauri-apps/api/window";
import { createPopup as _createPopup, type Popup } from "./popups";
import type { TauriEvent, Preferences, CurrentActivity, PlayerDataStatus } from "../core/types";
import { countDailyClears, determineActivityType, formatMillis, formatTime, getDefaultPreferences } from "../core/util";
import { getPlayerdata, getPreferences, getActivityInfo } from "../core/ipc";

const widgetElem = document.querySelector<HTMLElement>("#widget")!;
const widgetContentElem = document.querySelector<HTMLElement>("#widget-content")!;
const timerElem = document.querySelector<HTMLElement>("#timer")!;
const timeElem = document.querySelector<HTMLElement>("#time")!;
const msElem = document.querySelector<HTMLElement>("#ms")!;
const counterElem = document.querySelector<HTMLElement>("#counter")!;
const dailyElem = document.querySelector<HTMLElement>("#daily")!;

let currentActivity: CurrentActivity;
let lastRaidId;
let doneInitialRefresh = false;

let shown = false;
let prefs: Preferences;
let timerInterval;

async function init() {
    appWindow.listen("show", () => {
        if (shown) {
            return;
        }

        appWindow.show();
        shown = true;

        checkTimerInterval();
    });

    appWindow.listen("hide", () => {
        if (!shown) {
            return;
        }

        appWindow.hide();
        shown = false;

        checkTimerInterval();
    });

    applyPreferences(await getPreferences());
    refresh(await getPlayerdata());

    appWindow.listen("preferences_update", (p: TauriEvent<Preferences>) => applyPreferences(p.payload));
    appWindow.listen("playerdata_update", (e: TauriEvent<PlayerDataStatus>) => refresh(e.payload));
}

async function fetchActivityName(activityHash: number): Promise<string | null> {
    try {
        const activityInfo = await getActivityInfo(activityHash);
        return activityInfo.name;
    } catch (error) {
        console.error("Failed to fetch activity name:", error);
        return null;
    }
}

function createPopup(popup: Popup) {
    _createPopup(popup, shown);
}

function checkTimerInterval() {
    if (!prefs || !shown || !determineActivityType(currentActivity?.activityInfo?.activityModes)) {
        clearTimeout(timerInterval);
        timerInterval = null;
        timerElem.classList.add("hidden");
        return;
    }

    timerElem.classList.remove("hidden");

    if (!timerInterval) {
        timerInterval = setInterval(() => requestAnimationFrame(timerTick), 1000 / 30);
    }
}

function refresh(playerDataStatus: PlayerDataStatus) {
    let playerData = playerDataStatus?.lastUpdate;

    if (!playerData) {
        widgetContentElem.classList.add("hidden");

        currentActivity = null;
        doneInitialRefresh = false;

        if (playerDataStatus?.error) {
            createPopup({ title: "Failed to fetch initial stats", subtext: playerDataStatus.error });
        }

        return;
    }

    widgetContentElem.classList.remove("hidden");

    currentActivity = playerData.currentActivity;

    checkTimerInterval();

    dailyElem.innerText = String(countDailyClears(playerData.activityHistory));

    let latestRaid = playerData.activityHistory[0];

    if (doneInitialRefresh && latestRaid?.completed && lastRaidId != latestRaid.instanceId && prefs.displayClearNotifications) {
        const type = determineActivityType(latestRaid.modes);

        if (type) {
            fetchActivityName(latestRaid.activityHash).then(activityName => {
                const displayName = activityName || `${type.charAt(0).toUpperCase() + type.slice(1)}`;
                createPopup({
                    title: `${displayName} completed!`,
                    subtext: `Clear time: <strong>${latestRaid.activityDuration}</strong>`
                });
            }).catch(() => {
                const typeFormatted = type.charAt(0).toUpperCase() + type.slice(1);
                createPopup({
                    title: `${typeFormatted} completed!`,
                    subtext: `Clear time: <strong>${latestRaid.activityDuration}</strong>`
                });
            });
        }
    }

    lastRaidId = latestRaid?.instanceId;

    if (!doneInitialRefresh) {
        createPopup({ title: `${playerData.profileInfo.displayName}#${playerData.profileInfo.displayTag}`, subtext: "Threepole is active." });
    }

    doneInitialRefresh = true;
}

function applyPreferences(p: Preferences) {
    const defaults = getDefaultPreferences();
    prefs = {
        ...defaults,
        ...p,
        colors: { ...defaults.colors, ...p.colors },
        filters: { ...defaults.filters, ...p.filters },
        sorting: { ...defaults.sorting, ...p.sorting }
    };

    if (prefs.displayDailyClears) {
        counterElem.classList.remove("hidden");
    } else {
        counterElem.classList.add("hidden");
    }

    if (prefs.displayMilliseconds) {
        msElem.classList.remove("hidden");
    } else {
        msElem.classList.add("hidden");
    }

    const root = document.documentElement;
    if (prefs.colors.textColor) {
        root.style.setProperty('--overlay-text-color', prefs.colors.textColor);
        root.style.setProperty('--overlay-icon-color', prefs.colors.textColor);
        root.style.setProperty('--overlay-secondary-color', prefs.colors.textColor);
        root.style.setProperty('--app-text-color', prefs.colors.textColor);
    }

    if (prefs.colors.mapBackgroundColor) {
        root.style.setProperty('--app-background-color', prefs.colors.mapBackgroundColor);
    }

    if (prefs.colors.notificationBackgroundColor) {
        root.style.setProperty('--notification-background-color', prefs.colors.notificationBackgroundColor);
    }

    if (prefs.colors.completedDotColor) {
        root.style.setProperty('--completion-dot-color', prefs.colors.completedDotColor);
    }

    if (prefs.colors.textBackgroundColor &&
        prefs.colors.textBackgroundColor !== "#000000" &&
        prefs.colors.textBackgroundColor !== "transparent" &&
        prefs.colors.textBackgroundColor !== "") {

        timerElem.style.backgroundColor = prefs.colors.textBackgroundColor;
        timerElem.style.borderRadius = "4px";
        timerElem.style.padding = "4px 8px";
        timerElem.style.marginBottom = "4px";

        counterElem.style.backgroundColor = prefs.colors.textBackgroundColor;
        counterElem.style.borderRadius = "4px";
        counterElem.style.padding = "4px 8px";
    } else {
        timerElem.style.backgroundColor = "transparent";
        timerElem.style.borderRadius = "";
        timerElem.style.padding = "";
        timerElem.style.marginBottom = "";

        counterElem.style.backgroundColor = "transparent";
        counterElem.style.borderRadius = "";
        counterElem.style.padding = "";
    }

    widgetElem.style.backgroundColor = "transparent";
    widgetContentElem.style.backgroundColor = "transparent";

    clearTimeout(timerInterval);
    timerInterval = null;

    checkTimerInterval();
}

function timerTick() {
    if (!currentActivity || !currentActivity.startDate) {
        timeElem.innerHTML = "00:00";
        msElem.innerHTML = "000";
        return;
    }

    let millis = Number(new Date()) - Number(new Date(currentActivity.startDate));
    timeElem.innerHTML = formatTime(millis);
    msElem.innerHTML = formatMillis(millis);
}

init();
