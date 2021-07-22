<!-- Scripts -->

<script>
    // The wasm module, that needs to be awaited to be downloaded and initialized.
    export let wasm;

    import {onMount} from "svelte";
    import {fade, scale} from "svelte/transition";

    import "chessboard-element";

    import Popup from "./Popup.svelte";

    // The component that will contain the chessboard element, and it's div.
    let board;

    // The chess object, initialized with the component.
    let chess;

    // ================================ thinking duration

    // The number of seconds to make the engine think for.
    let seconds = 5;

    // The text displayed about the thinking duration.
    $: durationText = `Current thinking duration is ${seconds} seconds.`;

    // Sets the duration the engine needs to think for.
    function setDuration(secs) {
        seconds = secs;
    }

    // ================================ state

    // Game state.
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let history = [];
    let end = false;

    // Engine state.
    let thinking = false;
    let engineMove = null;
    let engineDepth = 0;

    // For reactivity.
    $: historyText = makeHistory(history);

    // Make a string of the history.
    function makeHistory(history) {
         return history.map((move, i) => {
            if (i % 2 === 0) {
                return `${i / 2 + 1}. ${move}`
            }
            return move;
        }).join(" ");
    }

    // If the game is done, makes the text corresponding to the status of the game.
    function makeEndText() {
        if (chess.isInCheck()) {
            if (chess.isWhiteToMove()) {
                return "Black won by checkmate.";
            } else {
                return "White won by checkmate.";
            }
        }
        return "The game is drawn."
    }

    // ================================ move validation

    // When a piece is dropped.
    function onDropPiece(e) {
        let from = e.detail.source;
        let to = e.detail.target;

        if (to === "offboard" || !chess.isLegal(from, to)) {
            e.detail.setAction("snapback");
        } else {
            let move = `${from}${to}`;
            if (chess.isPromotion(from, to)) {
                choosingPromotion = true;
                promoteMove = move;
            } else {
                send({kind: "play", move});
            }
        }
    }

    // When choosing a promotion.
    let choosingPromotion = false;
    let promoteMove = null;

    // Sends a promotion move.
    function promote(piece) {
        choosingPromotion = false;
        send({kind: "play", move: `${promoteMove}${piece}`});
        promoteMove = null;
    }

    // ================================ websocket

    // The websocket connected to the server.
    let ws;

    // Send an object through the websocket.
    function send(data) {
        ws.send(JSON.stringify(data));
    }

    // Upon receiving a message from the server.
    function onReceive(msg) {
        // Parse the message's data.
        const data = JSON.parse(msg.data);

        // Update the game and engine states.
        history = data.history;
        end = data.end;
        thinking = data.thinking;
        engineMove = data.engineMove;
        engineDepth = data.engineDepth;

        // Reset promotions values.
        choosingPromotion = false;
        promoteMove = null;

        // Sets the fen if it is present in the history.
        fen = data.fen;
        chess.setPosition(fen, end);
        board.setPosition(fen.split(" ")[0], true);
    }

    // ================================ on mount

    // When the board component is mounted.
    onMount(async () => {
        // Initialze the chess library and chess object.
        const lib = await wasm;
        chess = new lib.Chess();

        // Opens the websocket.
        const uri = `ws://${window.location.host}/ws/`;
        ws = new WebSocket(uri);
        ws.onmessage = onReceive;
        ws.onclose = () => alert("Warning: connection closed.");
    })
</script>

<!-- Components -->

<div id=wrapper class=centered transition:fade={{duration: 3000, delay: 2500}}>
    {#if history.length}
        <h1 id=history transition:fade>Move History:</h1>
        <p id=history-text transition:fade>{historyText}</p>
    {/if}

    <h1 id=fen>{fen}</h1>

    <button id=undo class=glow on:click={_ => send({kind: "undo"})}>Undo</button>
    <button id=flip class=glow on:click={_ => board.flip()}>Flib Board</button>
    <button id=redo class=glow on:click={_ => send({kind: "redo"})}>Redo</button>

    {#if end}
        <h1 id="thinking" transition:fade>{makeEndText()}</h1>
    {:else}
        {#if thinking}
            <h1 id=thinking transition:fade>Engine is currently thinking...</h1>
            <button id=stop class=glow on:click={_ => send({kind: "stop"})} transition:fade>Stop</button>
        {:else}
            <h1 id=thinking transition:fade>Engine is idling.</h1>
            <button id=think class=glow on:click={_ => send({kind: "think", seconds})} transition:fade>Think</button>
            <button id=think-do class=glow on:click={_ => send({kind: "thinkdo", seconds})} transition:fade>Think & Do</button>

            {#if seconds !== null}
                <h1 id=duration transition:fade>{durationText}</h1>
                <button id=change-seconds class=glow on:click={_ => seconds=null} transition:fade>Change Duration</button>
            {/if}

            {#if engineMove}
                <h1 id=engine transition:fade>Engine's preferred move: {engineMove}.<br>Furthest depth searched: {engineDepth}.</h1>
                <button id=do class=glow on:click={_ => send({kind: "do"})} transition:fade>Do Engine's Move</button>
            {:else}
                <h1 id=engine class=text transition:fade>Engine has no preferred move yet.</h1>
            {/if}
        {/if}
    {/if}
</div>

<chess-board id=board class=centered bind:this={board} on:drop={onDropPiece} transition:scale={{duration: 5000, delay: 500}} draggable-pieces></chess-board>

<Popup display={choosingPromotion}>
    <h1>Choose a promotion:</h1>
    <button class=glow on:click={_ => promote("q")}>Queen</button>
    <button class=glow on:click={_ => promote("r")}>Rook</button>
    <button class=glow on:click={_ => promote("b")}>Bishop</button>
    <button class=glow on:click={_ => promote("n")}>Knight</button>
</Popup>

<Popup display={seconds === null}>
    <h1>Choose a duration:</h1>
    <button class=glow on:click={_ => setDuration(1)}>1 Second</button>
    <button class=glow on:click={_ => setDuration(3)}>3 Seconds</button>
    <button class=glow on:click={_ => setDuration(5)}>5 Seconds</button>
    <button class=glow on:click={_ => setDuration(10)}>10 Seconds</button>
</Popup>

<!-- Styles -->

<style>
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
        grid-row: 2;
    }

    #stop {
        grid-column: 6 / 8;
        grid-row: 3;
    }

    #think {
        grid-column: 6;
        grid-row: 3;
    }

    #think-do {
        grid-column: 7;
        grid-row: 3;
    }

    #duration {
        grid-column: 6 / 8;
        grid-row: 4;
    }

    #change-seconds {
        grid-column: 6 / 8;
        grid-row: 5;
    }

    #engine {
        grid-column: 6 / 8;
        grid-row: 6;
    }

    #do {
        grid-column: 6 / 8;
        grid-row: 7;
    }

    h1 {
        margin-top: auto;
        margin-bottom: auto;
        font-size: 1em;
        font-family: sans-serif;
        position: relative;
        font-family: sans-serif;
        color: #999;
        text-align: center;
    }

    p {
        font-size: 0.9em;
        font-family: sans-serif;
        position: relative;
        color: #999;
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