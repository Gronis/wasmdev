
// Dummy async function
async fn make_hello_world_text() -> String {
    "Hello Async".into()
}

// Using both tokio::main and wasmdev::main is no problem
// We have to use "current_thread" because a browser js env
// is single threaded
#[tokio::main(flavor = "current_thread")]
#[wasmdev::main]
async fn main() {

    // Grab "Hello World" text asyncronously
    let text = make_hello_world_text().await;

    // Grab document body
    let window = web_sys::window()
        .expect("no global `window` exists");
    let document = window.document()
        .expect("should have a document on window");
    let body = document.body()
        .expect("document should have a body");

    // Create Hello World paragraph
    let val = document.create_element("p")
        .expect("Unable to create element");
    val.set_text_content(Some(&text));

    // Append text to document body:
    body.append_child(&val)
        .expect("Unable to append element");
}
