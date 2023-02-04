use leptos::*;
#[wasmdev::main]
fn main() {
    mount_to_body(|cx| view! { cx,  
        <p>"Hello Leptos"</p>
    })
}
