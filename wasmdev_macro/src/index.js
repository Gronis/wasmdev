import init from '/index.js';
init("/index.wasm");

let url = "ws://" + window.location.host;
let ws = new WebSocket(url);
ws.onopen = () => {
    ws.send("Hello WebSocket");
};
ws.onmessage = msg => {
    console.log("WebSocket message:", msg);
}