// ==================================== VARIABLES

const ROLE = {
    white: 0,
    black: 1,
    guest: 2,
}

// The canvas context2D
var ctx

// The atlas image
var atlas

// The role of the client
var role = ROLE.black

// ==================================== PARSING UTIL

// Gets the string corresponding to the square x, y
function to_square(x, y) {
    return "abcdefgh"[x] + "12345678"[y]
}

// ==================================== DRAWING UTLI

// Draw an image on the canvas at dx, dy
function drawImage(sx, sy, dx, dy) {
    ctx.drawImage(atlas, sx*128, sy*128, 128, 128, dx*128, dy*128, 128, 128);
}

// Fills the square x, y with the given color
function fillRect(x, y, color) {
    ctx.fillStyle = color
    ctx.fillRect(x*128, y*128, 128, 128)
}

// ==================================== DRAWING

// Draw the square x, y as white
function drawSquareWhite(x, y) {
    if ((x + y) % 2 !== 0) {
        fillRect(x, y, "#5d9e3f")
    }
}

// Draw the square x, y as black
function drawSquareBlack(x, y) {
    if ((x + y) % 2 === 0) {
        fillRect(x, y, "#5d9e3f")
    }
}

// Draw the board
function draw() {
    var drawSquare
    if (role === ROLE.black) {
        drawSquare = drawSquareBlack
    } else {
        drawSquare = drawSquareWhite
    }

    for (var y = 0; y < 8; y++)
        for (var x = 0; x < 8; x++)
            drawSquare(x, y)      
}

// ==================================== EVENT HANDLING

// When window is ready
window.onload = function() {
    canvas = document.getElementById("canvas")
    ctx = canvas.getContext("2d")
    
    atlas = document.createElement("img")
    atlas.src = "atlas.png"
    atlas.onload = draw

    canvas.addEventListener("click", onclick)
}

// Gets the coordinates of the click event and transforms them into board coordinates
function onclick(e) {
    var rect = e.target.getBoundingClientRect();
    var x = Math.floor(e.offsetX * 8 / (rect.right - rect.left))
    var y = Math.floor(e.offsetY * 8 / (rect.bottom - rect.top))

    if (role === ROLE.white)
        y = 7 - y

    console.log(to_square(x, y))
}
