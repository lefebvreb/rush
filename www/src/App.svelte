<!-- Meta -->

<svelte:head>
    <title>Chess Engine Client</title>
    <meta charset="UTF-8">
    <meta name="description" content="Chess Engine Client">
    <meta name="author" content="Benjamin Lefebvre">
</svelte:head>

<!-- Scripts -->

<script>
    export let wasm;

	import {onMount} from "svelte";
	
	onMount(async () => {
        let lib = await wasm;
        // Initialize the chess library.
        lib.start();

        console.log("Chess initialized");

        const chess = new lib.Chess("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        console.log(chess.fen);

        const legals = chess.getLegalMoves();
        console.log(legals);

        let total = 0;
        for (let move of legals) {
            chess.play(move);
            const count = chess.getLegalMoves().size;
            console.log(`${move} ${count}`);
            total += count;
            chess.back();
        }
        console.log(`${total}`);

        chess.setFen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2");
        console.log(chess.fen);

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
