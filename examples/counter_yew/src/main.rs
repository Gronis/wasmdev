use yew::prelude::*;

#[function_component]
pub fn Counter() -> Html {
    let counter = use_state(|| 0);
    let increment = {
        let counter = counter.clone();
        move |_| { counter.set(*counter + 1); }
    };
    let decrement = {
        let counter = counter.clone();
        move |_| { counter.set(*counter - 1); }
    };
    html! {
        <table>
            <td>
                <tr><button onclick={increment}>{"+"}</button></tr>
                <tr><div>{*counter}</div></tr>
                <tr><button onclick={decrement}>{"-"}</button></tr>
            </td>
        </table>
    }
}

#[wasmdev::main(port: 3000, path: "www")]
fn main() {
    yew::Renderer::<Counter>::new().render();
}
