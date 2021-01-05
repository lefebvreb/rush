# Communication protocol

`<fen> ::= `

`<move> ::= [a-h][1-8]{2}[qrbn]?`

## Client's commands

`legals`: Request a list of all legal moves
`move <move>`: Try to play the given move
`play`: Ask to play and be given a new role

## Server's commands

`legals <move>*`: Give the list of all legal moves
`role [wbs]`: Give a new role
`update <fen> [wb]`: Update clients about the state of the game