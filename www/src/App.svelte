<!-- Scripts -->

<script>
    export let wasm;

	import {onMount} from "svelte";
	
	onMount(async () => {
        const lib = await wasm;

        console.log("Chess initialized");

        const chess = new lib.Chess("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        console.log(chess.getFen());

        let legals = chess.getLegalMoves();
        console.log(legals);

        chess.setFen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2");
        console.log(chess.getFen());

        legals = chess.getLegalMoves();
        console.log(legals);

        // TODO: everything else.
	});
</script>

<!-- Components -->

<!-- Styles -->

<style>
    :global(body) {
        background: #000000;
        padding: 0;
        display: grid;
        place-items: center;
    }
</style>
