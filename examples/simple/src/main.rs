#[wasmdev::main]
fn main() {
    console_error_panic_hook::set_once();
    simple::app();
}
