// #![feature(proc_macro_hygiene)]

// #[wasmdev::import]
// #[cfg(target_family = "wasm")]

#[wasmdev::main]
pub fn main() {
    console_error_panic_hook::set_once();
    simple::app();
}