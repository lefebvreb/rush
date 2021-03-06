# chess-engine

This project is:
+ A fast chess game featuring bitboards, pext lookup and lazy move generation.
+ A parallel tree search AI using the lazy-SMP algorithm and a shared hashtable.
+ A web server backend in rust using the [actix-web](https://actix.rs/) crate.
+ A lightweight, pure javascript front-end.

This chess engine was made with the goal of beating a friend of mine in a game of chess. Said friend has an elo of around 2000.

Since pext lookup requires the pext asm instruction (introduced in the bmi2 instruction set), and my friend didn't have it, I decided to build a server and a web client as well so he could play with my AI.

Don't worry, you don't need to possess a cpu with the pext instruction to build the project, a (slower) software replacement is provided.

## Build instructions

### Build the game, AI and server

Simply place yourself in the root of the project and do:
```
$ cargo build --bin server --release
```

### Build the web client

To install the dependencies, place yourself in `www/` and do:
```
$ npm install
```

To build the client in release mode, do:
```
$ npm run clean
$ npm run build
```

## Run instructions

To run the server, place yourself in the root directory of the project and do:
```
$ cargo run --bin server --release -- [ipv4 address]
```
Where `[ipv4 address]` is the ip address the server will be bound to, if none is provided, `127.0.0.1:8080` (localhost port 8080) is assumed.

To run the automated tests, do:
```
$ cargo test
```

We also provide another executable, to be used for debugging move generation along [perftree](https://github.com/agausmann/perftree). It can be built and run with:
```
$ cargo run --bin perft
```