import App from './App.svelte';
import chess_wasm from "./chess-wasm/Cargo.toml"

const wasm = chess_wasm();

const app = new App({
    target: document.body,
    props: {
        wasm,
    },
});

export default app;