import App from './App.svelte';
import chess_wasm from "./chess-wasm/Cargo.toml"

(async () => {
	const chess = await chess_wasm();

	new App({
		target: document.body,
		props: {
			chess: chess,
		},
	});
})();