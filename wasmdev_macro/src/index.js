
import init from '/index.js';
init("/index.wasm");
// -- debug -- \\
var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
var url = protocol + "//" + window.location.host;

var on_msg = msg => {
    if(!msg || !msg.data || !(msg.data + "").startsWith("reload ")) return;
    var path = msg.data.split(" ")[1];
    if(!path) return;
    if (path.includes("index.wasm") || path.includes("index.html")){
        window.location.reload();
    } else if (path.endsWith(".js")) {
        for (var script of document.querySelectorAll('script[src]')) {
            if(script.src?.split("?")[0].endsWith(path)){
                script.src = path + "?t=" + new Date().getTime();
            }
        }
    } else if (path.endsWith(".css")) {
        for (var style of document.querySelectorAll('link[rel="stylesheet"]')) {
            if(style.href?.split("?")[0].endsWith(path)){
                style.href = path + "?t=" + new Date().getTime();
            }
        }
    }
}

var reconnect = () => window.setTimeout(open_websocket, 5000);

var open_websocket = init => {
    var ws = new WebSocket(url);
    if (!init) ws.onopen = () => window.location.reload();
    ws.onmessage = msg => on_msg(msg);
    ws.onclose = reconnect;
}

open_websocket(true);
