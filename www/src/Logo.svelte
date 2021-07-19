<script>
    // Reference to the logo <img> element.
    let logo;

    import {onMount} from "svelte";

    // Change the color of the logo by rotating it's hue by a random amount.
    function changeColor() {
        const rand = Math.floor(Math.random() * 360);
        logo.style.filter = `hue-rotate(${rand}deg)`
    }

    // Schedule the logo to change color on each bounce.
    onMount(() => {
        changeColor();
        setInterval(changeColor, 6000);
        setInterval(changeColor, 14000);
    })
</script>

<!-- Components -->

<img bind:this={logo} src="svg/logo.svg" alt="The Rush chess engine logo">

<!-- Styles -->

<style>
    img {
        position: absolute;
        width: 16em;
        animation: rotation 40s infinite linear, moveX 14s infinite linear alternate, moveY 6s infinite linear alternate;
    }

    @keyframes rotation {
        from {transform: rotate(0deg);}
        to {transform: rotate(359deg);}
    }

    @keyframes moveX {
        from {left: 0;} 
        to {left: calc(100% - 16em);}
    }

    @keyframes moveY {
        from {top: 0;} 
        to {top: calc(100% - 16em);}
    }
</style>