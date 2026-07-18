import {
  products,
  purchase,
  registerPush,
  requestPushAuthorization,
  restorePurchases,
  validateReceipt,
} from "@bridgekit/tauri";

const PRODUCT_ID = "pro.monthly";

const output = document.querySelector<HTMLPreElement>("#output");
const button = document.querySelector<HTMLButtonElement>("#run-demo");

button?.addEventListener("click", () => {
  void runNativeFlow();
});

async function runNativeFlow(): Promise<void> {
  write("Running native Apple BridgeKit flow...");

  const availableProducts = await products({
    productIds: [PRODUCT_ID],
    storefront: "apple_app_store",
  });

  const transaction = await purchase({
    productId: PRODUCT_ID,
  });

  const restored = await restorePurchases();

  const validation = transaction.receipt
    ? await validateReceipt({
        receipt: transaction.receipt,
        productId: transaction.productId,
        transactionId: transaction.transactionId,
        storefront: "apple_app_store",
      })
    : null;

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
        validation,
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
