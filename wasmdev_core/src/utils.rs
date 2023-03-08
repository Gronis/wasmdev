
use std::io::{self, Read};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use minify_js::{Session, TopLevelMode, minify};
use xshell::{Shell, cmd};
use wasm_bindgen_cli_support::Bindgen;

pub fn find_files(path: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    let mut paths = vec![path.to_path_buf()];
    let mut traverse = |paths_in: &mut Vec<PathBuf>| -> io::Result<Vec<PathBuf>> {
        let mut paths_out = vec![];
        for path in paths_in.drain(..) {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    paths_out.push(path);
                } else {
                    files.push(path);
                }
            }
        }
        Ok(paths_out)
    };

    // Recurse 3 layers down
    let Ok(mut paths) = traverse(&mut paths) else { return vec![] };
    let Ok(mut paths) = traverse(&mut paths) else { return vec![] };
    let Ok(_)         = traverse(&mut paths) else { return vec![] };

    files
}

pub fn load_file(file_path: &Path) -> Option<Vec<u8>> {
    let mut file_handle = File::open(file_path).ok()?;
    let mut file_contents = Vec::new();
    file_handle.read_to_end(&mut file_contents).ok()?;
    Some(file_contents)
}

pub fn build_wasm(input_path: &str, is_release: bool, target_dir: &str) -> Option<()> {
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
    let output_path = Path::new(input_path).parent().expect("No parent when building wasm");
    Bindgen::new()
        .input_path(input_path)
        .web(true)
        .map_err(|err| println!("{}", err)).ok()?
        .demangle(!is_release)
        .debug(!is_release)
        .remove_name_section(is_release)
        .remove_producers_section(is_release)
        .generate(output_path).map_err(|err| println!("{}", err)).ok()
}

pub fn minify_javascript(code_in: &[u8]) -> Vec<u8>{
    let session = Session::new();
    let mut code_out = vec![];
    minify(&session, TopLevelMode::Module, code_in, &mut code_out).unwrap();
    code_out
} 