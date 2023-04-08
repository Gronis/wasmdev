use std::path::Path;
use minify_js::{Session, TopLevelMode, minify};
use xshell::{Shell, cmd};
use wasm_bindgen_cli_support::Bindgen;

pub fn build_wasm(input_path: impl AsRef<Path>, is_release: bool, target_dir: impl AsRef<Path>) -> Option<()> {
    let target_dir = target_dir.as_ref().to_str()?;
    let mut args = vec![
        "--target", "wasm32-unknown-unknown",
        "--target-dir", target_dir,
        "--color", "always",
    ];
    if is_release { args.push("--release") };
    let args = args; // Remove mutability;
    let sh = Shell::new().expect("Unable to create shell");
    {
        // This lets wasmdev::main know if cargo was started from within wasmdev::main
        let _env_guard = sh.push_env("CARGO_WASMDEV", "1");
        cmd!(sh, "cargo build").args(args).quiet().run().ok()?;
    }
    let output_path = input_path.as_ref().parent()?;
    Bindgen::new()
        .input_path(&input_path)
        .web(true)
        .map_err(|err| println!("{}", err)).ok()?
        .demangle(!is_release)
        .debug(!is_release)
        .remove_name_section(is_release)
        .remove_producers_section(is_release)
        .generate(output_path)
        .map_err(|err| println!("{}", err)).ok()
}

pub fn minify_javascript(code_in: &[u8]) -> Option<Vec<u8>>{
    let session = Session::new();
    let mut code_out = vec![];
    minify(&session, TopLevelMode::Module, code_in, &mut code_out).ok()?;
    Some(code_out)
} 