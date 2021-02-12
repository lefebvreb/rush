import {
    ChessBoardElement as _, 
    renderWikipediaSVGPiece as renderPiece,
} from "../node_modules/chessboard-element/index.js"

// ==================================== CONSTANTS

// Convenient shortcut for document.getElementById
const $ = document.getElementById.bind(document);

// The role of a client
const ROLE = {
    Spectator: 0,
    White    : 1,
    Black    : 2,
}

// ==================================== VARIABLES

// Gameplay variables
var legals  = new Set()
var role    = null
var playing = null

// Played move info
var src = null
var dst = null

// The websocket, allowing communication with the server
var socket = null

// ==================================== PARSING

// Converts a string to a role
function parseRole(s) {
    switch (s) {
        case "s": return ROLE.Spectator
        case "w": return ROLE.White
        case "b": return ROLE.Black
    }
}

// Parse a history string and change the client's text accordingly
function parseHistory(s) {
    let i = 1
    let j = 0
    let res   = ""
    let split = s.split(",")

    while (i < split.length) {
        res += (++j) + ". " + split[i++]
        if (i < split.length)
            res += " " + split[i++] + " "
    }

    $("moves").innerHTML = res
}

// Parse a status string and change the client's status accordingly
function parseStatus(s) {
    switch (s) {
        case "w": playing = ROLE.White; break
        case "b": playing = ROLE.Black; break
        default: playing = null; break
    }

    if (role === playing) {
        $("status").innerHTML = "•It's your turn!"
        return
    }

    switch (s) {
        case "w":  $("status").innerHTML = "•It's white's turn"; break
        case "b":  $("status").innerHTML = "•It's black's turn"; break
        case "d":  $("status").innerHTML = "•The game is drawn"; break
        case "wm": $("status").innerHTML = "•White won by checkmate"; break
        case "bm": $("status").innerHTML = "•Black won by checkmate"; break
    }
}

// Parse an "info" command
function parseInfo(args) {
    // Update our role
    let oldRole = role
    role = parseRole(args[1])
    switch (role) {
        case ROLE.Spectator: 
            $("role").innerHTML = "•You are spectating"
            break
        case ROLE.White: 
            $("role").innerHTML = "•You are playing as white"
            if (oldRole !== role) $("board").orientation = "white"
            break
        case ROLE.Black: 
            $("role").innerHTML = "•You are playing as black"
            if (oldRole !== role) $("board").orientation = "black"
            break
    }

    // Update the list of legal moves
    legals = new Set(args[2].split(","))
}

// Parse a "state" command
function parseState(args) {
    // Set the new position of the board
    $("board").setPosition(args[1])
    // Set the game's history
    parseHistory(args[2])
    // Update status
    parseStatus(args[3])
}

// ==================================== EVENT HANDLING

// Fired when the socket just received a message
function socketMessaged(msg) {
    let args = msg.data.split(" ")

    switch (args[0]) {
        case "info" : parseInfo(args);  break
        case "state": parseState(args); break
    }
}

// Fired when clicking on promote button
function promote(piece) {
    // Set promote pop-up to hidden
    $("promote").style.visibility = "hidden"

    // Play move
    $("board").move({
        from: src,
        to: dst,
        promotion: piece,
    });

    // Send move to server
    socket.send("move " + src + dst + piece)
    
    // Enable dragging
    $("board").draggablePieces = true
}

// When the window is ready
window.onload = function() {
    // Initialize the socket
    let uri = "ws://" + window.location.host + "/ws/"
    socket = new WebSocket(uri)
    socket.onmessage = socketMessaged

    // Set click events for the promote pop-up
    $("promote-queen").addEventListener("click", (e) => promote("q"))
    $("promote-rook").addEventListener("click", (e) => promote("r"))
    $("promote-bishop").addEventListener("click", (e) => promote("b"))
    $("promote-knight").addEventListener("click", (e) => promote("n"))

    // When dragging a piece
    $("board").addEventListener("drag-start", (e) => {
        // If the piece is not of our color, or it's not our turn, prevent dragging
        if (role !== playing || parseRole(e.detail.piece[0]) !== role)
            e.preventDefault()
    })

    // When dropping a piece
    $("board").addEventListener("drop", (e) => {
        let setAction = e.detail.setAction

        src = e.detail.source
        dst = e.detail.target

        let move = src + dst

        // If move is legal
        if (legals.has(move)) {
            // Send it to server
            socket.send("move " + move)
            return
        }
        
        // If move + promoting to queen is legal
        if (legals.has(move + "q")) {
            // Disable dragging
            $("board").draggablePieces = false
            // Set promote pop-up visible
            $("promote").style.visibility = "visible"

            // Set portraits
            switch (role) {
                case ROLE.White:
                    renderPiece("wQ", $("promote-queen"))
                    renderPiece("wR", $("promote-rook"))
                    renderPiece("wB", $("promote-bishop"))
                    renderPiece("wN", $("promote-knight"))
                    break
                case ROLE.Black:
                    renderPiece("bQ", $("promote-queen"))
                    renderPiece("bR", $("promote-rook"))
                    renderPiece("bB", $("promote-bishop"))
                    renderPiece("bN", $("promote-knight"))
                    break
            }
        }

        // Move is invalid
        setAction("snapback")
    })

    // Ask to join the game
    $("join").onclick = function() {
        socket.send("play")
    }

    // Invite an AI
    $("invite").onclick = function() {
        socket.send("invite")
    }

    // Flip the board
    $("flip").onclick = function() {
        $("board").flip()
    }
}