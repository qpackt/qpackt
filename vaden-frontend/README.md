<h1>Installing from source</h1>

```bash
rustup target add wasm32-unknown-unknown
```

Install trunk
```bash
cargo install --locked trunk
```

To run:
```bash
trunk serve
```

To build for deployment:
```bash
trunk build --release --public-url "/vaden"
```