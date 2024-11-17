# ya-ya

### Prerequisites

```
npm install
```

```
cargo install cargo-watch
```

```
cargo install cargo-leptos
```

```
wasm-pack build wasm/front
```

```
wasm-pack build wasm/background
```

```
wasm-pack build wasm/popup
```

### Development

```
cargo watch -i "**/pkg/" -i "**/env.rs"  -s "wasm-pack build wasm/front && wasm-pack build wasm/background && wasm-pack build wasm/popup && npm run dev"
```


```
cd functions

zip -vr functions.zip . &&  yc serverless function version create --function-name=translate-word --source-path functions.zip --runtime python312 --entrypoint word.handler --service-account-id=<SERVICE_ACCOUNT> --environment FN_MODEL_FOLDER_ID=<FOLDER>
```
