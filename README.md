<img align="left" alt="" src="logo.svg" height="150"/>

# Rush: Rust Chess Engine

## Overview

This project is:
+ A fast chess game featuring bitboards, pext/pdep lookup and state of the art move generation.
+ A parallel tree search AI using the lazy-SMP algorithm and a shared hashtable.
+ A rust web server backend.
+ A lightweight web front-end, with a wasm version of the chess library.

This chess engine was made with the goal of beating a friend of mine in a game of chess. Said friend has an elo of around 2000.

Since pext/pdep lookup requires the bmi2 instruction set, and my friend didn't have it, I decided to build a server and a web client as well so he could play with my AI remotely.

Don't worry, you don't need to possess a cpu with the pext instruction to build the project, a (slower) software replacement is provided.

## Build & use instructions

For the web client's build instructions, checkout `www/README.md`.

All below commands assume that your shell is located at the root of the project.

### `chess` crate

The `chess` crate is a library, it is not useful to build it in itself. However, you can run the tests by doing:
```bash
cargo test
```

It also provides an executable binary, named perft. It is used to debug move generation and benchmarking. You can get information about it's usage and precise behaviour by running it with no arguments:
```bash
cargo run --bin perft --release
```

### `engine` crate

The `engine` crate is a library as well, it is used by the server.

It also provides a basic cli binary to play the engine via the terminal. Run it with no arguments to get the usage of this particular tool:
```bash
cargo run --bin engine-cli --release
```

### `server` crate

The `server` crate is a binary executable that distributes the client to the web and hosts an AI, for remote play. Before running it, make sure the client was build first, then do:
```bash
cargo build --bin server --release
./target/release/server <port>
```
Where port is the port the server will bind to. The link to the server will be printed to stdin.

## TODO list

### Engine:
+ Opening book integration.
+ Better evaluation.
+ Refactor movepick for quiescient search.