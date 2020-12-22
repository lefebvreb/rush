var ctx, atlas

// When window is ready
window.onload = function() {
    ctx = document.getElementById("canvas").getContext("2d");
    
    atlas = document.createElement("img");
    atlas.src = "atlas.png";
    atlas.onload = draw
}

function drawImage(sx, sy, dx, dy) {
    ctx.drawImage(atlas, sx*128, sy*128, 128, 128, dx*128, dy*128, 128, 128);
}

// Draw the board
function draw() {
    for (var y=0; y<8; y++) {
        for (var x=0; x<8; x++) {
            drawImage(1, 0, x, y)
        }
    }
}