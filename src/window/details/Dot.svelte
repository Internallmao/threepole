<script lang="ts">
    export let completed: boolean;
    export let completedColor: string = "#3e3";
    export let incompleteColor: string = "#e33";
</script>

<span
    class="dot"
    style="background-color: {completed ? completedColor : incompleteColor}"
/>

<style>
    .dot {
        display: inline-block;
        vertical-align: middle;
        width: 8px;
        height: 8px;
        border-radius: 50%;
        margin-right: 8px;
    }
</style>
