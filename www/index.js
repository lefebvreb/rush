// ==================================== CONSTANTS

// The colors used in the game
const COLOR = {
    beige: "#e0ca5e",
    green: "#578940",
    select: "#f7eb4c",
    lastMove: "#82d1d0",
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
    wPawn:   {char: "P", src: [0, 0]},
    wRook:   {char: "R", src: [128, 0]},
    wKnight: {char: "N", src: [256, 0]},
    wBishop: {char: "B", src: [384, 0]},
    wQueen:  {char: "Q", src: [512, 0]},
    wKing:   {char: "K", src: [640, 0]},
    bPawn:   {char: "p", src: [0, 128]},
    bRook:   {char: "r", src: [128, 128]},
    bKnight: {char: "n", src: [256, 128]},
    bBishop: {char: "b", src: [384, 128]},
    bQueen:  {char: "q", src: [512, 128]},
    bKing:   {char: "k", src: [640, 128]},
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
// The board of the game
var board

// The lastMove played
var lastMove
// The selected squares
var select
// The list of valid moves
var validMoves

// ==================================== BOARD UTIL

// Initializes the board
function initBoard() {
    board = new Array(8)
    for (let x = 0; x < 8; x++)
        board[x] = new Array(8)
}

// ==================================== PARSE UTIL

// Converts a square to a string
function squareToString(sq) {
    return "abcdefgh"[sq.x] + "12345678"[sq.y]
}

// ==================================== DRAWING UTIL

// Sets the alpha channel of the canvas 2d context
function setAlpha(alpha) {
    ctx.globalAlpha = alpha
}

// Fills the square x, y with the given color and alpha
function fillSquare(x, y, color) {
    ctx.fillStyle = color
    ctx.fillRect(x, y, 128, 128)
}

// Draws the character c to x, y, with the specified color
function drawCharacter(c, x, y, color) {
    ctx.fillStyle = color
    ctx.fillText(c, x , y)
}

// Draws the given piece to x, y
function drawPiece(piece, x, y) {
    ctx.drawImage(atlas, piece.src[0], piece.src[1], 128, 128, x, y, 128, 128)
}

// ==================================== DRAWING

// Draws the square sq, positionned at x, y on the canvas
function drawSquare(sq, x, y) {
    let color1, color2
    if ((sq.x + sq.y) % 2 == 0) {
        color1 = COLOR.green
        color2 = COLOR.beige
    } else {
        color1 = COLOR.beige
        color2 = COLOR.green
    }

    if (color1 == COLOR.green)
        fillSquare(x, y, color1, 1)
    if (x == 0)
        drawCharacter("12345678"[sq.y], x+5, y+40, color2)
    if (y == 896)
        drawCharacter("abcdefgh"[sq.x], x+95, y+120, color2)

    let piece = board[sq.x][sq.y]
    if (piece)
        drawPiece(piece, x, y)
}

// Draw the page
function draw() {
    for (let y = 0; y < 8; y++)
        for (let x = 0; x < 8; x++) {
            if (role == ROLE.black)
                drawSquare({x, y}, x*128, y*128)
            else
                drawSquare({x, y}, x*128, (7-y)*128)
        }

    if (select) {
        setAlpha(0.5)
        fillSquare(select.x*128, y, COLOR.select)
    }

    if (lastMove) {
        setAlpha(0.3)
        fillSquare(x, y, COLOR.lastMove)
    }
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
    atlas.onload = draw

    canvas.addEventListener("click", onclick)
}

// Gets the coordinates of the click event and transforms them into board coordinates
function onclick(e) {
    let rect = e.target.getBoundingClientRect();
    let x = Math.floor(e.offsetX * 8 / (rect.right - rect.left))
    let y = Math.floor(e.offsetY * 8 / (rect.bottom - rect.top))

    if (role == ROLE.white)
        y = 7-y

    let sq = {x, y}

    console.log(squareToString(sq))
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
    socket.send("Hello from javascript")
}

// Fired when the socket just received a message
function socketMessaged(msg) {
    console.log("Got a message: " + msg.data)
}

// ==================================== INIT

initBoard()

initSocket()