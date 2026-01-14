<script lang="ts">
    import { appWindow } from "@tauri-apps/api/window";
    import LineButton from "../widgets/LineButton.svelte";
    import StyledCheckbox from "./StyledCheckbox.svelte";
    import ColorPicker from "./ColorPicker.svelte";
    import type { Preferences } from "../../core/types";
    import { getDefaultPreferences } from "../../core/util";
    import { getUniqueRaids, getUniqueDungeons } from "../../core/activities";
    import * as ipc from "../../core/ipc";

    let preferences: Preferences;
    let error: string;
    let activeTab: 'general' | 'colors' | 'filter' | 'sort' | 'duration' = 'general';

    // Duration state
    let minDurationMinutes: string = "";
    let minDurationSeconds: string = "";
    let maxDurationMinutes: string = "";
    let maxDurationSeconds: string = "";

    // Specific activity visibility
    let showSpecificRaids = false;
    let showSpecificDungeons = false;

    const uniqueRaids = getUniqueRaids();
    const uniqueDungeons = getUniqueDungeons();

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

            // Initialize duration inputs from saved preferences
            if (preferences.filters.minDurationSeconds !== null && preferences.filters.minDurationSeconds !== undefined) {
                const minutes = Math.floor(preferences.filters.minDurationSeconds / 60);
                const seconds = preferences.filters.minDurationSeconds % 60;
                minDurationMinutes = minutes > 0 ? String(minutes) : "";
                minDurationSeconds = seconds > 0 ? String(seconds) : "";
            }

            if (preferences.filters.maxDurationSeconds !== null && preferences.filters.maxDurationSeconds !== undefined) {
                const minutes = Math.floor(preferences.filters.maxDurationSeconds / 60);
                const seconds = preferences.filters.maxDurationSeconds % 60;
                maxDurationMinutes = minutes > 0 ? String(minutes) : "";
                maxDurationSeconds = seconds > 0 ? String(seconds) : "";
            }

            // Initialize specificRaids and specificDungeons if not set
            if (!preferences.filters.specificRaids) {
                preferences.filters.specificRaids = {};
            }
            if (!preferences.filters.specificDungeons) {
                preferences.filters.specificDungeons = {};
            }
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

    function handleMinDurationChange() {
        const minutes = parseInt(minDurationMinutes) || 0;
        const seconds = parseInt(minDurationSeconds) || 0;
        const totalSeconds = minutes * 60 + seconds;

        if (totalSeconds > 0) {
            preferences.filters.minDurationSeconds = totalSeconds;
        } else {
            preferences.filters.minDurationSeconds = null;
        }
    }

    function handleMaxDurationChange() {
        const minutes = parseInt(maxDurationMinutes) || 0;
        const seconds = parseInt(maxDurationSeconds) || 0;
        const totalSeconds = minutes * 60 + seconds;

        if (totalSeconds > 0) {
            preferences.filters.maxDurationSeconds = totalSeconds;
        } else {
            preferences.filters.maxDurationSeconds = null;
        }
    }

    function clearMinDuration() {
        minDurationMinutes = "";
        minDurationSeconds = "";
        preferences.filters.minDurationSeconds = null;
    }

    function clearMaxDuration() {
        maxDurationMinutes = "";
        maxDurationSeconds = "";
        preferences.filters.maxDurationSeconds = null;
    }

    function toggleSpecificRaids() {
        showSpecificRaids = !showSpecificRaids;
    }

    function toggleSpecificDungeons() {
        showSpecificDungeons = !showSpecificDungeons;
    }

    function handleSpecificRaidChange(allHashes: number[], enabled: boolean) {
        if (!preferences.filters.specificRaids) {
            preferences.filters.specificRaids = {};
        }

        for (const hash of allHashes) {
            preferences.filters.specificRaids[hash] = enabled;
        }

        if (enabled) {
            preferences.filters.showRaids = true;
        } else {
            const hasAnyRaidSelected = Object.values(preferences.filters.specificRaids).some(selected => selected);
            if (!hasAnyRaidSelected) {
                preferences.filters.showRaids = false;
            }
        }
    }

    function handleSpecificDungeonChange(activityHash: number, enabled: boolean) {
        if (!preferences.filters.specificDungeons) {
            preferences.filters.specificDungeons = {};
        }

        preferences.filters.specificDungeons[activityHash] = enabled;

        if (enabled) {
            preferences.filters.showDungeons = true;
        } else {
            const hasAnyDungeonSelected = Object.values(preferences.filters.specificDungeons).some(selected => selected);
            if (!hasAnyDungeonSelected) {
                preferences.filters.showDungeons = false;
            }
        }
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
                <button
                    class="tab"
                    class:active={activeTab === 'filter'}
                    on:click={() => activeTab = 'filter'}
                >
                    Filter
                </button>
                <button
                    class="tab"
                    class:active={activeTab === 'sort'}
                    on:click={() => activeTab = 'sort'}
                >
                    Sort
                </button>
                <button
                    class="tab"
                    class:active={activeTab === 'duration'}
                    on:click={() => activeTab = 'duration'}
                >
                    Duration
                </button>
            </div>

            <div class="tab-content" class:compact={activeTab === 'general' || activeTab === 'duration'}>
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
                    </div>
                {:else if activeTab === 'filter'}
                    <div class="filter-section">
                        <h2>Activity Types</h2>
                        <div class="filter-group">
                            <div class="activity-type-row">
                                <StyledCheckbox bind:checked={preferences.filters.showRaids}>
                                    Raids
                                </StyledCheckbox>
                                {#if preferences.filters.showRaids}
                                    <button class="specific-btn" on:click={toggleSpecificRaids}>
                                        {showSpecificRaids ? '−' : '+'}
                                    </button>
                                {/if}
                            </div>

                            {#if showSpecificRaids && preferences.filters.showRaids}
                                <div class="specific-activities">
                                    {#each uniqueRaids as raid}
                                        <label class="specific-label">
                                            <input
                                                type="checkbox"
                                                checked={preferences.filters.specificRaids[raid.hash] || false}
                                                on:change={(e) => handleSpecificRaidChange(raid.allHashes, e.target.checked)}
                                            />
                                            {raid.name}
                                        </label>
                                    {/each}
                                </div>
                            {/if}

                            <div class="activity-type-row">
                                <StyledCheckbox bind:checked={preferences.filters.showDungeons}>
                                    Dungeons
                                </StyledCheckbox>
                                {#if preferences.filters.showDungeons}
                                    <button class="specific-btn" on:click={toggleSpecificDungeons}>
                                        {showSpecificDungeons ? '−' : '+'}
                                    </button>
                                {/if}
                            </div>

                            {#if showSpecificDungeons && preferences.filters.showDungeons}
                                <div class="specific-activities">
                                    {#each uniqueDungeons as dungeon}
                                        <label class="specific-label">
                                            <input
                                                type="checkbox"
                                                checked={preferences.filters.specificDungeons[dungeon.hash] || false}
                                                on:change={(e) => handleSpecificDungeonChange(dungeon.hash, e.target.checked)}
                                            />
                                            {dungeon.name}
                                        </label>
                                    {/each}
                                </div>
                            {/if}

                            <StyledCheckbox bind:checked={preferences.filters.showStrikes}>
                                Strikes
                            </StyledCheckbox>
                            <StyledCheckbox bind:checked={preferences.filters.showLostSectors}>
                                Lost Sectors
                            </StyledCheckbox>
                        </div>

                        <h2>Completion Status</h2>
                        <div class="filter-group">
                            <StyledCheckbox bind:checked={preferences.filters.showCompleted}>
                                Completed
                            </StyledCheckbox>
                            <StyledCheckbox bind:checked={preferences.filters.showIncomplete}>
                                Incomplete
                            </StyledCheckbox>
                        </div>

                        <h2>Completion Type</h2>
                        <div class="filter-group">
                            <StyledCheckbox bind:checked={preferences.filters.showFreshStart}>
                                Fresh Start
                            </StyledCheckbox>
                            <StyledCheckbox bind:checked={preferences.filters.showCheckpoint}>
                                From Checkpoint
                            </StyledCheckbox>
                        </div>
                    </div>
                {:else if activeTab === 'sort'}
                    <div class="sort-section">
                        <h2>Sort By</h2>
                        <div class="sort-group">
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.sortBy} value="time" />
                                Time Started
                            </label>
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.sortBy} value="duration" />
                                Duration
                            </label>
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.sortBy} value="activity" />
                                Activity Name
                            </label>
                        </div>

                        <h2>Order</h2>
                        <div class="sort-group">
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.sortOrder} value="desc" />
                                Descending
                            </label>
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.sortOrder} value="asc" />
                                Ascending
                            </label>
                        </div>

                        <h2>Time Range</h2>
                        <div class="sort-group">
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.timeRange} value="today" />
                                Today
                            </label>
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.timeRange} value="week" />
                                This Week
                            </label>
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.timeRange} value="month" />
                                This Month
                            </label>
                            <label class="radio-label">
                                <input type="radio" bind:group={preferences.sorting.timeRange} value="all" />
                                All Time
                            </label>
                        </div>
                    </div>
                {:else if activeTab === 'duration'}
                    <div class="duration-section">
                        <h2>Minimum Duration</h2>
                        <div class="duration-group">
                            <div class="duration-input-group">
                                <div class="duration-inputs">
                                    <div class="duration-field">
                                        <input
                                            type="number"
                                            min="0"
                                            max="999"
                                            placeholder="0"
                                            bind:value={minDurationMinutes}
                                            on:input={handleMinDurationChange}
                                        />
                                        <span class="duration-label">min</span>
                                    </div>
                                    <div class="duration-field">
                                        <input
                                            type="number"
                                            min="0"
                                            max="59"
                                            placeholder="0"
                                            bind:value={minDurationSeconds}
                                            on:input={handleMinDurationChange}
                                        />
                                        <span class="duration-label">sec</span>
                                    </div>
                                </div>
                                {#if preferences.filters.minDurationSeconds !== null && preferences.filters.minDurationSeconds !== undefined}
                                    <button class="clear-duration-btn" on:click={clearMinDuration} title="Clear minimum duration">
                                        ✕
                                    </button>
                                {/if}
                            </div>
                        </div>

                        <h2>Maximum Duration</h2>
                        <div class="duration-group">
                            <div class="duration-input-group">
                                <div class="duration-inputs">
                                    <div class="duration-field">
                                        <input
                                            type="number"
                                            min="0"
                                            max="999"
                                            placeholder="0"
                                            bind:value={maxDurationMinutes}
                                            on:input={handleMaxDurationChange}
                                        />
                                        <span class="duration-label">min</span>
                                    </div>
                                    <div class="duration-field">
                                        <input
                                            type="number"
                                            min="0"
                                            max="59"
                                            placeholder="0"
                                            bind:value={maxDurationSeconds}
                                            on:input={handleMaxDurationChange}
                                        />
                                        <span class="duration-label">sec</span>
                                    </div>
                                </div>
                                {#if preferences.filters.maxDurationSeconds !== null && preferences.filters.maxDurationSeconds !== undefined}
                                    <button class="clear-duration-btn" on:click={clearMaxDuration} title="Clear maximum duration">
                                        ✕
                                    </button>
                                {/if}
                            </div>
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
        padding: 10px 12px;
        background: none;
        border: none;
        color: #ccc;
        cursor: pointer;
        transition: all 0.2s;
        border-bottom: 2px solid transparent;
        font-size: 13px;
        white-space: nowrap;
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
        min-height: 280px;
        overflow-y: auto;
    }

    .tab-content.compact {
        min-height: auto;
        overflow-y: hidden;
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

    .error {
        color: var(--error);
    }

    .actions {
        margin-top: 12px;
        float: right;
    }

    /* Filter, Sort, Duration section styles */
    .filter-section h2,
    .sort-section h2,
    .duration-section h2 {
        font-size: 14px;
        font-weight: 500;
        color: #fff;
        margin: 20px 0 12px 0;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        padding-bottom: 8px;
    }

    .filter-section h2:first-child,
    .sort-section h2:first-child,
    .duration-section h2:first-child {
        margin-top: 0;
    }

    .filter-group,
    .sort-group,
    .duration-group {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 8px;
        margin-bottom: 16px;
    }

    .filter-group:last-child,
    .sort-group:last-child,
    .duration-group:last-child {
        margin-bottom: 0;
    }

    .activity-type-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        align-self: stretch;
    }

    .specific-btn {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 3px;
        color: #ccc;
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 14px;
        font-weight: bold;
        line-height: 14px;
        text-align: center;
        padding: 0;
        padding-bottom: 2px;
        box-sizing: border-box;
        transition: all 0.2s;
    }

    .specific-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        color: #fff;
    }

    .specific-activities {
        margin-left: 28px;
        margin-top: 4px;
        margin-bottom: 8px;
        padding-left: 12px;
        border-left: 2px solid rgba(255, 255, 255, 0.1);
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .specific-label {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 13px;
        color: #aaa;
        cursor: pointer;
        transition: color 0.2s;
    }

    .specific-label:hover {
        color: #ddd;
    }

    .specific-label input[type="checkbox"] {
        width: 14px;
        height: 14px;
        accent-color: var(--primary-highlight);
    }

    .radio-label {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 14px;
        color: #ccc;
        cursor: pointer;
        transition: color 0.2s;
    }

    .radio-label:hover {
        color: #fff;
    }

    .radio-label input[type="radio"] {
        width: 14px;
        height: 14px;
        accent-color: var(--primary-highlight);
    }

    .duration-input-group {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        align-self: stretch;
    }

    .duration-inputs {
        display: flex;
        gap: 12px;
        flex: 1;
    }

    .duration-field {
        display: flex;
        align-items: center;
        gap: 6px;
        flex: 1;
    }

    .duration-field input {
        width: 100%;
        padding: 8px 10px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        color: #fff;
        font-size: 14px;
        font-family: inherit;
    }

    .duration-field input:focus {
        outline: none;
        border-color: var(--primary-highlight);
        background: rgba(255, 255, 255, 0.08);
    }

    .duration-field input::placeholder {
        color: #666;
    }

    .duration-label {
        font-size: 13px;
        color: #aaa;
        min-width: 28px;
    }

    .clear-duration-btn {
        background: rgba(255, 100, 100, 0.2);
        border: 1px solid rgba(255, 100, 100, 0.3);
        border-radius: 4px;
        color: #faa;
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 14px;
        transition: all 0.2s;
        flex-shrink: 0;
    }

    .clear-duration-btn:hover {
        background: rgba(255, 100, 100, 0.3);
        color: #fff;
    }
</style>
