import { invoke } from "@tauri-apps/api/core";
import type {
  Product,
  ProductRequest,
  Purchase,
  PurchaseRequest,
  PushAuthorization,
  PushAuthorizationRequest,
  PushRegistration,
  PushRegistrationRequest,
  ReceiptValidationRequest,
  ReceiptValidationResult,
} from "./types.js";

export type * from "./types.js";

export const COMMANDS = {
  products: "plugin:bridgekit|bridgekit_products",
  purchase: "plugin:bridgekit|bridgekit_purchase",
  restorePurchases: "plugin:bridgekit|bridgekit_restore_purchases",
  validateReceipt: "plugin:bridgekit|bridgekit_validate_receipt",
  requestPushAuthorization:
    "plugin:bridgekit|bridgekit_request_push_authorization",
  registerPush: "plugin:bridgekit|bridgekit_register_push",
  unregisterPush: "plugin:bridgekit|bridgekit_unregister_push",
} as const;

export async function products(request: ProductRequest): Promise<Product[]> {
  return invoke<Product[]>(COMMANDS.products, { request });
}

export async function purchase(request: PurchaseRequest): Promise<Purchase> {
  return invoke<Purchase>(COMMANDS.purchase, { request });
}

export async function restorePurchases(): Promise<Purchase[]> {
  return invoke<Purchase[]>(COMMANDS.restorePurchases);
}

export async function validateReceipt(
  request: ReceiptValidationRequest,
): Promise<ReceiptValidationResult> {
  return invoke<ReceiptValidationResult>(COMMANDS.validateReceipt, {
    request,
  });
}

export async function requestPushAuthorization(
  request: PushAuthorizationRequest = {},
): Promise<PushAuthorization> {
  return invoke<PushAuthorization>(COMMANDS.requestPushAuthorization, {
    request,
  });
}

export async function registerPush(
  request: PushRegistrationRequest = {},
): Promise<PushRegistration> {
  return invoke<PushRegistration>(COMMANDS.registerPush, { request });
}

export async function unregisterPush(): Promise<void> {
  return invoke<void>(COMMANDS.unregisterPush);
}
