
import init from '/index.js';
window.init = init;
init("/index.wasm");

var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
var url = protocol + "//" + window.location.host;

var initialize_reload_watcher = first_try => {
    var ws = new WebSocket(url);
    if (!first_try) 
        ws.onopen = () => window.location.reload();
    ws.onmessage =  () => window.location.reload();
    ws.onclose = () => window.setTimeout(initialize_reload_watcher, 5000);
}

initialize_reload_watcher(true);
