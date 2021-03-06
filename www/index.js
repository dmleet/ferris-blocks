import { make } from "wasm-ferris-blocks";

let game;
async function run() {
    game = make();
}
let started = false;
const start = () => {
    if (started) return;
    started = true;

    const delay = 400;
    let last = Date.now();
    function mainLoop() {
        if ((Date.now() - last) > delay) {
            game.tick();
            last = Date.now();
        }
        //if (!game.game_over) { requestAnimationFrame(mainLoop); }
        requestAnimationFrame(mainLoop);
    }
    requestAnimationFrame(mainLoop);

    function keyboardControls(event) {
        if (event.keyCode === 65) {
            game.move_left();
        } else if (event.keyCode === 87) {
            game.rotate();
        } else if (event.keyCode === 68) {
            game.move_right();
        } else if (event.keyCode === 83) {
            game.move_down();
        }
        last = Date.now();
    }
    document.addEventListener('keydown', keyboardControls);
};
run().then(
    document.getElementById("ferris-blocks-canvas"), addEventListener("click", start)
)