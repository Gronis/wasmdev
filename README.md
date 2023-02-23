# wasmdev
Main repo for wasmdev, a rust web development server.

## Project Goal
wasmdev aims to provide the most simple way to develop your frontend web application for rust. The idea is to use `cargo` just like you do with a native/binary executable. No need to install tools like `trunk` or `wasm-pack`. Just add `wasmdev` to your dependencies and put a simple macro in front of your main function, and you have yourself a frontend web-development server! You can also build all web assets with a simple `cargo build --release`, and they will be minified and ready for distribution. How cool is that!

### Disclaimer
**Note:** Project is in early stage of development. Bugs or other problems might still be present, error messages might have a long way to go etc. Don't use for large or $$$ projects. Use more tested tools like `trunk` instead.

**Note:** The server application that is used to run and test your web frontend app is NOT suitable for hosting your web-app in a production environment. It lacks many common http server features and is only intended to be a fast and simple way to test and develop your web frontend app.

### Features
wasmdev has similar features as `trunk`. Like:
* Auto-recompile and reload on rust/wasm on code changes
* Hot-reload on static file changes

It also has some features that `trunk` don't have (I believe), like:
* Optimized and minified release builds without additional tools or processes:
    * Run `cargo build --release` and you have your dist-optimized assets
* Auto-setup of `console_error_panic_hook` in your frontend app (can be disabled)

### What wasmdev DOESN'T DO:
* Server side rendering
* Transpilation of javascript to adhere to a certain ECMAScript version 

## Setup a new web-project with wasmdev

A simple wasmdev project can be created like so:

```bash
# Make sure that we can build web-assemby targets
rustup target add wasm32-unknown-unknown
# Setup project and dependencies
cargo new my-web-app
cd my-web-app
cargo add wasmdev
cargo add web-sys --features Document Element HtmlElement Node Window
```

Edit `src/main.rs` like so:
```rust

// This macro takes care of all web-server stuff for you
#[wasmdev::main]
fn main() {
    // Everything in the main function is executed inside the browser on the client side,
    // so build your frontend here. This example uses web-sys which binds directly to the
    // the browser api for Node-DOM manipulations:

    // Grab document body
    let window = web_sys::window()
        .expect("no global `window` exists");
    let document = window.document()
        .expect("should have a document on window");
    let body = document.body()
        .expect("document should have a body");

    // Create Hello World paragraph
    let val = document.create_element("p")
        .expect("Unable to create element");
    val.set_text_content(Some("Hello World"));

    // Append text to document body:
    body.append_child(&val)
        .expect("Unable to append element");
}

```
Then, start your web-app like you would with any other rust project:

```bash
cargo run
```
Simpilfied output:
```log
   Compiling wasmdev
   Compiling my-web-app
    Finished dev [unoptimized + debuginfo] target(s)
     Running `target/debug/my-web-app`
     Serving /index.html
    Building wasm target
   Compiling wasmdev
   Compiling wasmdev_macro
   Compiling my-web-app
    Finished dev [unoptimized + debuginfo] target(s)
     Serving /index.wasm, /index.js
             ┏━━━━━━━━━━━━━━━━━━━━━━━━━┓
     Serving ┃  http://127.0.0.1:8080  ┃ <= Click to open your app!
             ┗━━━━━━━━━━━━━━━━━━━━━━━━━┛
```
This project is equivalent with the provided example project: `simple` that can be found in the `examples` folder.
## Configuration

The following options can be set to `wasmdev::main` macro:
* port: What tcp port to run the http server on. Defaults to port `8080`
* path: Where to put your static web assets like css styles, icons, logos etc. Defaults to the `"src"` folder.

`src/main.rs`:
```rust
#[wasmdev::main(port: 8080, path: "src")]
fn main() {
    //...
}
```

### Use-case: `index.html` override

By default, all files in `src` folder is served by the web server. You can add an `index.html` file here to override the minimalistic default one:
```
.
├── Cargo.lock
├── Cargo.toml
└── src
    ├── index.html
    └── main.rs
```
This is necessary to pull in additional assets like css styles or setup a Progressive Web Application using Service Workers.

### Use-case: override asset path:
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
.
├── Cargo.toml
├── src
│   └── main.rs
└── www
    ├── index.css
    └── index.html
```

### Use-case: Don't include `console_error_panic_hook`
Just add `wasmdev` and ignore the default features:
```bash
cargo add wasmdev --no-default-features
```

## Build release version for distribution:

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
.
└── dist
    └── my-web-app
        ├── index.html
        ├── index.js
        └── index.wasm
```
When building in release mode, cache invalidation of build artifacts might not always work. This can happen if:
* You create a new static asset without modifying the rust source code or any existing static asset.

Changing any rust file in src directory, or pre-existing static asset fixes this. 

## Running examples

All examples can be built and executed by cargo like this:
```bash
cargo run -p <example>
# Run the simple project that outputs "Hello World" in the web-browser implemented with web_sys bindings:
cargo run -p simple
```
See `examples` folder for a complete list of examples.

## TODO:
Missing features that will or might be added in the future:

* Write tests and more examples for popular web projects (yew, sycamore, etc).
* Implement an easy way to run all tests (in the browser) by simply running `cargo test` with the test result in the cli window. Not sure if this is possible.

## License
TBD

## Contributors:

* Robin Grönberg