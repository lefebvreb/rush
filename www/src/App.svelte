<!-- Meta -->

<svelte:head>
    <title>Chess Engine Client</title>
    <meta charset="UTF-8">
    <meta name="description" content="Chess Engine Client">
    <meta name="author" content="Benjamin Lefebvre">
</svelte:head>

<!-- Scripts -->

<script>
    export let chess;

	import {onMount} from "svelte";
	
	onMount(() => {
        // Initialize the chess library.
        chess.start();

        console.log("ici");
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
