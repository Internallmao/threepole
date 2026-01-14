import "../core/global.css";
import "./window.css";
import ProfilesWindow from "./profiles/ProfilesWindow.svelte";
import PreferencesWindow from "./preferences/PreferencesWindow.svelte";
import DetailsWindow from "./details/DetailsWindow.svelte";
import { appWindow } from "@tauri-apps/api/window";
import { getPreferences } from "../core/ipc";
import type { Preferences, TauriEvent } from "../core/types";
import { getDefaultPreferences } from "../core/util";
import { invoke } from "@tauri-apps/api/tauri";

function applyColorPreferences(preferences: Preferences) {
    const root = document.documentElement;

    if (preferences.colors.mapBackgroundColor) {
        root.style.setProperty('--app-background-color', preferences.colors.mapBackgroundColor);
    }
    if (preferences.colors.textColor) {
        root.style.setProperty('--app-text-color', preferences.colors.textColor);
        root.style.setProperty('--primary-highlight', preferences.colors.textColor);
        root.style.setProperty('--secondary-highlight', preferences.colors.textColor);
        root.style.setProperty('--primary-highlight-light', preferences.colors.textColor);
        root.style.setProperty('--secondary-highlight-light', preferences.colors.textColor);
        root.style.setProperty('--primary-highlight-lighter', preferences.colors.textColor);
        root.style.setProperty('--secondary-highlight-lighter', preferences.colors.textColor);
        root.style.setProperty('--overlay-text-color', preferences.colors.textColor);
    }
    if (preferences.colors.completedDotColor) {
        root.style.setProperty('--completed-dot-color', preferences.colors.completedDotColor);
    }
    if (preferences.colors.incompleteDotColor) {
        root.style.setProperty('--incomplete-dot-color', preferences.colors.incompleteDotColor);
    }
    if (preferences.colors.notificationBackgroundColor) {
        root.style.setProperty('--notification-background-color', preferences.colors.notificationBackgroundColor);
    }
    if (preferences.colors.textBackgroundColor) {
        root.style.setProperty('--text-background-color', preferences.colors.textBackgroundColor);
    }
}

async function initializeColors() {
    try {
        const preferences = await getPreferences();
        const defaults = getDefaultPreferences();
        const mergedPrefs = {
            ...defaults,
            ...preferences,
            colors: { ...defaults.colors, ...preferences.colors }
        };
        applyColorPreferences(mergedPrefs);
    } catch (error) {
        console.error('Failed to load color preferences:', error);
    }
}

window.addEventListener("DOMContentLoaded", async () => {
    await initializeColors();
    appWindow.show();
    appWindow.setFocus();
});

appWindow.listen("preferences_update", (event: TauriEvent<Preferences>) => {
    applyColorPreferences(event.payload);
});

document.addEventListener("contextmenu", (e) => e.preventDefault());

document.querySelector("#exit-button").addEventListener("click", () => appWindow.close());
document.querySelector("#minimize-button").addEventListener("click", () => appWindow.minimize());

const target = document.querySelector("#content");

const app = getWindowType();

function getWindowType() {
    let windowType = window.location.hash.split("#")[1];
    switch (windowType) {
        case "preferences":
            return new PreferencesWindow({
                target
            });
        case "profiles":
            return new ProfilesWindow({
                target
            });
        case "details":
            return new DetailsWindow({
                target
            });
        default:
            appWindow.close();
            break;
    }
}

export default app;
