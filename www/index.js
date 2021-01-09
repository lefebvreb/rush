// ==================================== TYPES

// A type representing a square
class Square {
    // Construct a new square and compute it's screen coordinates
    constructor(x, y) {
        this.x = x
        if (role == ROLE.white)
            this.y = 7-y
        else
            this.y = y
        this.screenX = x*128
        this.screenY = y*128
    }

    // Gives the string representation of this square
    toString() {
        return "abcdefgh"[this.x] + "12345678"[this.y]
    }

    // Test for equality against another square
    equals(other) {
        return this.x == other.x && this.y == other.y
    }
}

class Board {
    // Construct a new board
    constructor() {
        this.pieces = new Array(64)

        this.pieces[8] = PIECE.bPawn
        this.pieces[48] = PIECE.wPawn
    }

    // Get the piece at the given square
    getPieceAt(sq) {
        return this.pieces[sq.x + 8*sq.y]
    }

    // Set the piece at the given square
    setPieceAt(sq, piece) {
        this.pieces[sq.x + 8*sq.y] = piece
    }
}

class Piece {
    // Construct a new Piece, should not be called outside of the PIECE record
    constructor(char, role, src) {
        this.char = char
        this.role = role
        this.src = [src[0] * 128, src[1] * 128]
    }

    // Get the piece corresponding to the given string
    static fromChar(char) {
        switch (char) {
            case "P": return PIECE.wPawn;
            case "R": return PIECE.wRook;
            case "N": return PIECE.wKnight;
            case "B": return PIECE.wBishop;
            case "Q": return PIECE.wQueen;
            case "K": return PIECE.wQueen;
            case "p": return PIECE.bPawn;
            case "r": return PIECE.bRook;
            case "n": return PIECE.bKnight;
            case "b": return PIECE.bBishop;
            case "q": return PIECE.bQueen;
            case "k": return PIECE.bQueen;
        }
    }
}

// ==================================== CONSTANTS

// The colors used in the game
const COLOR = {
    beige:     "#e5d792",
    green:     "#578940",
    select:    "#f7eb4c",
    lastMove:  "#82d1d0",
    validMove: "#df89e0",
}

// The roles a client can have
const ROLE = {
    white: 0,
    black: 1,
    guest: 2,
}

// All pieces, their representing chars and position in atlas
const PIECE = {
    wPawn:   new Piece("P", ROLE.white, [0, 0]),
    wRook:   new Piece("R", ROLE.white, [1, 0]),
    wKnight: new Piece("N", ROLE.white, [2, 0]),
    wBishop: new Piece("B", ROLE.white, [3, 0]),
    wQueen:  new Piece("Q", ROLE.white, [4, 0]),
    wKing:   new Piece("K", ROLE.white, [5, 0]),
    bPawn:   new Piece("p", ROLE.black, [0, 1]),
    bRook:   new Piece("r", ROLE.black, [1, 1]),
    bKnight: new Piece("k", ROLE.black, [2, 1]),
    bBishop: new Piece("b", ROLE.black, [3, 1]),
    bQueen:  new Piece("q", ROLE.black, [4, 1]),
    bKing:   new Piece("k", ROLE.black, [5, 1]),
}

// ==================================== GLOBAL VARIABLES

// The websocket connected to the server
var socket
// The canvas context2D
var ctx
// The atlas image
var atlas

// The role of the client
var role = ROLE.white

// The selected squares
var select //= {x: 0, y: 0}
// The lastMove played
var lastMove //= {from: {x: 5, y: 6}, to: {x: 5, y: 7}}
// The list of valid moves
var validSquares = [new Square(0, 7), new Square(0, 0)]//[{x: 0, y: 7}, {x: 0, y: 0}] //= [{x: 1, y: 1}, {x:1, y: 2}]

// The board of the game
var board
// Whether or not the client is playing
var playing = true
// Whether or not the client is promoting
var promote

// ==================================== BOARD UTIL

function tryMove(from, to) {
    if (validSquares)
        for (move of validSquares)
            if (to.equals(move)) {
                let piece = board.getPieceAt(from)
                board.setPieceAt(from, null)

                let x = to.x
                let y = to.y
                
                if (piece == PIECE.wPawn && to.y == 7)
                    promote = {role: ROLE.white, sq: to}
                else if (piece == PIECE.bPawn && to.y == 0)
                    promote = {role: ROLE.black, sq: to}
                else {
                    board.setPieceAt(to, piece)
                    socket.send("move " + squareToString(from) + squareToString(to))
                }

                return true 
            }

    return false 
}

function tryPromote(i) {
    console.log(i)
}

// ==================================== DRAWING UTIL

// Sets the alpha channel of the canvas 2d context
function setAlpha(alpha) {
    ctx.globalAlpha = alpha
}

// Fills the square x, y with the given color and alpha
function fillSquare(sq, color) {
    ctx.fillStyle = color
    ctx.fillRect(sq.screenX, sq.screenY, 128, 128)
}

// Draws the character c to x, y, with the specified color
function drawCharacter(c, sq, dx, dy, color) {
    ctx.fillStyle = color
    ctx.fillText(c, sq.screenX+dx, sq.screenY+dy)
}

// Draws the given piece to x, y
function drawImage(src, x, y) {
    ctx.drawImage(atlas, src[0], src[1], 128, 128, x, y, 128, 128)
}

// ==================================== DRAWING

// Draws the square sq, positionned at x, y on the canvas
function drawSquare(sq) {
    let color1, color2
    if ((sq.x + sq.y) % 2 == 0) {
        color1 = COLOR.green
        color2 = COLOR.beige
    } else {
        color1 = COLOR.beige
        color2 = COLOR.green
    }

    setAlpha(1)
    fillSquare(sq, color1)
    if (sq.screenX == 0)
        drawCharacter("12345678"[sq.y], sq, 5, 40, color2)
    if (sq.screenY == 896)
        drawCharacter("abcdefgh"[sq.y], sq, 95, 120, color2)
}

function drawPiece(sq) {
    let piece = board.getPieceAt(sq)
    if (piece)
        drawImage(piece.src, sq.screenX, sq.screenY)
}

function drawPromote() {
    let pieces
    if (promote.role == ROLE.white)
        pieces = [PIECE.wQueen, PIECE.wRook, PIECE.wBishop, PIECE.wKnight]
    else
        pieces = [PIECE.bQueen, PIECE.bRook, PIECE.bBishop, PIECE.bKnight]

    function draw(i, dx, dy) {
        let src = pieces[i].src
        ctx.drawImage(atlas, src[0], src[1], 128, 128, promote.sq.screenX+dx, dy, 64, 64)
    }

    draw(0, 0, 0)
    draw(1, 64, 0)
    draw(2, 0, 64)
    draw(3, 64, 64)
}

// Draw the page
function draw() {
    setAlpha(1)
    for (let y = 0; y < 8; y++)
        for (let x = 0; x < 8; x++)
            drawSquare(new Square(x, y))

    setAlpha(0.5)
    if (select && !promote)
        fillSquare(select, COLOR.select)
    if (lastMove) {
        fillSquare(lastMove.from, COLOR.lastMove)
        fillSquare(lastMove.to, COLOR.lastMove)
    }
    if (validSquares && !promote)
        for (sq of validSquares)
            fillSquare(sq, COLOR.validMove)

    setAlpha(1)
    for (let y = 0; y < 8; y++)
        for (let x = 0; x < 8; x++)
            drawPiece(new Square(x, y))

    if (promote)
        drawPromote()
}

// ==================================== WINDOW EVENTS HANDLING

// When window is ready
window.onload = function() {
    canvas = document.getElementById("canvas")
    ctx = canvas.getContext("2d")
    ctx.font = "32px Arial"

    board = new Board()
    
    atlas = document.createElement("img")
    atlas.src = "atlas.png"
    atlas.onload = function() {
        draw()
        canvas.addEventListener("click", onCanvasClicked)
    }
    
    initSocket()
}

// Gets the coordinates of the click event and transforms them into board coordinates
function onCanvasClicked(e) {
    if (!playing)
        return

    let rect = e.target.getBoundingClientRect()
    let x = e.offsetX / (rect.right - rect.left)
    let y = e.offsetY / (rect.bottom - rect.top)

    let sq = new Square(Math.floor(x * 8), Math.floor(y * 8))

    console.log(sq.toString())

    function trySelect() {
        let piece = board.getPieceAt(sq)
        if (piece && piece.role == role)
            select = sq
        else
            select = null
    }

    if (promote && promote.sq.equals(sq)) {
        let i = Math.floor(x * 16) % 2
        let j = Math.floor(y * 16) % 2
        tryPromote(i + 2 * j)
    } else if (select && tryMove(select, sq))
        select = null
    else
        trySelect()

    draw()
}

// ==================================== SOCKET EVENTS HANDLING

// Initializes the socket
function initSocket() {
    let uri = "ws://" + window.location.host + "/ws/"
    socket = new WebSocket(uri)

    socket.onopen = socketOpened
    socket.onmessage = socketMessaged
}

// Fired when the socket is opened for the first time
function socketOpened() {
    console.log("Socket opened")
    socket.send("Hello from web client")
}

// Fired when the socket just received a message
function socketMessaged(msg) {
    console.log("Got a message: " + msg.data)
}
