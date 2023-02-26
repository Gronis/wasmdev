use sycamore::prelude::*;

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let state = create_signal(cx, 0);
    let increment = |_| state.set(*state.get() + 1);
    let decrement = |_| state.set(*state.get() - 1);
    view! { cx,
        table {
            td {
                tr { button(on:click=increment) {"+"} }
                tr { div { (state.get()) } }
                tr { button(on:click=decrement) {"-"} }
            }
        }
    }
}

#[wasmdev::main(port: 3000, path: "www")]
fn main() {
    sycamore::render(App);
}