import { make } from "wasm-ferris-blocks";

function run() {
    document.getElementById('ferris-blocks-canvas').appendChild(make());
}

run();