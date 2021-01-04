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
    wPawn:   {char: "P", role: ROLE.white, src: [0, 0]},
    wRook:   {char: "R", role: ROLE.white, src: [128, 0]},
    wKnight: {char: "N", role: ROLE.white, src: [256, 0]},
    wBishop: {char: "B", role: ROLE.white, src: [384, 0]},
    wQueen:  {char: "Q", role: ROLE.white, src: [512, 0]},
    wKing:   {char: "K", role: ROLE.white, src: [640, 0]},
    bPawn:   {char: "p", role: ROLE.black, src: [0, 128]},
    bRook:   {char: "r", role: ROLE.black, src: [128, 128]},
    bKnight: {char: "n", role: ROLE.black, src: [256, 128]},
    bBishop: {char: "b", role: ROLE.black, src: [384, 128]},
    bQueen:  {char: "q", role: ROLE.black, src: [512, 128]},
    bKing:   {char: "k", role: ROLE.black, src: [640, 128]},
}

// ==================================== TYPES

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
    constructor() {
        this.pieces = new Array(64)
    }

    pieceAt(x, y) {
        return this.pieces[x + 8*y]
    }
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
var validMoves //= [{x: 0, y: 7}, {x: 0, y: 0}] //= [{x: 1, y: 1}, {x:1, y: 2}]

// The board of the game
var board
// Whether or not the client is playing
var playing = true
// Whether or not the client is promoting
var promote

// ==================================== BOARD UTIL

// Initializes the board
function initBoard() {
    board = new Array(8)

    for (let x = 0; x < 8; x++)
        board[x] = new Array(8)

    board[0][6] = PIECE.wPawn
    board[0][1] = PIECE.bPawn
}

function tryMove(from, to) {
    if (validMoves)
        for (move of validMoves)
            if (to.equals(move)) {
                let piece = board[from.x][from.y]
                board[from.x][from.y] = null

                let x = to.x
                let y = to.y
                
                if (piece == PIECE.wPawn && to.y == 7)
                    promote = {role: ROLE.white, sq: {x, y}, x: x*128}
                else if (piece == PIECE.bPawn && to.y == 0)
                    promote = {role: ROLE.black, sq: {x, y}, x: x*128}
                else {
                    board[to.x][to.y] = piece
                    socket.send("move " + squareToString(from) + squareToString(to))
                }

                break
            }
}

function tryPromote(i) {

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
    let piece = board[sq.x][sq.y]
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
        ctx.drawImage(atlas, src[0], src[1], 128, 128, promote.screenX+dx, dy, 64, 64)
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
    if (validMoves && !promote) {
        for (let i = 0; i < validMoves.length; i++)
        fillSquare(validMoves[i], COLOR.validMove)
    }

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

    initBoard()
    
    atlas = document.createElement("img")
    atlas.src = "atlas.png"
    atlas.onload = function() {
        draw()
        canvas.addEventListener("click", onCanvasClicked)
    }
}

// Gets the coordinates of the click event and transforms them into board coordinates
function onCanvasClicked(e) {
    if (!playing)
        return

    let rect = e.target.getBoundingClientRect();
    let x = Math.floor(e.offsetX * 8 / (rect.right - rect.left))
    let y = Math.floor(e.offsetY * 8 / (rect.bottom - rect.top))

    let sq = new Square(x, y)

    console.log(sq.toString())

    function trySelect() {
        let piece = board[sq.x][sq.y]
        if (piece && piece.role == role) {
            select = sq
            return false
        }
        return true
    }

    if (promote && compareSquares(promote.sq, sq)) {
        //tryPromote(x)
    } else if (select) {
        if (!trySelect())
            tryMove(select, sq)
        else
            select = null
    } else 
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

// ==================================== INIT

initSocket()
