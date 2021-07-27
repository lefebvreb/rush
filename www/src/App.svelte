<!-- Scripts -->

<script>
    // The wasm module, that needs to be awaited to be downloaded and initialized.
    export let wasm;

    import {fade} from "svelte/transition";

    import Board from "./Board.svelte";
    import Logo from "./Logo.svelte";

    // A flag indicating if the client has joined the game yet or not.
    let joined = false;
</script>

<!-- Components -->

<Logo></Logo>

<h1>Rush Chess Engine ~ Copyleft🄯 2021 Benjamin Lefebvre ~ GPLv3 Licensed ~ <a href="https://github.com/L-Benjamin/rush">Repository</a></h1>

{#if joined}
    <Board wasm={wasm}></Board>
{:else}
    <button class=glow on:click={_ => joined=true} transition:fade>Join</button>
{/if}

<!-- Styles -->

<style>
    h1 {
        font-family: sans-serif;
        font-size: 0.9em;
        color: #fff;
        position: absolute;
        left: 2em;
        bottom: 2em;
    }

    /* Global styles */

    :global(body) {
        overflow: hidden;
        background: #000;
        padding: 0;
        height: 100vh;
        display: grid;
        place-content: center;
    }

    :global(.centered), :global(.glow) {
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    :global(.glow) {
        font-size: 1em;
        font-family: sans-serif;
        position: relative;
        width: 9em;
        height: 4em;
        border: none;
        outline: none;
        color: #fff;
        background: #111;
        cursor: pointer;
        position: relative;
        z-index: 0;
        border-radius: 1em;
    }

    :global(.glow:before) {
        content: '';
        background: linear-gradient(45deg, #ff0000, #ff7300, #fffb00, #48ff00, #00ffd5, #002bff, #7a00ff, #ff00c8, #ff0000);
        position: absolute;
        top: -0.5em;
        left: -0.5em;
        background-size: 400%;
        z-index: -1;
        filter: blur(1em);
        width: calc(100% + 1em);
        height: calc(100% + 1em);
        animation: glowing 10s infinite linear alternate;
        opacity: 0;
        transition: opacity .3s ease-in-out;
        border-radius: 0.5em;
    }

    :global(.glow:active:after) {
        background: transparent;
    }

    :global(.glow:hover:before) {
        opacity: 1;
    }

    :global(.glow:after) {
        z-index: -1;
        content: '';
        position: absolute;
        width: 100%;
        height: 100%;
        background: #111;
        left: 0;
        top: 0;
        border-radius: 0.5em;
    }

    @keyframes glowing {
        from { background-position: 0 0; }
        to { background-position: 400% 0; }
    }
</style>
