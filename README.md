# wasmdev
Main repo for wasmdev, a rust web development server.

**Note:** Project is in early stage of development. While it is near feature complete, bugs or other problems might still be present. Don't use for large or $$$ projects. Use more tested tools like `trunk` instead.

**Note:** The server application that is used to run and test your web frontend app is NOT suitable for hosting your web-app in a production environment. It lacks many common http server features and is only intended to be a fast and simple way to test and develop your web frontend app.

wasmdev is a good solution when the following is true for your web application project
* Your frontend is written entirely in `rust` using popular rust web frontends like `yew` or similar
* Your frontend web app is a Single Page Application (i.e not server rendered)
* You want an easy way to start testing your frontend
* You want a **batteries included** experience where everything you need is installed togeather with the rest of your app dependences (e.g no extra cli-tools like trunk or wasm-pack etc)
* You have a separate project for your web-api backend, or your frontend is a static web-page with no backend needed (wasmdev web-server only serves your web-page, it does not handle custom api requests)

## Features
wasmdev has similar features as `trunk`. Like:
* Auto-recompile and reload on rust/wasm code-changes
* Hot-reload on static file-changes

It also has some features that `trunk` hasn't (I belive), like:
* Optimized and minified release builds without additional tools or processes
* Auto-setup of `console_error_panic_hook` in your frontend app (can be disabled)

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

### Example: `index.html` override

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

### Example: override asset path:
If you want to have a separate path to static assets, the can be specified in the `wasmdev::main` macro as mention previously. This is recommended, since the web-server won't try to recompile your wasm-code when you change your static assets like css styles.

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

## Release ready build:

When running your project with a release build, the web assets (generated javascript files and wasm code) will be optimized for release and any hot-reload code is removed.
```
cargo run --release
```
When navigating to `127.0.0.1:8080` you will notice that the javascript code (index.js) and wasm code (index.wasm) is minified and reduced in size.

Currently, the optimized versions exists only in working memory on the web-server, so exporting the app is very manual at the moment. This is expected to change in the future.

## TODO:
Missing features that will or might be added in the future:

* When running in release mode: export all assets for easier production deploy process
* Minify any static javascript (not just generated index.js) assets when doing a release export
* Write tests and more examples for popular web projects (yew, sycamore, etc).
* Implement an easy way to run all tests (in the browser) by simply running `cargo test` with the test result in the cli window. Not sure if this is possible, but it is a goal. This is one of the reason why I wrote wasmdev to begin with so if this is implemented, it would be great.

## License
TBD

## Contributors:

* Robin Grönberg