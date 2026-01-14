<script lang="ts">
    export let label: string;
    export let value: string;
    export let disabled: boolean = false;

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement;
        value = target.value;
    }
</script>

<div class="color-picker">
    <label for={label.replace(/\s+/g, '-').toLowerCase()}>
        {label}
    </label>
    <div class="input-container">
        <input
            id={label.replace(/\s+/g, '-').toLowerCase()}
            type="color"
            {value}
            {disabled}
            on:input={handleInput}
        />
        <input
            type="text"
            {value}
            {disabled}
            on:input={handleInput}
            placeholder="#000000"
            maxlength="7"
        />
    </div>
</div>

<style>
    .color-picker {
        display: flex;
        align-items: center;
        margin: 8px 0;
        gap: 12px;
    }

    label {
        font-size: 13px;
        font-weight: 400;
        color: #fff;
        min-width: 145px;
    }

    .input-container {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-shrink: 0;
    }

    input[type="color"] {
        width: 32px;
        height: 32px;
        min-width: 32px;
        min-height: 32px;
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        cursor: pointer;
        background: none;
        flex-shrink: 0;
        padding: 0;
    }

    input[type="color"]:disabled {
        cursor: not-allowed;
        opacity: 0.5;
    }

    input[type="text"] {
        width: 80px;
        min-width: 80px;
        padding: 6px 8px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        color: #fff;
        font-size: 12px;
        font-family: monospace;
        flex-shrink: 0;
    }

    input[type="text"]:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    input[type="text"]:focus {
        border-color: var(--primary-highlight);
        background: rgba(255, 255, 255, 0.1);
    }
</style>