# chess-engine

This project is (or will soon be):
+ A fast chess game featuring bitboards, pext lookup state of the art move generation.
+ A parallel tree search AI using the lazy-SMP algorithm and a shared hashtable.
+ A web server backend.
+ A lightweight web front-end.

This chess engine was made with the goal of beating a friend of mine in a game of chess. Said friend has an elo of around 2000.

Since pext lookup requires the pext asm instruction (introduced in the bmi2 instruction set), and my friend didn't have it, I decided to build a server and a web client as well so he could play with my AI.

Don't worry, you don't need to possess a cpu with the pext instruction to build the project, a (slower) software replacement is provided.

## Build instructions

Coming soon.