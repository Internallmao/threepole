<script lang="ts">
    import type { FilterPreferences, SortPreferences } from "../../core/types";
    import { ACTIVITY_TYPES } from "../../core/consts";
    import { KNOWN_RAIDS, KNOWN_DUNGEONS, getUniqueRaids, getUniqueDungeons } from "../../core/activities";

    export let filters: FilterPreferences;
    export let sorting: SortPreferences;
    export let onFiltersChange: (filters: FilterPreferences) => void;
    export let onSortingChange: (sorting: SortPreferences) => void;

    let showFilters = false;
    let showSorting = false;
    let showDuration = false;
    let showSpecificRaids = false;
    let showSpecificDungeons = false;
    
    let minDurationMinutes: string = "";
    let minDurationSeconds: string = "";
    let maxDurationMinutes: string = "";
    let maxDurationSeconds: string = "";

    const uniqueRaids = getUniqueRaids();
    const uniqueDungeons = getUniqueDungeons();

    function toggleFilters() {
        if (showSorting) showSorting = false;
        if (showDuration) showDuration = false;
        showFilters = !showFilters;
    }

    function toggleSorting() {
        if (showFilters) showFilters = false;
        if (showDuration) showDuration = false;
        showSorting = !showSorting;
    }
    
    function toggleDuration() {
        if (showFilters) showFilters = false;
        if (showSorting) showSorting = false;
        showDuration = !showDuration;
    }

    function handleFilterChange() {
        onFiltersChange(filters);
    }
    
    function handleMinDurationChange() {
        const minutes = parseInt(minDurationMinutes) || 0;
        const seconds = parseInt(minDurationSeconds) || 0;
        const totalSeconds = minutes * 60 + seconds;
        
        if (totalSeconds > 0) {
            filters.minDurationSeconds = totalSeconds;
        } else {
            filters.minDurationSeconds = null;
        }
        
        onFiltersChange(filters);
    }
    
    function handleMaxDurationChange() {
        const minutes = parseInt(maxDurationMinutes) || 0;
        const seconds = parseInt(maxDurationSeconds) || 0;
        const totalSeconds = minutes * 60 + seconds;
        
        if (totalSeconds > 0) {
            filters.maxDurationSeconds = totalSeconds;
        } else {
            filters.maxDurationSeconds = null;
        }
        
        onFiltersChange(filters);
    }
    
    function clearMinDuration() {
        minDurationMinutes = "";
        minDurationSeconds = "";
        filters.minDurationSeconds = null;
        onFiltersChange(filters);
    }
    
    function clearMaxDuration() {
        maxDurationMinutes = "";
        maxDurationSeconds = "";
        filters.maxDurationSeconds = null;
        onFiltersChange(filters);
    }

    function handleSortingChange() {
        onSortingChange(sorting);
    }

    function toggleSpecificRaids() {
        showSpecificRaids = !showSpecificRaids;
    }

    function toggleSpecificDungeons() {
        showSpecificDungeons = !showSpecificDungeons;
    }

    function handleSpecificRaidChange(allHashes: number[], enabled: boolean) {
        if (!filters.specificRaids) {
            filters.specificRaids = {};
        }
        
        for (const hash of allHashes) {
            filters.specificRaids[hash] = enabled;
        }
        
        if (enabled) {
            filters.showRaids = true;
        } else {
            const hasAnyRaidSelected = Object.values(filters.specificRaids).some(selected => selected);
            if (!hasAnyRaidSelected) {
                filters.showRaids = false;
            }
        }
        
        onFiltersChange(filters);
    }

    function handleSpecificDungeonChange(activityHash: number, enabled: boolean) {
        if (!filters.specificDungeons) {
            filters.specificDungeons = {};
        }
        
        filters.specificDungeons[activityHash] = enabled;
        
        if (enabled) {
            filters.showDungeons = true;
        } else {
            const hasAnyDungeonSelected = Object.values(filters.specificDungeons).some(selected => selected);
            if (!hasAnyDungeonSelected) {
                filters.showDungeons = false;
            }
        }
        
        onFiltersChange(filters);
    }

    $: if (!filters.specificRaids) {
        filters.specificRaids = {};
    }
    $: if (!filters.specificDungeons) {
        filters.specificDungeons = {};
    }
    
    $: if (filters.minDurationSeconds !== null && filters.minDurationSeconds !== undefined) {
        const minutes = Math.floor(filters.minDurationSeconds / 60);
        const seconds = filters.minDurationSeconds % 60;
        if (minDurationMinutes === "" && minDurationSeconds === "") {
            minDurationMinutes = minutes > 0 ? String(minutes) : "";
            minDurationSeconds = seconds > 0 ? String(seconds) : "";
        }
    }
    
    $: if (filters.maxDurationSeconds !== null && filters.maxDurationSeconds !== undefined) {
        const minutes = Math.floor(filters.maxDurationSeconds / 60);
        const seconds = filters.maxDurationSeconds % 60;
        if (maxDurationMinutes === "" && maxDurationSeconds === "") {
            maxDurationMinutes = minutes > 0 ? String(minutes) : "";
            maxDurationSeconds = seconds > 0 ? String(seconds) : "";
        }
    }
</script>

<div class="controls">
    <div class="control-buttons">
        <button class="control-btn" class:active={showFilters} on:click={toggleFilters}>
            <svg xmlns="http://www.w3.org/2000/svg" height="16" width="16">
                <path d="M6.5 14v-2h3v2zm-4-4v-2h11v2zm-2-4V4h15v2z"/>
            </svg>
            Filter
        </button>
        <button class="control-btn" class:active={showSorting} on:click={toggleSorting}>
            <svg xmlns="http://www.w3.org/2000/svg" height="16" width="16">
                <path d="M3 18v-2h4v2zm0-5v-2h7v2zm0-5V6h10v2z"/>
            </svg>
            Sort
        </button>
        <button class="control-btn" class:active={showDuration} on:click={toggleDuration}>
            <svg xmlns="http://www.w3.org/2000/svg" height="16" width="16">
                <path d="M8 14c-3.3 0-6-2.7-6-6s2.7-6 6-6 6 2.7 6 6-2.7 6-6 6zm0-10.5c-2.5 0-4.5 2-4.5 4.5s2 4.5 4.5 4.5 4.5-2 4.5-4.5-2-4.5-4.5-4.5zm.5 7h-1v-4h1v4z"/>
            </svg>
            Duration
        </button>
    </div>

    {#if showFilters}
        <div class="panel filters-panel">
            <h3>Activity Types</h3>
            <div class="filter-group">
                <div class="activity-type-row">
                    <label>
                        <input type="checkbox" bind:checked={filters.showRaids} on:change={handleFilterChange} />
                        Raids
                    </label>
                    {#if filters.showRaids}
                        <button class="specific-btn" on:click={toggleSpecificRaids} on:keypress={toggleSpecificRaids}>
                            {showSpecificRaids ? '−' : '+'}
                        </button>
                    {/if}
                </div>
                
                {#if showSpecificRaids && filters.showRaids}
                    <div class="specific-activities">
                        {#each uniqueRaids as raid}
                            <label class="specific-label">
                                <input
                                    type="checkbox"
                                    checked={filters.specificRaids[raid.hash] || false}
                                    on:change={(e) => handleSpecificRaidChange(raid.allHashes, e.target.checked)}
                                />
                                {raid.name}
                            </label>
                        {/each}
                    </div>
                {/if}

                <div class="activity-type-row">
                    <label>
                        <input type="checkbox" bind:checked={filters.showDungeons} on:change={handleFilterChange} />
                        Dungeons
                    </label>
                    {#if filters.showDungeons}
                        <button class="specific-btn" on:click={toggleSpecificDungeons} on:keypress={toggleSpecificDungeons}>
                            {showSpecificDungeons ? '−' : '+'}
                        </button>
                    {/if}
                </div>
                
                {#if showSpecificDungeons && filters.showDungeons}
                    <div class="specific-activities">
                        {#each uniqueDungeons as dungeon}
                            <label class="specific-label">
                                <input
                                    type="checkbox"
                                    checked={filters.specificDungeons[dungeon.hash] || false}
                                    on:change={(e) => handleSpecificDungeonChange(dungeon.hash, e.target.checked)}
                                />
                                {dungeon.name}
                            </label>
                        {/each}
                    </div>
                {/if}

                <label>
                    <input type="checkbox" bind:checked={filters.showStrikes} on:change={handleFilterChange} />
                    Strikes
                </label>
                <label>
                    <input type="checkbox" bind:checked={filters.showLostSectors} on:change={handleFilterChange} />
                    Lost Sectors
                </label>
            </div>
            
            <h3>Completion Status</h3>
            <div class="filter-group">
                <label>
                    <input type="checkbox" bind:checked={filters.showCompleted} on:change={handleFilterChange} />
                    Completed
                </label>
                <label>
                    <input type="checkbox" bind:checked={filters.showIncomplete} on:change={handleFilterChange} />
                    Incomplete
                </label>
            </div>
            
            <h3>Completion Type</h3>
            <div class="filter-group">
                <label>
                    <input type="checkbox" bind:checked={filters.showFreshStart} on:change={handleFilterChange} />
                    Fresh Start
                </label>
                <label>
                    <input type="checkbox" bind:checked={filters.showCheckpoint} on:change={handleFilterChange} />
                    From Checkpoint
                </label>
            </div>
        </div>
    {/if}

    {#if showSorting}
        <div class="panel sorting-panel">
            <h3>Sort By</h3>
            <div class="sort-group">
                <label>
                    <input type="radio" bind:group={sorting.sortBy} value="time" on:change={handleSortingChange} />
                    Time Started
                </label>
                <label>
                    <input type="radio" bind:group={sorting.sortBy} value="duration" on:change={handleSortingChange} />
                    Duration
                </label>
                <label>
                    <input type="radio" bind:group={sorting.sortBy} value="activity" on:change={handleSortingChange} />
                    Activity Name
                </label>
            </div>

            <h3>Order</h3>
            <div class="sort-group">
                <label>
                    <input type="radio" bind:group={sorting.sortOrder} value="desc" on:change={handleSortingChange} />
                    Descending
                </label>
                <label>
                    <input type="radio" bind:group={sorting.sortOrder} value="asc" on:change={handleSortingChange} />
                    Ascending
                </label>
            </div>

            <h3>Time Range</h3>
            <div class="sort-group">
                <label>
                    <input type="radio" bind:group={sorting.timeRange} value="today" on:change={handleSortingChange} />
                    Today
                </label>
                <label>
                    <input type="radio" bind:group={sorting.timeRange} value="week" on:change={handleSortingChange} />
                    This Week
                </label>
                <label>
                    <input type="radio" bind:group={sorting.timeRange} value="month" on:change={handleSortingChange} />
                    This Month
                </label>
                <label>
                    <input type="radio" bind:group={sorting.timeRange} value="all" on:change={handleSortingChange} />
                    All Time
                </label>
            </div>
        </div>
    {/if}
    
    {#if showDuration}
        <div class="panel duration-panel">
            <h3>Minimum Duration</h3>
            <div class="filter-group">
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
                    {#if filters.minDurationSeconds !== null && filters.minDurationSeconds !== undefined}
                        <button class="clear-duration-btn" on:click={clearMinDuration} on:keypress={clearMinDuration} title="Clear minimum duration">
                            ✕
                        </button>
                    {/if}
                </div>
                <p class="duration-hint">Show only activities that took at least this long</p>
            </div>
            
            <h3>Maximum Duration</h3>
            <div class="filter-group">
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
                    {#if filters.maxDurationSeconds !== null && filters.maxDurationSeconds !== undefined}
                        <button class="clear-duration-btn" on:click={clearMaxDuration} on:keypress={clearMaxDuration} title="Clear maximum duration">
                            ✕
                        </button>
                    {/if}
                </div>
                <p class="duration-hint">Show only activities completed within this time</p>
            </div>
        </div>
    {/if}
</div>

<style>
    .controls {
        margin-bottom: 16px;
    }

    .control-buttons {
        display: flex;
        gap: 8px;
        margin-bottom: 8px;
    }

    .control-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 6px 12px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        color: #ccc;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
        fill: #ccc;
    }

    .control-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
        fill: #fff;
    }

    .control-btn.active {
        background: var(--primary-highlight);
        border-color: var(--primary-highlight);
        color: #fff;
        fill: #fff;
    }

    .panel {
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        padding: 12px;
        margin-bottom: 8px;
    }

    h3 {
        font-size: 12px;
        font-weight: 500;
        color: #fff;
        margin-bottom: 8px;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .filter-group,
    .sort-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 12px;
    }

    .filter-group:last-child,
    .sort-group:last-child {
        margin-bottom: 0;
    }

    label {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 13px;
        color: #ccc;
        cursor: pointer;
        transition: color 0.2s;
    }

    label:hover {
        color: #fff;
    }

    input[type="checkbox"],
    input[type="radio"] {
        width: 14px;
        height: 14px;
        accent-color: var(--primary-highlight);
    }

    .activity-type-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
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
        font-size: 12px;
        font-weight: bold;
        transition: all 0.2s;
    }

    .specific-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        color: #fff;
    }

    .specific-activities {
        margin-left: 20px;
        margin-top: 6px;
        margin-bottom: 6px;
        padding-left: 12px;
        border-left: 2px solid rgba(255, 255, 255, 0.1);
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .specific-label {
        font-size: 12px;
        color: #aaa;
    }

    .specific-label:hover {
        color: #ddd;
    }
    
    .duration-input-group {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    
    .duration-inputs {
        display: flex;
        gap: 8px;
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
        padding: 6px 8px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        color: #fff;
        font-size: 13px;
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
        font-size: 12px;
        color: #aaa;
        min-width: 28px;
    }
    
    .clear-duration-btn {
        background: rgba(255, 100, 100, 0.2);
        border: 1px solid rgba(255, 100, 100, 0.3);
        border-radius: 3px;
        color: #faa;
        width: 24px;
        height: 24px;
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
    
    .duration-hint {
        font-size: 11px;
        color: #888;
        margin-top: 4px;
        font-style: italic;
    }
</style>