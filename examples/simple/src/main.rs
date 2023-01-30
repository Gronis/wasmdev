
#[wasmdev::main]
fn main() {
    console_error_panic_hook::set_once();

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
    val.set_text_content(Some("Hello Rust!"));

    // Append text to document body:
    body.append_child(&val)
        .expect("Unable to append element");
}
