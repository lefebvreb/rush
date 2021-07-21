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

    // Send an object through the websocket.
    function send(data) {
        ws.send(JSON.stringify(data));
    }

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
        if ("history" in data) history = data.history;
        if ("end" in data) end = data.end;
        if ("thinking" in data) thinking = data.thinking;
        if ("engineMove" in data) engineMove = data.engineMove;
        if ("engineDepth" in data) engineDepth = data.engineDepth;

        // Sets the fen if it is present in the history.
        if ("fen" in data) {
            fen = data.fen;
            chess.setPosition(fen, end);
            board.setPosition(fen.split(" ")[0], true);
        }
    }

    // For reactivity.
    $: historyText = makeHistory(history);

    // Make a string of the history.
    function makeHistory(history) {
        let res = "";

        for (let i = 0; i < history.length/2; i++) {
            let x = i*2;
            res += `${i+1}. ${history[x]} `;
            let y = x+1;
            if (y < history.length) {
                res += `${history[y]} `;
            }
        }

        return res;
    }

    // If the game is done, makes the text corresponding to the status of the game.
    function makeEndText() {
        if (chess.isInCheck()) {
            if (chess.isWhiteToMove()) {
                return "White won by checkmate";
            } else {
                return "Black won by checkmate";
            }
        }
        return "The game is drawn."
    }

    // When a piece is dropped.
    function dropPiece(e) {
        let from = e.detail.source;
        let to = e.detail.target;

        if (to === "offboard" || !chess.isLegal(from, to)) {
            e.detail.setAction("snapback");
        } else {
            if (chess.isPromotion(from, to)) {
                // TODO: under-promotes
                send({kind: "play", move: `${from}${to}q`});
            } else {
                send({kind: "play", move: `${from}${to}`});
            }
        }
    }

    onMount(async () => {
        // Initialze the chess library and chess object.
        const lib = await wasm;
        chess = new lib.Chess();

        // Opens the websocket.
        const uri = `ws://${window.location.host}/ws/`;
        ws = new WebSocket(uri);
        ws.onmessage = socketMessaged;
        ws.onclose = () => alert("Warning: connection closed.");
    })
</script>

<!-- Components -->

<div id=wrapper class=centered transition:fade={{duration: 3000, delay: 2500}}>
    {#if history.length}
        <h1 id=history class=text>Move History:</h1>
        <p id=history-text>{historyText}</p>
    {/if}

    <h1 id=fen class=text>{fen}</h1>

    <button id=undo class="glow centered" on:click={_ => send({kind: "undo"})}>Undo</button>
    <button id=flip class="glow centered" on:click={_ => board.flip()}>Flib Board</button>
    <button id=redo class="glow centered" on:click={_ => send({kind: "redo"})}>Redo</button>

    {#if end}
        <h1 id="thinking" class=text>{makeEndText()}</h1>
    {:else}
        {#if thinking}
            <h1 id=thinking class=text>Engine is currently thinking...</h1>
            <button id=think class="glow centered" on:click={_ => send({kind: "stop"})}>Stop</button>
        {:else}
            <h1 id=thinking class=text>Engine is idling.</h1>
            <range id=seconds></range>
            <button id=think class="glow centered" on:click={_ => send({kind: "think", seconds: 5})}>Think</button>

            {#if engineMove}
                <h1 id=engine class=text>Engine preferred move: {engineMove}.<br>Furthest depth searched: {engineDepth}.</h1>
                <button id=do class="glow centered" on:click={_ => send({kind: "do"})}>Do Engine's Move</button>
            {:else}
                <h1 id=engine class=text>Engine has no preferred move yet.</h1>
            {/if}
        {/if}
    {/if}
</div>

<chess-board id=board class=centered bind:this={board} on:drop={dropPiece} transition:scale={{duration: 5000, delay: 500}} draggable-pieces></chess-board>

<!-- Styles -->

<style>
    .centered {
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    #wrapper {
        position: absolute;
        display: grid;
        grid-auto-rows: 5em;
        grid-auto-columns: 10em;
        place-content: center;
        text-align: center;
    }

    #fen {
        grid-column: 2 / 7;
        grid-row: 1;
        white-space: nowrap;
    }

    #history {
        grid-column: 1 / 3;
        grid-row: 3;
    }

    #history-text {
        grid-column: 1 / 3;
        grid-row: 4 / 8;
    }

    #undo {
        grid-column: 3;
        grid-row: 8;
    }

    #flip {
        grid-column: 4;
        grid-row: 8;
    }

    #redo {
        grid-column: 5;
        grid-row: 8;
    }

    #thinking {
        grid-column: 6 / 8;
        grid-row: 3;
    }

    #think {
        grid-column: 6 / 8;
        grid-row: 4;
    }

    #engine {
        grid-column: 6 / 8;
        grid-row: 5;
    }

    #do {
        grid-column: 6 / 8;
        grid-row: 6;
    }

    #seconds {
        position: absolute;
        left: 50%;
        top: 50%;
        width: 200px;
        margin-top: 10px;
        transform: translate(-50%, -50%);
    }

    h1 {
        margin-top: auto;
        margin-bottom: auto;
        font-size: 1em;
        font-family: sans-serif;
        position: relative;
        font-family: sans-serif;
        color: #777;
        text-align: center;
    }

    p {
        font-size: 0.9em;
        font-family: sans-serif;
        position: relative;
        color: #777;
        text-align: justify;
        text-justify: inter-word;
        text-align-last: left;
    }

    chess-board {
        --light-color: #c9c;
        --dark-color: #c0c;
        --highlight-color: #0f0;
        width: 30em;
        height: 30em;
        position: absolute;
        opacity: 0.75;
        animation: change-color 16s infinite linear;
    }

    @keyframes change-color {
        from {filter: hue-rotate(0def);}
        to {filter: hue-rotate(359deg);}
    }
</style>