# Web client for the chess engine

To simply build the client, place yourself in `www/` and do:
```bash
npm install
npm run build
```

## Overview

This directory contains a front end for the chess engine server in `server/`. Logic is handled by a wasm module implemented in `www/src/chess-wasm/`, which provides simple bindings to the chess library. UI and communication are implemented in vanilla JavaScript managed by the [Svelte](https://svelte.dev/) framework. Sources in `www/src/`. JavaScript packages are managed by [npm](https://www.npmjs.com/).

## Build instructions

### Wasm module

To check the wasm module, run from `www/src/chess-wasm/`:
```bash
cargo check --target wasm32-unknown-unknown
```
Do not build the wasm module directly: a rollup plugin does that for us.

### Svelte app

You will need npm and some dependencies to build the client, to get them, run the following command in the `www/` directory:
```bash
npm install
```

To run in dev mode, with file watch and live reload, do in `www/`:
```bash
npm run dev
```

To build the app in production mode, inluding the wasm module, run in `www`:
```bash
npm run build
```