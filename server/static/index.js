window.onload = function() {
    var canvas = document.getElementById("canvas"),
    context = canvas.getContext("2d");
    
    var atlas = document.createElement("img");
    atlas.src = "atlas.png";
    atlas.onload = function() {
        context.drawImage(atlas, 0, 0, 128, 128, 0, 0, 100, 100);
    }
}