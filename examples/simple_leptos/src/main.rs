use leptos::*;

#[component]
pub fn Counter(cx: Scope, initial_value: i32) -> impl IntoView {
    let (value, set_value) = create_signal(cx, initial_value);
    let decrement = move |_| set_value.update(|value| *value -= 1);
    let increment = move |_| set_value.update(|value| *value += 1);
    view! { cx,
        <div>
            <button on:click=decrement>"-1"</button>
            <span>"  Value: " {move || value.get().to_string()} "  "</span>
            <button on:click=increment>"+1"</button>
        </div>
    }
}

#[wasmdev::main(port: 3000, path: "www")]
fn main() {
    mount_to_body(|cx| view! { cx, 
        <Counter initial_value=0 />
    })
}
