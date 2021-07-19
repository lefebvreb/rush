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

    // Upon receiving a message from the server.
    function socketMessaged(msg) {
        console.log(msg.data);
    }

    onMount(async () => {
        console.log(board);

        // Iinitialze the chess library and chess object.
        const lib = await wasm;
        chess = new lib.Chess();

        // Opens the websocket.
        const uri = `ws://${window.location.host}/ws/`;
        ws = new WebSocket(uri);
        ws.onmessage = socketMessaged;
    })
</script>

<!-- Components -->

<chess-board bind:this={board}
    draggable-pieces
    position="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
></chess-board>

<!-- Styles -->

<style>
    chess-board {
        --light-color: #c99;
        --dark-color: #c00;
        width: 40em;
        height: 40em;
        opacity: 0.9;
        animation: change-color 20s infinite;
    }

    @keyframes change-color {
        from {filter: hue-rotate(0def);}
        to {filter: hue-rotate(359deg);}
    }
</style>