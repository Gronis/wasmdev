# wasmdev

[![crates.io](https://img.shields.io/crates/v/wasmdev.svg)](https://crates.io/crates/wasmdev)
[![docs.rs](https://docs.rs/wasmdev/badge.svg)](https://docs.rs/wasmdev)

Simple web development in rust based web frontends:
```rust
// src/main.rs
#[wasmdev::main]
fn main() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let val = document.create_element("p").unwrap();
    val.set_text_content(Some("Hello World"));
    body.append_child(&val).unwrap();
}

```
```bash
cargo run
```
```log
   Compiling my-web-app
    Finished dev [unoptimized + debuginfo] target(s)
     Running `target/debug/my-web-app`
    Building wasm target
   Compiling my-web-app
    Finished dev [unoptimized + debuginfo] target(s)
             ┏━━━━━━━━━━━━━━━━━━━━━━━━━┓
     Serving ┃  http://127.0.0.1:8080  ┃ <= Click to open your app!
             ┗━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

# Project Goal
wasmdev aims to provide the most simple way to develop your frontend web application for rust. The idea is to use `cargo` just like you do with a native/binary executable. No need to install tools like `trunk` or `wasm-pack`. Just add `wasmdev` to your dependencies and put a simple macro in front of your main function, and you have yourself a frontend web-development server! You can also build all web assets with a simple `cargo build --release`, and they will be minified and ready for distribution. How cool is that!

# Disclaimer
**Note:** Project is in early stage of development. Bugs or other problems might still be present, error messages might have a long way to go etc. Don't use for large or $$$ projects. Use more tested tools like `trunk` instead.

**Note:** The server application that is used to run and test your web frontend app is NOT suitable for hosting your web-app in a production environment. It lacks many common http server features and is only intended to be a fast and simple way to test and develop your web frontend app.

# Features
## What wasmdev **DO**:
wasmdev has similar features as `trunk`. Like:
* Auto-recompile and reload on rust/wasm on code changes
* Hot-reload on static file changes

It also has some features that `trunk` don't have (I believe), like:
* Optimized and minified release builds without additional tools or processes:
    * Run `cargo build --release` and you have your dist-optimized assets
* Auto-setup of `console_error_panic_hook` in your frontend app (can be disabled)

## What wasmdev **DOESN'T DO**:
* Server side rendering
* Transpilation of javascript to adhere to a certain ECMAScript version
* Bundle multiple javascript files together

# Configuration

The following options can be set to `wasmdev::main` macro:
* **port**: What tcp port to run the http server on. Defaults to port `8080`
* **path**: Where to put your static web assets like css styles, icons, logos etc. Defaults to the `"src"` folder.

`src/main.rs`:
```rust
#[wasmdev::main(port: 8080, path: "src")]
fn main() {
    //...
}
```

## Use-case: `index.html` override

By default, all files in `src` folder is served by the web server. You can add an `index.html` file here to override the minimalistic default one:
```
my-web-app
├── Cargo.lock
├── Cargo.toml
└── src
    ├── index.html
    └── main.rs
```
This is necessary to pull in additional assets like css styles or setup a Progressive Web Application using Service Workers.

## Use-case: override asset path:
If you want to have a separate path to static assets, they can be specified in the `wasmdev::main` macro as mention previously. This is recommended, since the web-server won't try to recompile your wasm-code when you change your static assets like css styles.

`src/main.rs`:
```rust
#[wasmdev::main(path: "www")]
fn main() {
    //...
}
```
Project file tree:
```
my-web-app
├── Cargo.toml
├── src
│   └── main.rs
└── www
    ├── index.css
    └── index.html
```

## Use-case: Don't include `console_error_panic_hook`
Just add `wasmdev` and ignore the default features:
```bash
cargo add wasmdev --no-default-features
```

# Build release version for distribution:

When building your project with a release build, the web assets (all javascript files and wasm code) will be built and optimized for release.
```bash
cargo build --release
```
```
   Compiling wasmdev_macro
   Compiling wasmdev
   Compiling my-web-app
    Finished release [optimized] target(s)
    Finished release artifacts in: 'target/dist/my-web-app'
    Finished release [optimized] target(s)
```
The release artifacts will be located at `target/dist/{project_name}`
```
my-web-app
└── target
    └── dist
        └── my-web-app
            ├── index.html
            ├── index.js
            └── index.wasm
```
When building in release mode, cache invalidation of build artifacts might not always work. This can happen if:
* You create a new static asset without modifying the rust source code or any existing static asset.

Changing any rust file in src directory, or pre-existing static asset fixes this. 

# Running examples

All examples can be built and executed by cargo like this:
```bash
cargo run -p <example>
# Run the simple project that outputs "Hello World" in the web-browser implemented with web_sys bindings:
cargo run -p simple
```
See `examples` folder for a complete list of examples.

## TODO:
Missing features that will or might be added in the future:

* Unit tests
* Docs
* More examples for popular web projects (yew, sycamore, etc).
* Implement an easy way to run all tests (in the browser) by simply running `cargo test` with the test result in the cli window. Not sure if this is possible.

## License
MIT

## Contributors:

* Robin Grönberg