window.onload = function() {
    var canvas = document.getElementById("canvas"),
    ctx = canvas.getContext("2d");
    
    var atlas = document.createElement("img");
    atlas.src = "atlas.png";
    atlas.onload = function() {
        for (var y=0; y<8; y++) {
            for (var x=0; x<8; x++) {
                ctx.drawImage(atlas, 0, 0, 128, 128, x*64, y*64, 64, 64);
            }
        }
    }
}