# ya-ya

### Prerequisites

```
npm install
```

```
cargo install cargo-watch
```

```
wasm-pack build wasm/front
```

```
wasm-pack build wasm/popup
```

```
wasm-pack build wasm/options
```

### Development

```
cargo watch \
-i "**/pkg/" \
-i "**/env.rs" \
-s "wasm-pack build wasm/front --debug && wasm-pack build wasm/options && wasm-pack build wasm/popup && npm run dev"
```


### Deploy

```
yc serverless api-gateway update --name=ya-ya-api-gw --spec=spec.yaml
```

```
cd functions

zip -vr functions.zip .
yc serverless function version create --function-name=translate-word \
  --source-path functions.zip \
  --runtime python312 \
  --entrypoint word.handler \
  --service-account-id=ajem26g1ji06b6fvn3gh \
  --environment FN_MODEL_FOLDER_ID=b1gompirgbut357v15gm \
  --environment YDB_ENDPOINT=grpcs://ydb.serverless.yandexcloud.net:2135 \
  --environment YDB_DATABASE=/ru-central1/b1gtihve0dnl8to5iv7k/etnngbkfqn4uqfcftp6v
yc serverless function version create --function-name=success-record \
  --source-path functions.zip \
  --runtime python312 \
  --entrypoint record.handler \
  --service-account-id=ajem26g1ji06b6fvn3gh \
  --environment YDB_ENDPOINT=grpcs://ydb.serverless.yandexcloud.net:2135 \
  --environment YDB_DATABASE=/ru-central1/b1gtihve0dnl8to5iv7k/etnngbkfqn4uqfcftp6v
```

### Build

```
wasm-pack build wasm/front --release
wasm-pack build wasm/options --release
wasm-pack build wasm/popup --release
npm run build
```
