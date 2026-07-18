import {
  products,
  purchase,
  registerPush,
  requestPushAuthorization,
  restorePurchases,
  validateReceipt,
} from "@bridgekit/tauri";

// Replace with the Store ID from Partner Center.
const PRODUCT_ID = "9NBLGGH4R315";

const output = document.querySelector<HTMLPreElement>("#output");
const button = document.querySelector<HTMLButtonElement>("#run-demo");

button?.addEventListener("click", () => {
  void runNativeFlow();
});

async function runNativeFlow(): Promise<void> {
  write("Running native Windows BridgeKit flow...");

  const availableProducts = await products({
    productIds: [PRODUCT_ID],
    storefront: "microsoft_store",
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
        storefront: "microsoft_store",
      })
    : null;

  const authorization = await requestPushAuthorization({
    alert: true,
    badge: true,
    sound: true,
  });

  const registration = await registerPush({
    environment: "production",
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
