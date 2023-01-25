
pub fn hej(a: i32) -> i32{
    a + a
}

pub fn app() {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p").expect("Unable to create element");
    val.set_text_content(Some(&format!("Hello from Rust! {}", hej(3))));
    body.append_child(&val).expect("Unable to append element");
}