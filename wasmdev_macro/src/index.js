
import init from '/index.js';
init("/index.wasm");
// -- debug -- \\
var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
var url = protocol + "//" + window.location.host;

var reconnect = () => window.setTimeout(initialize_reload_watcher, 5000);

var handle_msg = msg => {
    if(!msg || !msg.data || !msg.data.startsWith("reload ")) return;
    var path = msg.data.split(" ")[1];
    if(!path) return;
    if (path.includes("index.wasm") || path.includes("index.html")){
        window.location.reload();
    } else if (path.endsWith(".js")) {
        for (var script of document.querySelectorAll('script[src]')) {
            if(script.src?.endsWith(path)){
                script.src = "";
                script.src = path;
            }
        }
    } else if (path.endsWith(".css")) {
        for (var style of document.querySelectorAll('link[rel="stylesheet"]')) {
            if(style.href?.endsWith(path)){
                style.href = "";
                style.href = path;
            }
        }
    }
}

var initialize_reload_watcher = first_try => {
    var ws = new WebSocket(url);
    if (!first_try) ws.onopen = () => window.location.reload()
    ws.onmessage = msg => handle_msg(msg)
    ws.onclose = () => reconnect()
}

initialize_reload_watcher(true);
