// ==================================== CONSTANTS

const ROLE = {
    Spectator: 0,
    White: 1,
    Black: 2,
}

// ==================================== VARIABLES

var board   = null
var legals  = []
var playing = null
var role    = ROLE.Spectator
var socket  = null
var status  = null

// ==================================== PARSING

// Converts a string to a role
function stringToRole(s) {
    switch (s) {
        case "s": return ROLE.Spectator
        case "w": return ROLE.White
        case "b": return ROLE.Black
    }
}

// 
function parseStatus(s) {
    switch (s) {
        case "w": case "W": role = ROLE.White; break
        case "b": case "B": role = ROLE.Black; break
    }

    if ("swb"[role] == roleToString(role)) {
        status = "It's your turn!"
        return
    }

    switch (s) {
        case "w": status = "White is currently playing, it is a human player"; break
        case "b": status = "Black is currently playing, it is a human player"; break
        case "W": status = "White is currently playing, it is a computer"; break
        case "B": status = "Black is currently playing, it is a computer"; break
        case "d": status = "The game is drawn"; break
        case "m": status = "White won by checkmate"; break
        case "M": status = "Black won by checkmate"; break
    }
}

function parse_info(args) {
    // Update our role
    role = stringToRole(args[1])
    // Update the list of legal moves
    legals = args[2].split(",")
}

function parse_state(args) {
    // Set the new position of the board
    board.setPosition(args[1])
    history = args[2].split(",")
    parseStatus(args[3])
}

// ==================================== SOCKET EVENTS HANDLING

function initSocket() {
    let uri = "ws://" + window.location.host + "/ws/"
    socket = new WebSocket(uri)
    socket.onmessage = socketMessaged
}

// Fired when the socket just received a message
function socketMessaged(msg) {
    let args = msg.data.split(" ")

    switch (split[0]) {
        case "info":  parse_info(args);  break
        case "state": parse_state(args); break
    }
}

// ==================================== WINDOW EVENTS HANDLING

window.onload = function() {
    board = document.getElementById("board")
    initSocket()
}