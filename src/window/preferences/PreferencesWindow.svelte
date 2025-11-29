<script lang="ts">
    import { appWindow } from "@tauri-apps/api/window";
    import LineButton from "../widgets/LineButton.svelte";
    import StyledCheckbox from "./StyledCheckbox.svelte";
    import ColorPicker from "./ColorPicker.svelte";
    import type { Preferences } from "../../core/types";
    import { getDefaultPreferences } from "../../core/util";
    import * as ipc from "../../core/ipc";

    let preferences: Preferences;
    let error: string;
    let activeTab: 'general' | 'colors' = 'general';

    function init() {
        ipc.getPreferences().then((p: Preferences) => {
            const defaults = getDefaultPreferences();
            preferences = {
                ...defaults,
                ...p,
                colors: { ...defaults.colors, ...p.colors },
                filters: { ...defaults.filters, ...p.filters },
                sorting: { ...defaults.sorting, ...p.sorting }
            };
        });
    }

    function confirm() {
        ipc.setPreferences(preferences)
            .then(() => appWindow.close())
            .catch((e) => {
                error = e.message ?? e;
                appWindow.show();
            });

        appWindow.hide();
    }

    function resetColors() {
        const defaults = getDefaultPreferences();
        preferences.colors = { ...defaults.colors };
    }

    function resetFilters() {
        const defaults = getDefaultPreferences();
        preferences.filters = { ...defaults.filters };
        preferences.sorting = { ...defaults.sorting };
    }


    init();
</script>

<main>
    <h1>Preferences</h1>
    {#if preferences}
        <div class="preferences">
            {#if error}
                <p class="error">{error}</p>
            {/if}
            
            <div class="tabs">
                <button
                    class="tab"
                    class:active={activeTab === 'general'}
                    on:click={() => activeTab = 'general'}
                >
                    General
                </button>
                <button
                    class="tab"
                    class:active={activeTab === 'colors'}
                    on:click={() => activeTab = 'colors'}
                >
                    Colors
                </button>
            </div>

            <div class="tab-content">
                {#if activeTab === 'general'}
                    <div class="preference">
                        <StyledCheckbox bind:checked={preferences.enableOverlay}
                            >Enable overlay</StyledCheckbox
                        >
                    </div>
                    <div class="preference-group">
                        <div class="preference">
                            <StyledCheckbox
                                bind:checked={preferences.displayDailyClears}
                                disabled={!preferences.enableOverlay}
                                >Display daily clears</StyledCheckbox
                            >
                        </div>
                        <div class="preference">
                            <StyledCheckbox
                                bind:checked={preferences.displayClearNotifications}
                                disabled={!preferences.enableOverlay}
                                >Display activity clear notifications</StyledCheckbox
                            >
                        </div>
                        <div class="preference">
                            <StyledCheckbox
                                bind:checked={preferences.displayMilliseconds}
                                disabled={!preferences.enableOverlay}
                                >Display timer milliseconds</StyledCheckbox
                            >
                        </div>
                    </div>
                {:else if activeTab === 'colors'}
                    <div class="color-section">
                        <h2>Completion Dots</h2>
                        <ColorPicker
                            label="Completed Activity"
                            bind:value={preferences.colors.completedDotColor}
                        />
                        <ColorPicker
                            label="Incomplete Activity"
                            bind:value={preferences.colors.incompleteDotColor}
                        />
                        
                        <h2>App Appearance</h2>
                        <ColorPicker
                            label="App Background"
                            bind:value={preferences.colors.mapBackgroundColor}
                        />
                        <ColorPicker
                            label="Text Color"
                            bind:value={preferences.colors.textColor}
                        />
                        
                        <h2>Overlay & Notifications</h2>
                        <ColorPicker
                            label="Notification Background"
                            bind:value={preferences.colors.notificationBackgroundColor}
                        />
                        <ColorPicker
                            label="Text Background"
                            bind:value={preferences.colors.textBackgroundColor}
                        />
                        
                        <div class="reset-section">
                            <button class="reset-btn" on:click={resetColors}>
                                Reset to Defaults
                            </button>
                        </div>
                    </div>
                {/if}
            </div>
            
            <div class="actions">
                <LineButton clickCallback={confirm}>Confirm</LineButton>
            </div>
        </div>
    {/if}
</main>

<style>
    h1 {
        margin: 24px 48px;
    }

    .preferences {
        margin: 16px 48px;
    }

    .tabs {
        display: flex;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        margin-bottom: 20px;
    }

    .tab {
        padding: 8px 16px;
        background: none;
        border: none;
        color: #ccc;
        cursor: pointer;
        transition: all 0.2s;
        border-bottom: 2px solid transparent;
    }

    .tab:hover {
        color: #fff;
        background: rgba(255, 255, 255, 0.05);
    }

    .tab.active {
        color: #fff;
        border-bottom-color: var(--primary-highlight);
    }

    .tab-content {
        min-height: 300px;
    }

    .preference-group {
        padding: 8px 12px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .preference {
        margin: 12px 8px;
    }

    .color-section h2 {
        font-size: 16px;
        font-weight: 500;
        color: #fff;
        margin: 20px 0 12px 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        padding-bottom: 8px;
    }

    .color-section h2:first-child {
        margin-top: 0;
    }

    .reset-section {
        margin-top: 20px;
        padding-top: 16px;
        border-top: 1px solid rgba(255, 255, 255, 0.1);
    }

    .reset-btn {
        padding: 6px 12px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        color: #ccc;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
    }

    .reset-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
        border-color: var(--primary-highlight);
    }

    .error {
        color: var(--error);
    }

    .actions {
        margin-top: 24px;
        float: right;
    }

</style>
