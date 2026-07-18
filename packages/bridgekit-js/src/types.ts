export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | JsonObject | JsonValue[];
export interface JsonObject {
  [key: string]: JsonValue;
}

export type Platform = "apple" | "microsoft" | "other";
export type Storefront = "apple_app_store" | "microsoft_store";

export type TransactionState =
  | "pending"
  | "purchased"
  | "restored"
  | "failed"
  | "deferred"
  | "cancelled";

export type PushAuthorizationStatus =
  | "not_determined"
  | "denied"
  | "authorized"
  | "provisional"
  | "ephemeral"
  | "unsupported";

export interface ProductRequest {
  productIds: string[];
  storefront?: Storefront;
  appUserId?: string;
  metadata?: JsonValue;
}

export interface Product {
  id: string;
  title: string;
  description: string;
  price: string;
  currencyCode: string;
  storefront: Storefront;
  subscription?: SubscriptionInfo;
  metadata?: JsonValue;
}

export interface SubscriptionInfo {
  period: string;
  introductoryPrice?: string;
  trialPeriod?: string;
}

export interface PurchaseRequest {
  productId: string;
  quantity?: number;
  appAccountToken?: string;
  metadata?: JsonValue;
}

export interface Purchase {
  transactionId: string;
  productId: string;
  state: TransactionState;
  storefront: Storefront;
  receipt?: string;
  originalTransactionId?: string;
  purchasedAtMs?: number;
  expiresAtMs?: number;
  metadata?: JsonValue;
}

export interface ReceiptValidationRequest {
  receipt: string;
  transactionId?: string;
  productId?: string;
  storefront: Storefront;
  metadata?: JsonValue;
}

export interface ReceiptValidationResult {
  isValid: boolean;
  productId?: string;
  transactionId?: string;
  expiresAtMs?: number;
  raw?: JsonValue;
}

export interface PushAuthorizationRequest {
  alert?: boolean;
  badge?: boolean;
  sound?: boolean;
  provisional?: boolean;
  metadata?: JsonValue;
}

export interface PushAuthorization {
  status: PushAuthorizationStatus;
  platform: Platform;
  metadata?: JsonValue;
}

export interface PushRegistrationRequest {
  environment?: "sandbox" | "production" | string;
  metadata?: JsonValue;
}

export interface PushRegistration {
  token: string;
  platform: Platform;
  environment?: string;
  expiresAtMs?: number;
  metadata?: JsonValue;
}

export interface NotificationPayload {
  id: string;
  title?: string;
  body?: string;
  badge?: number;
  sound?: string;
  data?: JsonObject;
}
