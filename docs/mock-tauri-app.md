# Mock Tauri example

The `examples/mock-tauri-app` project demonstrates the full BridgeKit stack
without Apple or Microsoft store credentials:

- `bridgekit` Rust core
- `MockStoreProvider`
- `MockPushProvider`
- `tauri-plugin-bridgekit`
- `@bridgekit/tauri` TypeScript SDK

## What it does

The Rust side registers a mock Apple product named `pro.monthly` and a mock APNs
token. The frontend calls the TypeScript SDK to:

1. load products
2. purchase `pro.monthly`
3. restore purchases
4. request push authorization
5. register for push notifications

## Run

Install workspace dependencies from the repository root:

```sh
npm install
```

Run the example:

```sh
cd examples/mock-tauri-app
npm run tauri dev
```

## Checks

From the repository root:

```sh
npm run typecheck
npm run build
cargo test
```

The example is included in the root npm workspace and Cargo workspace, so those
commands typecheck/build the frontend and compile the Rust app entrypoint.
