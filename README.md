<img align="left" alt="" src="logo.svg" height="150"/>

# Rush: Rust Chess Engine

## Overview

This project is:
+ A fast chess game featuring bitboards, `pext`/`pdep` lookup and performant move generation.
+ A parallel tree search AI using the lazy-SMP algorithm and a shared hashtable.
+ A rust web server backend.
+ A lightweight web front-end, with a wasm version of the chess library.

This chess engine was made with the goal of beating a friend of mine in a game of chess. Said friend has an elo of around 2000.

Since `pext`/`pdep` lookup requires the `bmi2` instruction set, and my friend didn't have it, I decided to build an http server and a web client so he could play with my AI remotely.

Don't worry, you don't need to have a cpu with the `bmi2` instruction set to build the project, a (slower) software replacement is provided.

## Build & use instructions

All below commands assume that your shell is located at the root of the project.

### `chess` crate

The `chess` crate is a library, it is not useful to build it in itself. However, you can run the tests by doing:
```bash
cargo test
```

It also provides an executable binary, named perft. It is used to debug move generation and for benchmarking. You can get information about it's usage and precise behaviour by running it with the `--help` option:
```bash
cargo build --bin perft --release
./target/release/perft --help
```

### `engine` crate

The `engine` crate is a library as well, it is used by the server.

It also provides a basic cli binary to play the engine via the terminal. Run it with no arguments to get the usage of this particular tool:
```bash
cargo build --bin engine-cli --release
./target/release/engine-cli --help
```

### Web client and `www` directory

This directory contains a front end for the chess engine server in `server/`. Logic is handled by a wasm module implemented in `www/src/chess-wasm/`, which provides simple bindings to the chess library. UI and websocket communication are implemented in vanilla JavaScript managed by the [Svelte](https://svelte.dev/) framework. Sources are in `www/src/`. JavaScript packages are managed by [npm](https://www.npmjs.com/).

The client is automatically built when building the server, thanks to a build script fired when the client's sources change. You however need to install npm and perform the following command once, after downloading the repo and before building the server:
```bash
npm run install
```
This will resolve all JavaScript dependencies.

### `server` crate

The `server` crate is a binary executable that distributes the client to the web and hosts an AI, for remote play. To build and get help on how to invoke it, do:
```bash
cargo build --bin server --release
./target/release/server --help
```
Where port is the port the server will bind to. The link to the server will be printed to stdin.

Yes, it takes forever to compile.

## TODO list

### Chess
+ Make book reading lazy (later).

### Engine
+ Incremental evaluation.
+ Better evaluation.
+ Profiling.