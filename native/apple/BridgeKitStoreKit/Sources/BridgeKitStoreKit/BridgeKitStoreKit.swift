import Foundation

#if canImport(StoreKit)
import StoreKit
#endif

#if os(Linux)
import Glibc
#else
import Darwin
#endif

private struct BridgeKitProductRequest: Decodable {
    let productIds: [String]
}

private struct BridgeKitPurchaseRequest: Decodable {
    let productId: String
    let quantity: UInt32?
    let appAccountToken: String?
}

private struct BridgeKitProductResponse: Encodable {
    let id: String
    let localizedTitle: String
    let localizedDescription: String
    let displayPrice: String
    let currencyCode: String
    let subscriptionPeriod: String?
    let introductoryPrice: String?
    let trialPeriod: String?
    let raw: [String: String]
}

private struct BridgeKitTransactionResponse: Encodable {
    let transactionId: String
    let productId: String
    let state: String
    let signedTransactionJws: String?
    let appStoreReceipt: String?
    let originalTransactionId: String?
    let purchasedAtMs: Int64?
    let expiresAtMs: Int64?
    let raw: [String: String]
}

@_cdecl("bridgekit_storekit_products_json")
public func bridgekitStoreKitProductsJson(_ requestJson: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>? {
    guard let requestJson else {
        return nil
    }

    let requestString = String(cString: requestJson)

    do {
        let requestData = Data(requestString.utf8)
        let request = try JSONDecoder().decode(BridgeKitProductRequest.self, from: requestData)
        let products = try fetchProductsSynchronously(productIds: request.productIds)
        let responseData = try JSONEncoder.bridgeKit.encode(products)
        guard let responseString = String(data: responseData, encoding: .utf8) else {
            return nil
        }
        return strdup(responseString)
    } catch {
        return nil
    }
}

@_cdecl("bridgekit_storekit_purchase_json")
public func bridgekitStoreKitPurchaseJson(_ requestJson: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>? {
    guard let requestJson else {
        return nil
    }

    let requestString = String(cString: requestJson)

    do {
        let requestData = Data(requestString.utf8)
        let request = try JSONDecoder().decode(BridgeKitPurchaseRequest.self, from: requestData)
        let transaction = try purchaseSynchronously(request: request)
        let responseData = try JSONEncoder.bridgeKit.encode(transaction)
        guard let responseString = String(data: responseData, encoding: .utf8) else {
            return nil
        }
        return strdup(responseString)
    } catch {
        return nil
    }
}

@_cdecl("bridgekit_string_free")
public func bridgekitStringFree(_ value: UnsafeMutablePointer<CChar>?) {
    free(value)
}

private func fetchProductsSynchronously(productIds: [String]) throws -> [BridgeKitProductResponse] {
    if productIds.isEmpty {
        return []
    }

    #if canImport(StoreKit)
    if #available(iOS 15.0, macOS 12.0, *) {
        let semaphore = DispatchSemaphore(value: 0)
        let lock = NSLock()
        var result: Result<[BridgeKitProductResponse], Error>?

        Task.detached {
            do {
                let products = try await Product.products(for: productIds)
                let response = products.map(BridgeKitProductResponse.init(storeKitProduct:))
                lock.lock()
                result = .success(response)
                lock.unlock()
            } catch {
                lock.lock()
                result = .failure(error)
                lock.unlock()
            }
            semaphore.signal()
        }

        semaphore.wait()
        lock.lock()
        defer { lock.unlock() }
        return try result!.get()
    }
    #endif

    throw BridgeKitStoreKitError.storeKitUnavailable
}

private func purchaseSynchronously(request: BridgeKitPurchaseRequest) throws -> BridgeKitTransactionResponse {
    #if canImport(StoreKit)
    if #available(iOS 15.0, macOS 12.0, *) {
        let semaphore = DispatchSemaphore(value: 0)
        let lock = NSLock()
        var result: Result<BridgeKitTransactionResponse, Error>?

        Task.detached {
            do {
                let products = try await Product.products(for: [request.productId])
                guard let product = products.first else {
                    throw BridgeKitStoreKitError.productNotFound(request.productId)
                }

                var options = Set<Product.PurchaseOption>()
                if let appAccountToken = request.appAccountToken,
                   let uuid = UUID(uuidString: appAccountToken) {
                    options.insert(.appAccountToken(uuid))
                }

                let purchaseResult = try await product.purchase(options: options)
                let response = try await BridgeKitTransactionResponse(
                    purchaseResult: purchaseResult,
                    productId: request.productId
                )
                lock.lock()
                result = .success(response)
                lock.unlock()
            } catch {
                lock.lock()
                result = .failure(error)
                lock.unlock()
            }
            semaphore.signal()
        }

        semaphore.wait()
        lock.lock()
        defer { lock.unlock() }
        return try result!.get()
    }
    #endif

    throw BridgeKitStoreKitError.storeKitUnavailable
}

private enum BridgeKitStoreKitError: Error {
    case storeKitUnavailable
    case productNotFound(String)
}

private extension JSONEncoder {
    static var bridgeKit: JSONEncoder {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        return encoder
    }
}

#if canImport(StoreKit)
@available(iOS 15.0, macOS 12.0, *)
private extension BridgeKitProductResponse {
    init(storeKitProduct product: Product) {
        let subscription = product.subscription
        let introductoryOffer = subscription?.introductoryOffer

        self.init(
            id: product.id,
            localizedTitle: product.displayName,
            localizedDescription: product.description,
            displayPrice: product.displayPrice,
            currencyCode: product.priceFormatStyle.currencyCode,
            subscriptionPeriod: subscription?.subscriptionPeriod.bridgeKitISO8601,
            introductoryPrice: introductoryOffer?.displayPrice,
            trialPeriod: introductoryOffer?.period.bridgeKitISO8601,
            raw: [
                "source": "storekit",
                "type": product.type.bridgeKitRawValue
            ]
        )
    }
}

@available(iOS 15.0, macOS 12.0, *)
private extension BridgeKitTransactionResponse {
    init(purchaseResult: Product.PurchaseResult, productId: String) async throws {
        switch purchaseResult {
        case .success(let verificationResult):
            switch verificationResult {
            case .verified(let transaction):
                await transaction.finish()
                self.init(
                    transaction: transaction,
                    state: "purchased",
                    verification: "verified"
                )
            case .unverified(let transaction, _):
                self.init(
                    transaction: transaction,
                    state: "failed",
                    verification: "unverified"
                )
            }
        case .userCancelled:
            self.init(
                transactionId: "cancelled-\(productId)",
                productId: productId,
                state: "cancelled",
                signedTransactionJws: nil,
                appStoreReceipt: nil,
                originalTransactionId: nil,
                purchasedAtMs: nil,
                expiresAtMs: nil,
                raw: [
                    "source": "storekit",
                    "result": "user_cancelled"
                ]
            )
        case .pending:
            self.init(
                transactionId: "pending-\(productId)",
                productId: productId,
                state: "deferred",
                signedTransactionJws: nil,
                appStoreReceipt: nil,
                originalTransactionId: nil,
                purchasedAtMs: nil,
                expiresAtMs: nil,
                raw: [
                    "source": "storekit",
                    "result": "pending"
                ]
            )
        @unknown default:
            self.init(
                transactionId: "unknown-\(productId)",
                productId: productId,
                state: "failed",
                signedTransactionJws: nil,
                appStoreReceipt: nil,
                originalTransactionId: nil,
                purchasedAtMs: nil,
                expiresAtMs: nil,
                raw: [
                    "source": "storekit",
                    "result": "unknown"
                ]
            )
        }
    }

    init(transaction: Transaction, state: String, verification: String) {
        self.init(
            transactionId: String(transaction.id),
            productId: transaction.productID,
            state: state,
            signedTransactionJws: String(data: transaction.jsonRepresentation, encoding: .utf8),
            appStoreReceipt: nil,
            originalTransactionId: String(transaction.originalID),
            purchasedAtMs: transaction.purchaseDate.bridgeKitMillisecondsSince1970,
            expiresAtMs: transaction.expirationDate?.bridgeKitMillisecondsSince1970,
            raw: [
                "source": "storekit",
                "verification": verification
            ]
        )
    }
}

@available(iOS 15.0, macOS 12.0, *)
private extension Product.SubscriptionPeriod {
    var bridgeKitISO8601: String {
        switch unit {
        case .day:
            "P\(value)D"
        case .week:
            "P\(value)W"
        case .month:
            "P\(value)M"
        case .year:
            "P\(value)Y"
        @unknown default:
            "P\(value)D"
        }
    }
}

@available(iOS 15.0, macOS 12.0, *)
private extension Product.ProductType {
    var bridgeKitRawValue: String {
        switch self {
        case .consumable:
            "consumable"
        case .nonConsumable:
            "non_consumable"
        case .nonRenewable:
            "non_renewable"
        case .autoRenewable:
            "auto_renewable"
        @unknown default:
            "unknown"
        }
    }
}

private extension Date {
    var bridgeKitMillisecondsSince1970: Int64 {
        Int64((timeIntervalSince1970 * 1000).rounded())
    }
}
#endif
