<!-- Scripts -->

<script>
    // The wasm module, that needs to be awaited to be downloaded and initialized.
    export let wasm;

    import "chessboard-element";
    import {onMount} from "svelte";

    // The component that will contain the chessboard element, and it's div.
    let board;

    // The chess object, initialized with the component.
    let chess;

    // The websocket connected to the server.
    let ws;

    // Game state.
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let history = "";
    let draw = false;

    // Engine state.
    let thinking = false;
    let engineMove = null;
    let engineDepth = null;

    // Upon receiving a message from the server.
    function socketMessaged(msg) {
        // Parse the message's data.
        const data = JSON.parse(msg.data);

        // Update the game and engine states.
        const properties = ["fen", "history", "draw", "thinking", "engineMove", "engineDepth"];
        for (let property of properties) {
            if (property in data) this[property] = data[property];
        }

        // Sets the fen if it is present in the history.
        if ("fen" in data) {
            chess.setPosition(data.fen, draw);
            board.setPosition(data.fen.split(" ")[0], true);
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
    })
</script>

<!-- Components -->

<chess-board bind:this={board} draggable-pieces></chess-board>

<!-- Styles -->

<style>
    chess-board {
        --light-color: #c99;
        --dark-color: #c00;
        width: 32em;
        height: 32em;
        opacity: 0.75;
        border-radius: 30px;
        animation: change-color 20s infinite;
    }

    @keyframes change-color {
        from {filter: hue-rotate(0def);}
        to {filter: hue-rotate(359deg);}
    }
</style>