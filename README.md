# wasmdev

[![crates.io](https://img.shields.io/crates/v/wasmdev.svg)](https://crates.io/crates/wasmdev)
[![docs.rs](https://docs.rs/wasmdev/badge.svg)](https://docs.rs/wasmdev)

Simple web development for web frontends written in rust:

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
          ┏━━━━━━━━━━━━━━━━━━━━━━━┓
  Serving ┃ http://127.0.0.1:8080 ┃ <= Click to open your app!
          ┗━━━━━━━━━━━━━━━━━━━━━━━┛
```

# Project Goal
wasmdev aims to provide the most simple way to develop your rust frontend web application. The idea is to use `cargo` just like you would do when developing a native/binary executable. No need to install tools like `trunk` or `wasm-pack`. Just add `wasmdev` to your dependencies and add a macro in front of your main function, and you have yourself a web server fit for rapid development! You can also build all web assets with a simple `cargo build --release`, and they will be minified and ready for distribution. How cool is that!

# Disclaimers
**Note:** Project is in early stage of development. Bugs or other problems might still be present, error messages might have a long way to go etc. Don't use for large or $$$ projects. Use more tested tools like `trunk` instead.

**Note:** The server application that is used to run and test your web frontend app is NOT suitable for hosting your web-app in a production environment. It lacks many common http server features and is only intended to be a fast and simple way to test and develop your web frontend app.

# Features
### What wasmdev **DO**:
wasmdev has similar features as `trunk`. Like:
* Auto-recompile and reload on rust/wasm on code changes
* Hot-reload on static file changes (like css-styles)

It also has some features that `trunk` don't have (I believe), like:
* Optimized and minified release builds without additional tools or processes:
    * Run `cargo build --release` and you have your dist-optimized assets
* Auto-setup of `console_error_panic_hook` in your frontend app (can be disabled)

### What wasmdev **DOESN'T DO**:
* Server side rendering
* Transpilation of javascript to adhere to a certain ECMAScript version
* Bundle multiple javascript files together
* No `sass` or `less`. Might be implemented in the future as optional features

# Configuration

The following options can be set to `wasmdev::main` macro:
* **addr**: Socket address to webserver 
  - Default "127.0.0.1"
* **path**: Path to static web assets 
  - Default: "src"
* **port**: TCP socket port to use 
  - Default: 8080
* **watch**: Reload assets on file-system changes
  - Default: true
  - Note: **Only affects debug build**, always false for release build

```rust
// src/main.rs
#[wasmdev::main(port: 8080, path: "src")]
fn main() {
    //...
}
```

## Use-case: `index.html` override

By default, all files in `src` folder is served by the web server. You can add your `index.html` file here to override the default one. This is necessary to pull in additional assets like css styles.
```html
<!doctype html>
<html>
    <head><link rel="stylesheet" href="/index.css"></head>
    <body></body>
</html>
```
Project file-tree:
```
├── Cargo.toml
└── src
    ├── index.css
    ├── index.html
    └── main.rs
```
## Use-case: override asset path:
If you want to have a separate path to static assets, they can be specified in the `wasmdev::main` macro as mention previously. This is recommended, since the web-server won't try to recompile your wasm-code when you change your static assets.
```rust
// src/main.rs
#[wasmdev::main(path: "www")]
fn main() {
    //...
}
```
Project file-tree:
```
├── Cargo.toml
├── src
│   └── main.rs
└── www
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
Compiling my-web-app
 Finished release [optimized] target(s)
 Finished release artifacts in: 'target/dist/my-web-app'
 Finished release [optimized] target(s)
```
The release artifacts will be located at `target/dist/{project_name}`
```
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

# Code examples

All examples can be built and executed by cargo like this:
```bash
cargo run -p <example>
# Run the simple project that outputs "Hello World"
cargo run -p simple
```
See `examples` folder for a complete list of examples.

## License
* MIT

## Contributors:
* Robin Grönberg

## TODO:

* Write unit tests
* Implement an easy way to run all tests (in the browser) by simply running `cargo test` with the test result in the cli window. Not sure if this is possible.