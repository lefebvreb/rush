<!-- Scripts -->

<script>
    // The wasm module, that needs to be awaited to be downloaded and initialized.
    export let wasm;

    import "chessboard-element";

    import {fade, scale} from "svelte/transition";
    import {onMount} from "svelte";

    // The component that will contain the chessboard element, and it's div.
    let board;

    // The chess object, initialized with the component.
    let chess;

    // The websocket connected to the server.
    let ws;

    // Game state.
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let history = [];
    let end = false;

    // Engine state.
    let thinking = false;
    let engineMove = null;
    let engineDepth = null;

    // Upon receiving a message from the server.
    function socketMessaged(msg) {
        // Parse the message's data.
        const data = JSON.parse(msg.data);

        // Update the game and engine states.
        const properties = ["fen", "history", "end", "thinking", "engineMove", "engineDepth"];
        for (let property of properties) {
            if (property in data) this[property] = data[property];
        }

        // Sets the fen if it is present in the history.
        if ("fen" in data) {
            chess.setPosition(data.fen, end);
            board.setPosition(data.fen.split(" ")[0], true);
        }
    }

    function flipBoard() {
        board.flip();
    }

    onMount(async () => {
        // Initialze the chess library and chess object.
        const lib = await wasm;
        chess = new lib.Chess();

        // Opens the websocket.
        const uri = `ws://${window.location.host}/ws/`;
        ws = new WebSocket(uri);
        ws.onmessage = socketMessaged;
    })
</script>

<!-- Components -->

<!--
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let history = [];
    let draw = false;

    // Engine state.
    let thinking = false;
    let engineMove = null;
    let engineDepth = null;
-->

<div transition:fade={{duration: 3000, delay: 2500}}>
    {#if history.length == 0}
        <h1 id=history class=centered>~ Move History ~<br>a1a2 a1a2 a1a2 a1a2 a1a2 a1a2 a1a2 a1a2 a1a2</h1>
    {/if}

    <h1 id=fen class=centered>{fen}</h1>

    <button class="glow centered" id=flip on:click={flipBoard}>Flib Board</button>
</div>

<chess-board bind:this={board} transition:scale={{duration: 5000, delay: 500}} draggable-pieces></chess-board>

<!-- Styles -->

<style>
    .centered {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    h1 {
        font-family: sans-serif;
        font-size: 1em;
        color: #777;
        text-align: center;
    }

    #fen {
        top: calc(50% - 18em);
        white-space: nowrap;
    }

    #history {
        left: calc(50% - 32em);
        width: 32em;
    }

    #flip {
        top: calc(50% + 17em);
    }

    chess-board {
        --light-color: #c9c;
        --dark-color: #c0c;
        --highlight-color: #0f0;
        width: 32em;
        height: 32em;
        opacity: 0.75;
        border-radius: 30px;
        animation: change-color 16s infinite linear;
    }

    @keyframes change-color {
        from {filter: hue-rotate(0def);}
        to {filter: hue-rotate(359deg);}
    }
</style>