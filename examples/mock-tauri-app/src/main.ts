import {
  products,
  purchase,
  registerPush,
  requestPushAuthorization,
  restorePurchases,
} from "@bridgekit/tauri";

const output = document.querySelector<HTMLPreElement>("#output");
const button = document.querySelector<HTMLButtonElement>("#run-demo");

button?.addEventListener("click", () => {
  void runMockFlow();
});

async function runMockFlow(): Promise<void> {
  write("Running mock BridgeKit flow...");

  const availableProducts = await products({
    productIds: ["pro.monthly"],
    storefront: "apple_app_store",
  });

  const transaction = await purchase({
    productId: "pro.monthly",
  });

  const restored = await restorePurchases();

  const authorization = await requestPushAuthorization({
    alert: true,
    badge: true,
    sound: true,
  });

  const registration = await registerPush({
    environment: "sandbox",
  });

  write(
    JSON.stringify(
      {
        availableProducts,
        transaction,
        restored,
        authorization,
        registration,
      },
      null,
      2,
    ),
  );
}

function write(message: string): void {
  if (output) {
    output.textContent = message;
  }
}
