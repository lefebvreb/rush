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
var legals = []
var role   = ROLE.Spectator

// The websocket, allowing communication with the server
var socket = null

// ==================================== PARSING

// Converts a string to a role
function parseRole(s) {
    switch (s) {
        case "s": 
            role = ROLE.Spectator
            $("role").innerHTML = "•You are spectating"
            break
        case "w": 
            role = ROLE.White
            $("role").innerHTML = "•You are playing as white"
            break
        case "b": 
            role = ROLE.Black
            $("role").innerHTML = "•You are playing as black"
            break
    }
}

// Parse a history string and change the client's text accordingly
function parseHistory(s) {
    let i     = 0
    let res   = ""
    let split = s.split(",")

    while (i < split.length) {
        res += (i + 1) + ". " + split[i++]
        if (i < split.length)
            res += " " + split[i++] + " "
    }

    $("moves").innerHTML = res
}

// Parse a status string and change the client's status accordingly
function parseStatus(s) {
    switch (s) {
        case "w": case "W": role = ROLE.White; break
        case "b": case "B": role = ROLE.Black; break
    }

    if ("swb"[role] == s) {
        $("status").innerHTML = "•It's your turn!"
        return
    }

    switch (s) {
        case "w": $("status").innerHTML = "•White is currently playing, it is a human player"; break
        case "b": $("status").innerHTML = "•Black is currently playing, it is a human player"; break
        case "W": $("status").innerHTML = "•White is currently playing, it is a computer"; break
        case "B": $("status").innerHTML = "•Black is currently playing, it is a computer"; break
        case "d": $("status").innerHTML = "•The game is drawn"; break
        case "m": $("status").innerHTML = "•White won by checkmate"; break
        case "M": $("status").innerHTML = "•Black won by checkmate"; break
    }
}

// Parse an "info" command
function parseInfo(args) {
    // Update our role
    parseRole(args[1])
    // Update the list of legal moves
    legals = args[2].split(",")
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

    switch (split[0]) {
        case "info" : parseInfo(args);  break
        case "state": parseState(args); break
    }
}

// When the window is ready
window.onload = function() {
    // Initialize the socket
    let uri = "ws://" + window.location.host + "/ws/"
    socket = new WebSocket(uri)
    socket.onmessage = socketMessaged
}