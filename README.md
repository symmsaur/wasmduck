# Minimal deploy
Build with `cargo build`
Copy `target/wasm32-unknown-unknown/debug/wasmduck.wasm` to `deploy/`
Serve with `python serve.py 8080` or whichever port you want.

# Tests
Execute tests using `cargo test --target x86_64-unknown-linux-gnu`.
