# ya-ya

### Prerequisites

```
npm install
```

```
cargo install cargo-watch
```

```
cargo install wasm-pack
```

```
wasm-pack build wasm/front
```

```
wasm-pack build wasm/background
```

### Development

```
cargo watch -i "**/pkg/" -s "wasm-pack build wasm/front && wasm-pack build wasm/background && npm run dev"
```
