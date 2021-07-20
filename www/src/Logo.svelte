<!-- Components -->

<img src="svg/logo.svg" alt="The Rush chess engine logo.">

<!-- Styles -->

<style>
    img {
        position: absolute;
        width: 30vmin;
        animation: change-color 5s infinite linear, rotation 40s infinite linear, move-x 13s infinite linear alternate, move-y 7s infinite linear alternate;
    }

    @keyframes rotation {
        from {transform: rotate(0deg);}
        to {transform: rotate(359deg);}
    }

    @keyframes move-x {
        from {left: 0;} 
        to {left: calc(100% - 30vmin);}
    }

    @keyframes move-y {
        from {top: 0;} 
        to {top: calc(100% - 30vmin);}
    }

    @keyframes change-color {
        from {filter: hue-rotate(0def);}
        to {filter: hue-rotate(359deg);}
    }
</style>