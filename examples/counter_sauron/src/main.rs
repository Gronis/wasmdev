use sauron::{*, html::text};
enum Msg { Increment, Decrement }
struct App { count: i32 }

impl Application<Msg> for App {
    fn view(&self) -> Node<Msg> { node! {
        <table>
            <td>
                <tr><button on_click=|_| Msg::Increment>{text("+")}</button></tr>
                <tr><div>{text(self.count)}</div></tr>
                <tr><button on_click=|_| Msg::Decrement>{text("-")}</button></tr>
            </td>
        </table>
    }}

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::Decrement => self.count -= 1,
        }
        Cmd::none()
    }

    fn style(&self) -> String { jss! {
        "body": { font_family: "Arial, Helvetica, sans-serif" },
        "button": { width: percent(100) },
        "div": { text_align: "center" },
    }}
}

#[wasmdev::main(port: 3000)]
fn start() {
    Program::mount_to_body(App { count: 0 });
}
