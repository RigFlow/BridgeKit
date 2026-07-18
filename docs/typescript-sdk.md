# TypeScript SDK

The `@bridgekit/tauri` package wraps the `tauri-plugin-bridgekit` commands with
typed TypeScript functions and request/response interfaces.

## Install

```sh
npm install @bridgekit/tauri
```

When consuming directly from this repository:

```json
{
  "dependencies": {
    "@bridgekit/tauri": "github:RigFlow/BridgeKit#cursor/bridgekit-iap-push-fbcb"
  }
}
```

## Usage

```ts
import {
  products,
  purchase,
  registerPush,
  requestPushAuthorization,
} from "@bridgekit/tauri";

const availableProducts = await products({
  productIds: ["pro.monthly"],
  storefront: "apple_app_store",
});

const transaction = await purchase({
  productId: "pro.monthly",
});

await requestPushAuthorization({
  alert: true,
  badge: true,
  sound: true,
});

const registration = await registerPush({
  environment: "production",
});
```

## API

### In-app purchases

- `products(request: ProductRequest): Promise<Product[]>`
- `purchase(request: PurchaseRequest): Promise<Purchase>`
- `restorePurchases(): Promise<Purchase[]>`
- `validateReceipt(request: ReceiptValidationRequest): Promise<ReceiptValidationResult>`

### Push notifications

- `requestPushAuthorization(request?: PushAuthorizationRequest): Promise<PushAuthorization>`
- `registerPush(request?: PushRegistrationRequest): Promise<PushRegistration>`
- `unregisterPush(): Promise<void>`

### Command constants

If an app needs direct `invoke` access, the SDK exports the command strings:

```ts
import { COMMANDS } from "@bridgekit/tauri";

console.log(COMMANDS.purchase);
```

## Types

The package exports the TypeScript types that mirror the Rust `bridgekit` serde
payloads, including:

- `ProductRequest`
- `Product`
- `PurchaseRequest`
- `Purchase`
- `ReceiptValidationRequest`
- `ReceiptValidationResult`
- `PushAuthorizationRequest`
- `PushAuthorization`
- `PushRegistrationRequest`
- `PushRegistration`
- `Storefront`
- `Platform`
