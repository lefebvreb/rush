<!-- Scripts -->

<script>
    export let display;
</script>

<!-- Components -->

{#if display}
    <div id=focus-taker class=centered>
        <div id=wrapper class=centered>
            <slot></slot>
        </div>
    </div>
{/if}

<!-- Style -->

<style>
    #focus-taker {
        position: absolute;
        width: 100%;
        height: 100%;
    }
    
    #wrapper {
        position: absolute;
        display: grid;
        grid-auto-rows: 5em;
        grid-auto-columns: 10em;
        background: #000;
        border-radius: 1em;
        padding: 1em;
        place-content: center;
        text-align: center;
    }
</style>