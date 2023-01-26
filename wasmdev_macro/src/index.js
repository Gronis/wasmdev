let url = "ws://" + window.location.host;
let ws = new WebSocket(url);
ws.onopen = () => {
    ws.send("hejsan");
};
ws.onmessage = msg => {
    console.log(msg);
}